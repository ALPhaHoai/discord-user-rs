//! In-memory event-driven cache for Discord objects.
//!
//! The cache is updated from gateway events *before* listeners are notified,
//! matching serenity's `update_cache_with_event()` ordering guarantee.
//!
//! # Supported caches
//! - [`Cache::guild`] / [`Cache::guilds`] — keyed by guild ID string
//! - [`Cache::user`]  / [`Cache::users`]  — keyed by user ID string
//! - [`Cache::message`] / [`Cache::channel_messages`] — per-channel LRU ring
//!
//! # Usage
//! ```ignore
//! // Cache is populated automatically after init():
//! if let Some(guild) = user.cache().guild("123456789") {
//!     println!("Guild: {:?}", guild.name);
//! }
//! if let Some(msg) = user.cache().message("channel_id", "message_id") {
//!     println!("Cached message: {}", msg.content);
//! }
//! ```

use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;

use crate::types::{Channel, Guild, Message, Role, User};

/// Default maximum number of messages kept per channel.
pub const DEFAULT_MAX_MESSAGES: usize = 100;

/// Controls which objects are cached and how large each cache may grow.
///
/// Pass to [`Cache::with_settings`] or [`DiscordUserBuilder::cache_settings`].
///
/// # Example
/// ```
/// use std::time::Duration;
///
/// use discord_user::cache::CacheSettings;
/// let settings = CacheSettings {
///     cache_guilds: true,
///     cache_users: true,
///     cache_messages: true,
///     max_messages: 50,
///     time_to_live: Some(Duration::from_secs(3600)),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CacheSettings {
    /// Cache guilds populated from GUILD_CREATE/UPDATE/DELETE events.
    pub cache_guilds: bool,
    /// Cache users from message authors and guild members.
    pub cache_users: bool,
    /// Cache messages per-channel with LRU eviction.
    pub cache_messages: bool,
    /// Maximum messages stored per channel.  0 disables message caching
    /// regardless of `cache_messages`.
    pub max_messages: usize,
    /// Optional TTL for guild and user cache entries.  Entries older than this
    /// duration are evicted lazily on the next read access.
    /// `None` (the default) means entries never expire.
    pub time_to_live: Option<Duration>,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            cache_guilds: true,
            cache_users: true,
            cache_messages: true,
            max_messages: DEFAULT_MAX_MESSAGES,
            time_to_live: None,
        }
    }
}

/// Per-channel message ring — a `VecDeque` of messages capped at
/// `max_messages`. The front is the oldest; the back is the newest.
struct MessageRing {
    messages: VecDeque<Message>,
    max: usize,
}

impl MessageRing {
    fn new(max: usize) -> Self {
        Self { messages: VecDeque::with_capacity(max.min(128)), max }
    }

    fn push(&mut self, msg: Message) {
        // Update existing message (edit) if ID already present
        if let Some(pos) = self.messages.iter().position(|m| m.id == msg.id) {
            self.messages[pos] = msg;
            return;
        }
        if self.max == 0 {
            return;
        }
        if self.messages.len() >= self.max {
            self.messages.pop_front(); // evict oldest
        }
        self.messages.push_back(msg);
    }

    fn remove(&mut self, message_id: &str) -> Option<Message> {
        if let Some(pos) = self.messages.iter().position(|m| m.id == message_id) {
            return self.messages.remove(pos);
        }
        None
    }

    fn get(&self, message_id: &str) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == message_id)
    }

    fn all(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }
}

/// Shared, thread-safe in-memory cache.
///
/// Clone is cheap — all fields are `Arc`-backed.
#[derive(Clone)]
pub struct Cache {
    /// Active settings controlling what is cached.
    settings: CacheSettings,
    /// Guild objects keyed by guild ID string.
    /// Populated on GUILD_CREATE, updated on GUILD_UPDATE, removed on
    /// GUILD_DELETE.
    guilds: Arc<DashMap<String, Guild>>,
    /// Insertion timestamps for guild entries (used for TTL eviction).
    guild_timestamps: Arc<DashMap<String, Instant>>,
    /// User objects keyed by user ID string.
    /// Populated from message authors, guild member data, and USER_UPDATE
    /// events.
    users: Arc<DashMap<String, User>>,
    /// Insertion timestamps for user entries (used for TTL eviction).
    user_timestamps: Arc<DashMap<String, Instant>>,
    /// Per-channel message ring-buffers.
    /// Key = channel_id string.  Each ring holds at most `max_messages`
    /// messages.
    messages: Arc<DashMap<String, MessageRing>>,
    /// Channel objects keyed by channel ID string.
    /// Populated on GUILD_CREATE (from guild channels),
    /// CHANNEL_CREATE/UPDATE/DELETE.
    channels: Arc<DashMap<String, Channel>>,
    /// Role objects keyed by role ID string.
    /// Populated on GUILD_CREATE (from guild roles),
    /// GUILD_ROLE_CREATE/UPDATE/DELETE.
    roles: Arc<DashMap<String, Role>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::with_settings(CacheSettings::default())
    }
}

impl Cache {
    /// Create a cache with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a cache governed by the given `CacheSettings`.
    pub fn with_settings(settings: CacheSettings) -> Self {
        Self {
            settings,
            guilds: Arc::new(DashMap::new()),
            guild_timestamps: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            user_timestamps: Arc::new(DashMap::new()),
            messages: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            roles: Arc::new(DashMap::new()),
        }
    }

    /// Create a cache that keeps at most `max_messages` messages per channel.
    /// Pass `0` to disable message caching entirely.
    pub fn with_max_messages(max_messages: usize) -> Self {
        Self::with_settings(CacheSettings { max_messages, ..CacheSettings::default() })
    }

    /// Return a reference to the active cache settings.
    pub fn settings(&self) -> &CacheSettings {
        &self.settings
    }

    // ── Guild ───────────────────────────────────────────────────────────────

    /// Look up a guild by its ID string.
    ///
    /// Returns `None` if the entry does not exist or has exceeded its TTL
    /// (lazily evicted on access when `CacheSettings::time_to_live` is set).
    pub fn guild(&self, guild_id: &str) -> Option<Guild> {
        if let Some(ttl) = self.settings.time_to_live {
            if let Some(ts) = self.guild_timestamps.get(guild_id) {
                if ts.elapsed() > ttl {
                    drop(ts);
                    self.guilds.remove(guild_id);
                    self.guild_timestamps.remove(guild_id);
                    return None;
                }
            }
        }
        self.guilds.get(guild_id).map(|g| g.clone())
    }

    /// Return an iterator over all cached guilds (excluding any TTL-expired
    /// entries).
    ///
    /// Each item is a clone of the stored `Guild`; the cache is not locked for
    /// the duration of the iteration.
    pub fn guilds(&self) -> Vec<Guild> {
        if let Some(ttl) = self.settings.time_to_live {
            let expired: Vec<String> = self.guild_timestamps.iter().filter(|r| r.value().elapsed() > ttl).map(|r| r.key().clone()).collect();
            for id in expired {
                self.guilds.remove(&id);
                self.guild_timestamps.remove(&id);
            }
        }
        self.guilds.iter().map(|r| r.value().clone()).collect()
    }

    /// Number of guilds currently in the cache (including any not-yet-evicted
    /// expired entries).
    pub fn guild_count(&self) -> usize {
        self.guilds.len()
    }

    // ── User ────────────────────────────────────────────────────────────────

    /// Look up a user by their ID string.
    ///
    /// Returns `None` if the entry does not exist or has exceeded its TTL
    /// (lazily evicted on access when `CacheSettings::time_to_live` is set).
    pub fn user(&self, user_id: &str) -> Option<User> {
        if let Some(ttl) = self.settings.time_to_live {
            if let Some(ts) = self.user_timestamps.get(user_id) {
                if ts.elapsed() > ttl {
                    drop(ts);
                    self.users.remove(user_id);
                    self.user_timestamps.remove(user_id);
                    return None;
                }
            }
        }
        self.users.get(user_id).map(|u| u.clone())
    }

    /// Return all cached users (excluding any TTL-expired entries).
    pub fn users(&self) -> Vec<User> {
        if let Some(ttl) = self.settings.time_to_live {
            let expired: Vec<String> = self.user_timestamps.iter().filter(|r| r.value().elapsed() > ttl).map(|r| r.key().clone()).collect();
            for id in expired {
                self.users.remove(&id);
                self.user_timestamps.remove(&id);
            }
        }
        self.users.iter().map(|r| r.value().clone()).collect()
    }

    /// Number of users currently in the cache (including any not-yet-evicted
    /// expired entries).
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    // ── Message ─────────────────────────────────────────────────────────────

    /// Look up a specific message by channel and message ID.
    pub fn message(&self, channel_id: &str, message_id: &str) -> Option<Message> {
        self.messages.get(channel_id)?.get(message_id).cloned()
    }

    /// Return all cached messages for a channel, oldest first.
    pub fn channel_messages(&self, channel_id: &str) -> Vec<Message> {
        self.messages.get(channel_id).map(|r| r.all()).unwrap_or_default()
    }

    /// Total number of cached messages across all channels.
    pub fn message_count(&self) -> usize {
        self.messages.iter().map(|r| r.messages.len()).sum()
    }

    // ── Channel ─────────────────────────────────────────────────────────────

    /// Look up a channel by its ID string.
    pub fn channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.get(channel_id).map(|c| c.clone())
    }

    /// Return all cached channels.
    pub fn channels(&self) -> Vec<Channel> {
        self.channels.iter().map(|r| r.value().clone()).collect()
    }

    /// Number of channels currently in the cache.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    // ── Role ────────────────────────────────────────────────────────────────

    /// Look up a role by its ID string.
    pub fn role(&self, role_id: &str) -> Option<Role> {
        self.roles.get(role_id).map(|r| r.clone())
    }

    /// Return all cached roles.
    pub fn roles(&self) -> Vec<Role> {
        self.roles.iter().map(|r| r.value().clone()).collect()
    }

    /// Number of roles currently in the cache.
    pub fn role_count(&self) -> usize {
        self.roles.len()
    }

    // ── Internal mutators (called by event processing) ───────────────────────

    /// Insert or replace a guild in the cache.  No-op if `cache_guilds` is
    /// false.
    pub(crate) fn upsert_guild(&self, guild: Guild) {
        if self.settings.cache_guilds {
            self.guild_timestamps.insert(guild.id.clone(), Instant::now());
            self.guilds.insert(guild.id.clone(), guild);
        }
    }

    /// Remove a guild from the cache.  Returns the removed guild if present.
    pub(crate) fn remove_guild(&self, guild_id: &str) -> Option<Guild> {
        self.guild_timestamps.remove(guild_id);
        self.guilds.remove(guild_id).map(|(_, g)| g)
    }

    /// Insert or replace a user in the cache.  No-op if `cache_users` is false.
    pub(crate) fn upsert_user(&self, user: User) {
        if self.settings.cache_users {
            self.user_timestamps.insert(user.id.clone(), Instant::now());
            self.users.insert(user.id.clone(), user);
        }
    }

    /// Insert or update a message in the per-channel ring.
    /// No-op if `cache_messages` is false or `max_messages == 0`.
    pub(crate) fn upsert_message(&self, msg: Message) {
        if !self.settings.cache_messages || self.settings.max_messages == 0 {
            return;
        }
        let max = self.settings.max_messages;
        self.messages.entry(msg.channel_id.clone()).or_insert_with(|| MessageRing::new(max)).push(msg);
    }

    /// Remove a message from its channel ring.
    pub(crate) fn remove_message(&self, channel_id: &str, message_id: &str) -> Option<Message> {
        self.messages.get_mut(channel_id)?.remove(message_id)
    }

    /// Insert or replace a channel in the cache.
    pub(crate) fn upsert_channel(&self, channel: Channel) {
        self.channels.insert(channel.id.clone(), channel);
    }

    /// Remove a channel from the cache. Returns the removed channel if present.
    pub(crate) fn remove_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.remove(channel_id).map(|(_, c)| c)
    }

    /// Insert or replace a role in the cache.
    pub(crate) fn upsert_role(&self, role: Role) {
        self.roles.insert(role.id.clone(), role);
    }

    /// Remove a role from the cache. Returns the removed role if present.
    pub(crate) fn remove_role(&self, role_id: &str) -> Option<Role> {
        self.roles.remove(role_id).map(|(_, r)| r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Guild;

    fn make_guild(id: &str, name: &str) -> Guild {
        Guild {
            id: id.to_string(),
            name: Some(name.to_string()),
            icon: None,
            splash: None,
            banner: None,
            description: None,
            owner_id: None,
            member_count: None,
            premium_subscription_count: 0,
            premium_tier: 0,
            verification_level: 0,
            nsfw_level: 0,
            nsfw: false,
            features: vec![],
            roles: vec![],
            channels: vec![],
            emojis: vec![],
            stickers: vec![],
            joined_at: None,
            large: false,
            lazy: false,
        }
    }

    #[test]
    fn upsert_and_get_guild() {
        let cache = Cache::new();
        cache.upsert_guild(make_guild("111", "Test Guild"));
        let g = cache.guild("111").expect("guild should be cached");
        assert_eq!(g.name.as_deref(), Some("Test Guild"));
    }

    #[test]
    fn update_guild_replaces_entry() {
        let cache = Cache::new();
        cache.upsert_guild(make_guild("222", "Old Name"));
        cache.upsert_guild(make_guild("222", "New Name"));
        assert_eq!(cache.guild("222").unwrap().name.as_deref(), Some("New Name"));
        assert_eq!(cache.guild_count(), 1);
    }

    #[test]
    fn remove_guild() {
        let cache = Cache::new();
        cache.upsert_guild(make_guild("333", "To Remove"));
        assert!(cache.remove_guild("333").is_some());
        assert!(cache.guild("333").is_none());
    }

    #[test]
    fn guilds_list_returns_all() {
        let cache = Cache::new();
        cache.upsert_guild(make_guild("1", "A"));
        cache.upsert_guild(make_guild("2", "B"));
        assert_eq!(cache.guild_count(), 2);
        assert_eq!(cache.guilds().len(), 2);
    }

    fn make_user(id: &str, name: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "username": name,
            "discriminator": "0000",
            "avatar": null
        }))
        .unwrap()
    }

    #[test]
    fn upsert_and_get_user() {
        let cache = Cache::new();
        cache.upsert_user(make_user("u1", "Alice"));
        let u = cache.user("u1").expect("user should be cached");
        assert_eq!(u.username, "Alice");
    }

    #[test]
    fn update_user_replaces_entry() {
        let cache = Cache::new();
        cache.upsert_user(make_user("u2", "Bob"));
        cache.upsert_user(make_user("u2", "Bobby"));
        assert_eq!(cache.user("u2").unwrap().username, "Bobby");
        assert_eq!(cache.user_count(), 1);
    }

    fn make_msg(channel_id: &str, message_id: &str, content: &str) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": message_id,
            "channel_id": channel_id,
            "author": { "id": "1", "username": "u", "discriminator": "0", "avatar": null },
            "content": content,
            "timestamp": "2024-01-01T00:00:00Z",
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
            "type": 0
        }))
        .unwrap()
    }

    #[test]
    fn message_cache_stores_and_retrieves() {
        let cache = Cache::new();
        cache.upsert_message(make_msg("ch1", "m1", "hello"));
        let msg = cache.message("ch1", "m1").expect("message should be cached");
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn message_cache_lru_eviction() {
        // Ring of size 3 — 4th push evicts oldest
        let cache = Cache::with_max_messages(3);
        cache.upsert_message(make_msg("ch2", "1", "first"));
        cache.upsert_message(make_msg("ch2", "2", "second"));
        cache.upsert_message(make_msg("ch2", "3", "third"));
        cache.upsert_message(make_msg("ch2", "4", "fourth"));
        assert!(cache.message("ch2", "1").is_none(), "oldest should be evicted");
        assert!(cache.message("ch2", "4").is_some());
        assert_eq!(cache.channel_messages("ch2").len(), 3);
    }

    #[test]
    fn message_cache_delete() {
        let cache = Cache::new();
        cache.upsert_message(make_msg("ch3", "m10", "to delete"));
        assert!(cache.remove_message("ch3", "m10").is_some());
        assert!(cache.message("ch3", "m10").is_none());
    }

    #[test]
    fn message_cache_disabled_when_max_zero() {
        let cache = Cache::with_max_messages(0);
        cache.upsert_message(make_msg("ch4", "m1", "ignored"));
        assert_eq!(cache.message_count(), 0);
    }

    #[test]
    fn settings_cache_guilds_false_skips_upsert() {
        let cache = Cache::with_settings(CacheSettings { cache_guilds: false, ..CacheSettings::default() });
        cache.upsert_guild(make_guild("g1", "Ignored"));
        assert!(cache.guild("g1").is_none());
    }

    #[test]
    fn settings_cache_users_false_skips_upsert() {
        let cache = Cache::with_settings(CacheSettings { cache_users: false, ..CacheSettings::default() });
        cache.upsert_user(make_user("u99", "Ghost"));
        assert!(cache.user("u99").is_none());
    }

    #[test]
    fn settings_cache_messages_false_skips_upsert() {
        let cache = Cache::with_settings(CacheSettings { cache_messages: false, ..CacheSettings::default() });
        cache.upsert_message(make_msg("ch5", "m1", "ignored"));
        assert_eq!(cache.message_count(), 0);
    }

    #[test]
    fn settings_accessor_returns_config() {
        let settings = CacheSettings { max_messages: 42, cache_guilds: false, ..CacheSettings::default() };
        let cache = Cache::with_settings(settings.clone());
        assert_eq!(cache.settings().max_messages, 42);
        assert!(!cache.settings().cache_guilds);
    }

    #[test]
    fn ttl_expired_guild_returns_none_on_access() {
        // TTL of 0 ns — everything expires immediately.
        let cache = Cache::with_settings(CacheSettings { time_to_live: Some(Duration::from_nanos(0)), ..CacheSettings::default() });
        cache.upsert_guild(make_guild("ttl1", "Expiring"));
        // Sleep 1 ms to guarantee elapsed > 0 ns
        std::thread::sleep(Duration::from_millis(1));
        assert!(cache.guild("ttl1").is_none(), "entry should have expired");
    }

    #[test]
    fn ttl_expired_user_returns_none_on_access() {
        let cache = Cache::with_settings(CacheSettings { time_to_live: Some(Duration::from_nanos(0)), ..CacheSettings::default() });
        cache.upsert_user(make_user("uttl1", "Expiring"));
        std::thread::sleep(Duration::from_millis(1));
        assert!(cache.user("uttl1").is_none(), "user entry should have expired");
    }

    #[test]
    fn no_ttl_entries_stay_indefinitely() {
        let cache = Cache::with_settings(CacheSettings { time_to_live: None, ..CacheSettings::default() });
        cache.upsert_guild(make_guild("perm1", "Permanent"));
        std::thread::sleep(Duration::from_millis(1));
        assert!(cache.guild("perm1").is_some(), "entry without TTL should persist");
    }
}
