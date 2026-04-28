//! Main DiscordUser client combining HTTP, WebSocket, and event handling

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::{
    client::DiscordHttpClient,
    context::DiscordContext,
    error::Result,
    events::{DispatchEvent, EventEmitter, EventSubscription},
    gateway::Gateway,
    types::*,
};

/// Macro to generate typed event handler methods.
///
/// This macro reduces boilerplate by generating async methods that:
/// 1. Accept a callback function with the appropriate typed event struct
/// 2. Delegate to `on_typed_event_impl` with the event type string
///
/// Usage:
/// ```ignore
/// impl_typed_event_handlers! {
///     /// Doc comment
///     method_name => "EVENT_TYPE", EventStruct;
/// }
/// ```
macro_rules! impl_typed_event_handlers {
    ($(
        $(#[$meta:meta])*
        $method:ident => $event_type:literal, $event_struct:ty
    );* $(;)?) => {
        $(
            $(#[$meta])*
            pub async fn $method<F>(&self, callback: F) -> EventSubscription
            where
                F: Fn($event_struct) + Send + Sync + 'static,
            {
                self.on_typed_event_impl($event_type, callback).await
            }
        )*
    };
}

/// Main Discord self-bot client
pub struct DiscordUser {
    /// HTTP client for API requests
    http: DiscordHttpClient,
    /// WebSocket gateway connection
    gateway: Option<Gateway>,
    /// Event emitter for dispatch events
    events: EventEmitter,
    /// Current user data (populated after READY)
    user: Arc<RwLock<Option<User>>>,
    /// Friend list / relationships
    relationships: Arc<RwLock<Vec<Relationship>>>,
    /// Broadcast receiver for gateway events
    event_receiver: Option<broadcast::Receiver<DispatchEvent>>,
    /// Custom status to set on connect
    custom_status: UserStatus,
    /// Maximum number of reconnect attempts
    max_reconnect_attempts: u32,
    /// Size of the event buffer
    event_buffer_size: usize,
    /// Capabilities bitmask for the IDENTIFY payload (user-account intents)
    capabilities: u32,
    /// In-memory event-driven cache (guilds, users, messages)
    #[cfg(feature = "cache")]
    cache: crate::cache::Cache,
}

// Implement CacheHttp — DiscordUser has both a cache and an HTTP client.
#[cfg(feature = "cache")]
impl crate::cache_http::CacheHttp for DiscordUser {
    fn http(&self) -> &crate::client::DiscordHttpClient {
        &self.http
    }
    fn cache(&self) -> Option<&crate::cache::Cache> {
        Some(&self.cache)
    }
}

// Implement DiscordContext to provide access to core services
impl DiscordContext for DiscordUser {
    fn http(&self) -> &DiscordHttpClient {
        &self.http
    }

    fn events(&self) -> &EventEmitter {
        &self.events
    }

    fn gateway(&self) -> Option<&Gateway> {
        self.gateway.as_ref()
    }
}

impl DiscordUser {
    /// Create a new DiscordUser with a token
    pub fn new(token: impl Into<String>) -> Self {
        let token = token.into();
        Self {
            http: DiscordHttpClient::new(&token, None, false),
            gateway: None,
            events: EventEmitter::new(),
            user: Arc::new(RwLock::new(None)),
            relationships: Arc::new(RwLock::new(Vec::new())),
            event_receiver: None,
            custom_status: UserStatus::Online,
            max_reconnect_attempts: 5,
            event_buffer_size: 256,
            capabilities: 16381,
            #[cfg(feature = "cache")]
            cache: crate::cache::Cache::new(),
        }
    }

    /// Create a new DiscordUser with custom headers
    pub fn with_headers(headers: HashMap<String, String>) -> Option<Self> {
        let http = DiscordHttpClient::with_headers(headers.clone(), None, false)?;
        let _token = headers.iter().find(|(k, _)| k.to_lowercase() == "authorization").map(|(_, v)| v.clone())?;

        Some(Self {
            http,
            gateway: None,
            events: EventEmitter::new(),
            user: Arc::new(RwLock::new(None)),
            relationships: Arc::new(RwLock::new(Vec::new())),
            event_receiver: None,
            custom_status: UserStatus::Online,
            max_reconnect_attempts: 5,
            event_buffer_size: 256,
            capabilities: 16381,
            #[cfg(feature = "cache")]
            cache: crate::cache::Cache::new(),
        })
    }

    /// Set custom status to use when connecting
    pub fn with_status(mut self, status: UserStatus) -> Self {
        self.custom_status = status;
        self
    }

    /// Set the capabilities bitmask sent in the IDENTIFY payload.
    ///
    /// This is the user-account equivalent of gateway intents.  The default
    /// value (`16381`) matches what an unmodified Discord client sends.
    /// Pass [`GatewayIntents`] bits or a raw `u32` to customise.
    pub fn with_capabilities(mut self, capabilities: u32) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Initialize the WebSocket connection
    pub async fn init(&mut self) -> Result<()> {
        use crate::operations::RelationshipOps;

        let token = self.http.token().to_string();
        let (mut gateway, event_receiver) = Gateway::new_with_capabilities(token, self.custom_status, self.event_buffer_size, self.capabilities);

        gateway.connect_with_auto_reconnect(self.max_reconnect_attempts).await?;

        self.gateway = Some(gateway);
        self.event_receiver = Some(event_receiver);

        // Fetch initial relationships
        if let Ok(rels) = self.get_my_relationship().await {
            *self.relationships.write().await = rels;
        }

        // Start event processing
        self.start_event_processing();

        info!("DiscordUser initialized and connected");
        Ok(())
    }

    /// Start background task to process gateway events
    fn start_event_processing(&mut self) {
        if let Some(mut receiver) = self.event_receiver.take() {
            let events = self.events.clone();
            let user = Arc::clone(&self.user);
            let relationships = Arc::clone(&self.relationships);
            #[cfg(feature = "cache")]
            let cache = self.cache.clone();

            tokio::spawn(async move {
                loop {
                    match receiver.recv().await {
                        Ok(event) => {
                            // Update cache BEFORE dispatching so event handlers
                            // see the updated cache state (matches serenity's ordering).
                            match event.event_type.as_str() {
                                "READY" => {
                                    handle_ready_event(&event.data, &user, &relationships).await;
                                }
                                #[cfg(feature = "cache")]
                                "GUILD_CREATE" | "GUILD_UPDATE" => {
                                    if let Ok(guild) = serde_json::from_value::<Guild>(event.data.clone()) {
                                        for channel in &guild.channels {
                                            cache.upsert_channel(channel.clone());
                                        }
                                        for role in &guild.roles {
                                            cache.upsert_role(role.clone());
                                        }
                                        cache.upsert_guild(guild);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "GUILD_DELETE" => {
                                    if let Some(id) = event.data["id"].as_str() {
                                        cache.remove_guild(id);
                                    }
                                }
                                // Cache message authors and mentioned users; store the message
                                #[cfg(feature = "cache")]
                                "MESSAGE_CREATE" | "MESSAGE_UPDATE" => {
                                    if let Ok(author) = serde_json::from_value::<User>(event.data["author"].clone()) {
                                        cache.upsert_user(author);
                                    }
                                    if let Ok(msg) = serde_json::from_value::<Message>(event.data.clone()) {
                                        cache.upsert_message(msg);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "MESSAGE_DELETE" => {
                                    let channel_id = event.data["channel_id"].as_str().unwrap_or("");
                                    let message_id = event.data["id"].as_str().unwrap_or("");
                                    if !channel_id.is_empty() && !message_id.is_empty() {
                                        cache.remove_message(channel_id, message_id);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "MESSAGE_DELETE_BULK" => {
                                    let channel_id = event.data["channel_id"].as_str().unwrap_or("");
                                    if let Some(ids) = event.data["ids"].as_array() {
                                        for id in ids {
                                            if let Some(mid) = id.as_str() {
                                                cache.remove_message(channel_id, mid);
                                            }
                                        }
                                    }
                                }
                                // Cache guild members' user objects
                                #[cfg(feature = "cache")]
                                "GUILD_MEMBER_ADD" | "GUILD_MEMBER_UPDATE" => {
                                    if let Ok(member_user) = serde_json::from_value::<User>(event.data["user"].clone()) {
                                        cache.upsert_user(member_user);
                                    }
                                }
                                // Keep current user up to date
                                #[cfg(feature = "cache")]
                                "USER_UPDATE" => {
                                    if let Ok(updated_user) = serde_json::from_value::<User>(event.data.clone()) {
                                        cache.upsert_user(updated_user);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "CHANNEL_CREATE" | "CHANNEL_UPDATE" => {
                                    if let Ok(channel) = serde_json::from_value::<Channel>(event.data.clone()) {
                                        cache.upsert_channel(channel);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "CHANNEL_DELETE" => {
                                    if let Some(id) = event.data["id"].as_str() {
                                        cache.remove_channel(id);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "GUILD_ROLE_CREATE" | "GUILD_ROLE_UPDATE" => {
                                    if let Ok(role) = serde_json::from_value::<Role>(event.data["role"].clone()) {
                                        cache.upsert_role(role);
                                    }
                                }
                                #[cfg(feature = "cache")]
                                "GUILD_ROLE_DELETE" => {
                                    if let Some(role_id) = event.data["role_id"].as_str() {
                                        cache.remove_role(role_id);
                                    }
                                }
                                "RELATIONSHIP_ADD" => {
                                    handle_relationship_add(&event.data, &relationships).await;
                                }
                                "RELATIONSHIP_REMOVE" => {
                                    handle_relationship_remove(&event.data, &relationships).await;
                                }
                                _ => {}
                            }

                            // Dispatch to user listeners
                            events.dispatch(event).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!("Event receiver lagged by {} messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            debug!("Event channel closed");
                            break;
                        }
                    }
                }
            });
        }
    }

    /// Get the current user data
    pub async fn get_user(&self) -> Option<User> {
        self.user.read().await.clone()
    }

    /// Get current relationships
    pub async fn get_relationships(&self) -> Vec<Relationship> {
        self.relationships.read().await.clone()
    }

    /// Access the in-memory cache (guilds, users, messages).
    #[cfg(feature = "cache")]
    pub fn cache(&self) -> &crate::cache::Cache {
        &self.cache
    }

    // ==================== Typed Event Handlers ====================
    //
    // These methods are generated by the `impl_typed_event_handlers!` macro
    // to reduce code duplication. Each handler registers a callback for a
    // specific Discord event type with automatic deserialization.

    impl_typed_event_handlers! {
        // ---- Messages ----
        /// Register a handler for MESSAGE_CREATE events with typed data
        on_message_create => "MESSAGE_CREATE", crate::typed_events::MessageCreateEvent;
        /// Register a handler for MESSAGE_UPDATE events with typed data
        on_message_update => "MESSAGE_UPDATE", crate::typed_events::MessageUpdateEvent;
        /// Register a handler for MESSAGE_DELETE events with typed data
        on_message_delete => "MESSAGE_DELETE", crate::typed_events::MessageDeleteEvent;
        /// Register a handler for MESSAGE_DELETE_BULK events with typed data
        on_message_delete_bulk => "MESSAGE_DELETE_BULK", crate::typed_events::MessageDeleteBulkEvent;

        // ---- Reactions ----
        /// Register a handler for MESSAGE_REACTION_ADD events with typed data
        on_reaction_add => "MESSAGE_REACTION_ADD", crate::typed_events::MessageReactionAddEvent;
        /// Register a handler for MESSAGE_REACTION_REMOVE events with typed data
        on_reaction_remove => "MESSAGE_REACTION_REMOVE", crate::typed_events::MessageReactionRemoveEvent;
        /// Register a handler for MESSAGE_REACTION_REMOVE_ALL events with typed data
        on_reaction_remove_all => "MESSAGE_REACTION_REMOVE_ALL", crate::typed_events::MessageReactionRemoveAllEvent;
        /// Register a handler for MESSAGE_REACTION_REMOVE_EMOJI events with typed data
        on_reaction_remove_emoji => "MESSAGE_REACTION_REMOVE_EMOJI", crate::typed_events::MessageReactionRemoveEmojiEvent;

        // ---- Typing ----
        /// Register a handler for TYPING_START events with typed data
        on_typing_start => "TYPING_START", crate::typed_events::TypingStartEvent;

        // ---- Relationships ----
        /// Register a handler for RELATIONSHIP_ADD events with typed data
        on_relationship_add => "RELATIONSHIP_ADD", crate::typed_events::RelationshipAddEvent;
        /// Register a handler for RELATIONSHIP_REMOVE events with typed data
        on_relationship_remove => "RELATIONSHIP_REMOVE", crate::typed_events::RelationshipRemoveEvent;

        // ---- Presence ----
        /// Register a handler for PRESENCE_UPDATE events with typed data
        on_presence_update => "PRESENCE_UPDATE", crate::typed_events::PresenceUpdateEvent;

        // ---- Guild members ----
        /// Register a handler for GUILD_MEMBER_ADD events with typed data
        on_guild_member_add => "GUILD_MEMBER_ADD", crate::typed_events::GuildMemberAddEvent;
        /// Register a handler for GUILD_MEMBER_REMOVE events with typed data
        on_guild_member_remove => "GUILD_MEMBER_REMOVE", crate::typed_events::GuildMemberRemoveEvent;
        /// Register a handler for GUILD_MEMBER_UPDATE events with typed data
        on_guild_member_update => "GUILD_MEMBER_UPDATE", crate::typed_events::GuildMemberUpdateEvent;

        // ---- Channels ----
        /// Register a handler for CHANNEL_CREATE events with typed data
        on_channel_create => "CHANNEL_CREATE", crate::typed_events::ChannelCreateEvent;
        /// Register a handler for CHANNEL_UPDATE events with typed data
        on_channel_update => "CHANNEL_UPDATE", crate::typed_events::ChannelUpdateEvent;
        /// Register a handler for CHANNEL_DELETE events with typed data
        on_channel_delete => "CHANNEL_DELETE", crate::typed_events::ChannelDeleteEvent;
        /// Register a handler for CHANNEL_PINS_UPDATE events with typed data
        on_channel_pins_update => "CHANNEL_PINS_UPDATE", crate::typed_events::ChannelPinsUpdateEvent;

        // ---- Guilds ----
        /// Register a handler for GUILD_CREATE events with typed data
        on_guild_create => "GUILD_CREATE", crate::typed_events::GuildCreateEvent;
        /// Register a handler for GUILD_UPDATE events with typed data
        on_guild_update => "GUILD_UPDATE", crate::typed_events::GuildUpdateEvent;
        /// Register a handler for GUILD_DELETE events with typed data
        on_guild_delete => "GUILD_DELETE", crate::typed_events::GuildDeleteEvent;

        // ---- Guild roles ----
        /// Register a handler for GUILD_ROLE_CREATE events with typed data
        on_guild_role_create => "GUILD_ROLE_CREATE", crate::typed_events::GuildRoleCreateEvent;
        /// Register a handler for GUILD_ROLE_UPDATE events with typed data
        on_guild_role_update => "GUILD_ROLE_UPDATE", crate::typed_events::GuildRoleUpdateEvent;
        /// Register a handler for GUILD_ROLE_DELETE events with typed data
        on_guild_role_delete => "GUILD_ROLE_DELETE", crate::typed_events::GuildRoleDeleteEvent;

        // ---- Guild bans ----
        /// Register a handler for GUILD_BAN_ADD events with typed data
        on_guild_ban_add => "GUILD_BAN_ADD", crate::typed_events::GuildBanAddEvent;
        /// Register a handler for GUILD_BAN_REMOVE events with typed data
        on_guild_ban_remove => "GUILD_BAN_REMOVE", crate::typed_events::GuildBanRemoveEvent;

        // ---- Threads ----
        /// Register a handler for THREAD_CREATE events with typed data
        on_thread_create => "THREAD_CREATE", crate::typed_events::ThreadCreateEvent;
        /// Register a handler for THREAD_UPDATE events with typed data
        on_thread_update => "THREAD_UPDATE", crate::typed_events::ThreadUpdateEvent;
        /// Register a handler for THREAD_DELETE events with typed data
        on_thread_delete => "THREAD_DELETE", crate::typed_events::ThreadDeleteEvent;
        /// Register a handler for THREAD_LIST_SYNC events with typed data
        on_thread_list_sync => "THREAD_LIST_SYNC", crate::typed_events::ThreadListSyncEvent;
        /// Register a handler for THREAD_MEMBER_UPDATE events with typed data
        on_thread_member_update => "THREAD_MEMBER_UPDATE", crate::typed_events::ThreadMemberUpdateEvent;

        // ---- Voice ----
        /// Register a handler for VOICE_STATE_UPDATE events with typed data
        on_voice_state_update => "VOICE_STATE_UPDATE", crate::typed_events::VoiceStateUpdateEvent;

        // ---- User ----
        /// Register a handler for USER_UPDATE events with typed data
        on_user_update => "USER_UPDATE", crate::typed_events::UserUpdateEvent;

        // ---- Interactions ----
        /// Register a handler for INTERACTION_CREATE events with typed data
        on_interaction_create => "INTERACTION_CREATE", crate::typed_events::InteractionCreateEvent;
    }

    /// Internal helper to register a typed event handler with deserialization.
    /// This centralizes the common logic for all typed event handlers.
    async fn on_typed_event_impl<T, F>(&self, event_type: &'static str, callback: F) -> EventSubscription
    where
        T: serde::de::DeserializeOwned + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        self.events
            .on_event(event_type, move |event| match serde_json::from_value(event.data.clone()) {
                Ok(typed) => callback(typed),
                Err(e) => warn!(
                    event_type = event_type,
                    error = %e,
                    "Failed to deserialize typed event"
                ),
            })
            .await
    }

    /// Register a handler for all events with typed data via TypedEvent enum
    pub async fn on_typed_event<F>(&self, callback: F) -> EventSubscription
    where
        F: Fn(crate::typed_events::TypedEvent) + Send + Sync + 'static,
    {
        self.events
            .on_any_event(move |event| {
                let typed = crate::typed_events::TypedEvent::from_raw(&event.event_type, event.data);
                callback(typed);
            })
            .await
    }

    // ==================== Connection Management ====================

    /// Disconnect from the gateway
    pub async fn disconnect(&mut self) {
        if let Some(ref mut gateway) = self.gateway {
            gateway.disconnect().await;
        }
        self.gateway = None;
    }

    /// Check if connected to gateway
    pub async fn is_connected(&self) -> bool {
        if let Some(ref gateway) = self.gateway {
            gateway.is_connected().await
        } else {
            false
        }
    }

    /// Get the current connection stage of the gateway
    pub async fn connection_stage(&self) -> crate::types::ConnectionStage {
        if let Some(ref gateway) = self.gateway {
            gateway.stage().await
        } else {
            crate::types::ConnectionStage::Disconnected
        }
    }

    /// Return the round-trip gateway latency measured from the most recent
    /// heartbeat/ACK pair.  Returns `None` until the first ACK is received
    /// or when no gateway connection is active.
    ///
    /// Mirrors serenity's `Context::shard_latency()` / `Shard::latency()`.
    pub async fn latency(&self) -> Option<std::time::Duration> {
        if let Some(ref gateway) = self.gateway {
            gateway.latency().await
        } else {
            None
        }
    }

    /// Join (or move to) a voice channel in a guild via gateway opcode 4.
    ///
    /// Discord responds with a `VOICE_STATE_UPDATE` dispatch event.
    /// Pass `self_mute` / `self_deaf` to set initial mute/deaf state.
    ///
    /// Mirrors serenity's voice `join()` entry point.
    pub async fn join_voice_channel(&self, guild_id: &crate::types::GuildId, channel_id: &crate::types::ChannelId, self_mute: bool, self_deaf: bool) -> Result<()> {
        let gateway = self.gateway.as_ref().ok_or(crate::error::DiscordError::NotInitialized)?;
        gateway.send_voice_state_update(guild_id.get(), Some(channel_id.get()), self_mute, self_deaf).await
    }

    /// Leave the current voice channel in a guild via gateway opcode 4.
    ///
    /// Sends a Voice State Update with `channel_id = null`.
    pub async fn leave_voice_channel(&self, guild_id: &crate::types::GuildId) -> Result<()> {
        let gateway = self.gateway.as_ref().ok_or(crate::error::DiscordError::NotInitialized)?;
        gateway.send_voice_state_update(guild_id.get(), None, false, false).await
    }

    /// Reconnect to the gateway (disconnect then init)
    pub async fn reconnect(&mut self) -> Result<()> {
        warn!("Reconnecting to Discord gateway...");
        self.disconnect().await;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        self.init().await
    }

    /// Fetch user data from API (not cached)
    pub async fn fetch_user_data(&self) -> Result<User> {
        self.http.get(crate::route::Route::GetMe).await
    }

    /// Return a snapshot of the current connection state.
    ///
    /// Bundles `stage`, `latency`, and whether a gateway connection is active
    /// into a single struct.  Mirrors serenity's `ShardRunnerInfo`.
    pub async fn connection_info(&self) -> ConnectionInfo {
        match self.gateway.as_ref() {
            Some(gw) => ConnectionInfo { stage: gw.stage().await, latency: gw.latency().await, connected: true },
            None => ConnectionInfo { stage: crate::types::ConnectionStage::Disconnected, latency: None, connected: false },
        }
    }
}

/// A snapshot of the current gateway connection state.
///
/// Returned by [`DiscordUser::connection_info`].  Mirrors serenity's
/// `ShardRunnerInfo` which bundles latency, stage, and connection status.
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Current connection stage (e.g. Connected, Disconnected, Resuming).
    pub stage: crate::types::ConnectionStage,
    /// Round-trip gateway latency from the most recent heartbeat/ACK pair.
    /// `None` until the first heartbeat ACK is received.
    pub latency: Option<std::time::Duration>,
    /// Whether a gateway connection is currently active.
    pub connected: bool,
}

/// Internal struct for parsing READY event user
#[derive(Debug, serde::Deserialize)]
struct ReadyEventUser {
    id: String,
    username: String,
    #[serde(default)]
    discriminator: String,
    #[serde(default)]
    global_name: Option<String>,
    #[serde(default)]
    avatar: Option<crate::types::ImageHash>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    verified: bool,
    #[serde(default)]
    mfa_enabled: bool,
}

impl From<ReadyEventUser> for User {
    fn from(ready: ReadyEventUser) -> Self {
        User {
            id: ready.id,
            username: ready.username,
            discriminator: ready.discriminator,
            global_name: ready.global_name,
            avatar: ready.avatar,
            email: ready.email,
            verified: ready.verified,
            mfa_enabled: ready.mfa_enabled,
            ..Default::default()
        }
    }
}

// ==================== Event Processing Helpers ====================
//
// These functions are extracted from the event processing loop to improve
// readability and maintainability. Each function handles a specific event type.

/// Handle READY event: populate user data and relationships from the event
/// payload
async fn handle_ready_event(data: &serde_json::Value, user: &Arc<RwLock<Option<User>>>, relationships: &Arc<RwLock<Vec<Relationship>>>) {
    // Parse and store user data
    if let Some(user_data) = data.get("user") {
        if let Ok(ready_user) = serde_json::from_value::<ReadyEventUser>(user_data.clone()) {
            *user.write().await = Some(ready_user.into());
        }
    }

    // Parse and store relationships
    if let Some(rels_data) = data.get("relationships") {
        if let Ok(rels) = serde_json::from_value::<Vec<Relationship>>(rels_data.clone()) {
            *relationships.write().await = rels;
        }
    }
}

/// Handle RELATIONSHIP_ADD event: add or update a relationship in the list
async fn handle_relationship_add(data: &serde_json::Value, relationships: &Arc<RwLock<Vec<Relationship>>>) {
    if let Ok(rel) = serde_json::from_value::<Relationship>(data.clone()) {
        let mut rels = relationships.write().await;
        // Update existing relationship or add new one
        if let Some(pos) = rels.iter().position(|r| r.id == rel.id) {
            rels[pos] = rel;
        } else {
            rels.push(rel);
        }
    }
}

/// Handle RELATIONSHIP_REMOVE event: remove a relationship from the list
async fn handle_relationship_remove(data: &serde_json::Value, relationships: &Arc<RwLock<Vec<Relationship>>>) {
    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
        let mut rels = relationships.write().await;
        rels.retain(|r| r.id != id);
    }
}

/// Builder for creating a DiscordUser with validation
pub struct DiscordUserBuilder {
    token: Option<String>,
    custom_status: UserStatus,
    max_reconnect_attempts: u32,
    event_buffer_size: usize,
    ratelimit_callback: Option<std::sync::Arc<dyn Fn(crate::client::RatelimitInfo) + Send + Sync>>,
    proxy: Option<String>,
    ratelimiter_disabled: bool,
    capabilities: u32,
    #[cfg(feature = "cache")]
    cache_settings: crate::cache::CacheSettings,
}

impl Default for DiscordUserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscordUserBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            token: None,
            custom_status: UserStatus::Online,
            max_reconnect_attempts: 5,
            event_buffer_size: 256,
            ratelimit_callback: None,
            proxy: None,
            ratelimiter_disabled: false,
            capabilities: 16381,
            #[cfg(feature = "cache")]
            cache_settings: crate::cache::CacheSettings::default(),
        }
    }

    /// Configure the in-memory cache behaviour.
    ///
    /// # Example
    /// ```ignore
    /// DiscordUser::builder()
    ///     .token("…")
    ///     .cache_settings(CacheSettings { cache_users: false, max_messages: 50, ..Default::default() })
    ///     .build()
    /// ```
    #[cfg(feature = "cache")]
    pub fn cache_settings(mut self, settings: crate::cache::CacheSettings) -> Self {
        self.cache_settings = settings;
        self
    }

    /// Set the Discord token (required)
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the initial status
    pub fn status(mut self, status: UserStatus) -> Self {
        self.custom_status = status;
        self
    }

    /// Set maximum reconnection attempts
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Set event broadcast channel buffer size
    pub fn event_buffer_size(mut self, size: usize) -> Self {
        self.event_buffer_size = size;
        self
    }

    /// Set a callback for rate limits
    pub fn ratelimit_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(crate::client::RatelimitInfo) + Send + Sync + 'static,
    {
        self.ratelimit_callback = Some(std::sync::Arc::new(callback));
        self
    }

    /// Set an HTTP/HTTPS proxy
    pub fn proxy(mut self, url: impl Into<String>) -> Self {
        self.proxy = Some(url.into());
        self
    }

    /// Disable the ratelimiter (respect API rate limits yourself)
    pub fn ratelimiter_disabled(mut self, disabled: bool) -> Self {
        self.ratelimiter_disabled = disabled;
        self
    }

    /// Build the DiscordUser
    pub fn build(self) -> Result<DiscordUser> {
        let token = self.token.ok_or_else(|| crate::error::DiscordError::InvalidRequest("Token is required".into()))?;

        let mut http = DiscordHttpClient::new(&token, self.proxy, self.ratelimiter_disabled);
        if let Some(cb) = self.ratelimit_callback {
            http.set_ratelimit_callback(cb);
        }

        Ok(DiscordUser {
            http,
            gateway: None,
            events: EventEmitter::new(),
            user: Arc::new(RwLock::new(None)),
            relationships: Arc::new(RwLock::new(Vec::new())),
            event_receiver: None,
            custom_status: self.custom_status,
            max_reconnect_attempts: self.max_reconnect_attempts,
            event_buffer_size: self.event_buffer_size,
            capabilities: self.capabilities,
            #[cfg(feature = "cache")]
            cache: crate::cache::Cache::with_settings(self.cache_settings),
        })
    }
}
