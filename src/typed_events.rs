//! Typed event definitions and handlers for Discord gateway events
//!
//! This module provides type-safe event handling by defining strongly-typed
//! event structs that are automatically deserialized from gateway dispatch
//! events.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{Channel, Guild, Member, Message, Relationship, Role, User};

// ==================== Typed Event Structs ====================

/// READY event data - sent when the gateway connection is established
#[derive(Debug, Clone, Deserialize)]
pub struct ReadyEvent {
    /// Gateway protocol version
    pub v: u8,
    /// Current user data
    pub user: User,
    /// List of relationships (friends, blocked, pending)
    #[serde(default)]
    pub relationships: Vec<Relationship>,
    /// Session ID for resuming
    pub session_id: String,
    /// URL for resuming the gateway connection
    #[serde(default)]
    pub resume_gateway_url: Option<String>,
    /// Private channels (DMs)
    #[serde(default)]
    pub private_channels: Vec<Value>,
    /// Guild data
    #[serde(default)]
    pub guilds: Vec<Value>,
}

/// RESUMED event data — sent after a successful session resume.
///
/// Discord sends an empty `d` for this event; the struct exists so handlers
/// can pattern-match on `TypedEvent::Resumed`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ResumedEvent {}

/// MESSAGE_CREATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct MessageCreateEvent {
    #[serde(flatten)]
    pub message: Message,
}

/// MESSAGE_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct MessageUpdateEvent {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub author: Option<User>,
    #[serde(default)]
    pub embeds: Vec<Value>,
    #[serde(default)]
    pub attachments: Vec<Value>,
}

/// MESSAGE_DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeleteEvent {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// TYPING_START event data
#[derive(Debug, Clone, Deserialize)]
pub struct TypingStartEvent {
    pub user_id: String,
    pub channel_id: String,
    pub timestamp: u64,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub member: Option<Member>,
}

/// RELATIONSHIP_ADD event data
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipAddEvent {
    #[serde(flatten)]
    pub relationship: Relationship,
}

/// RELATIONSHIP_REMOVE event data
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipRemoveEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub relationship_type: u8,
    #[serde(default)]
    pub nickname: Option<String>,
}

/// PRESENCE_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceUpdateEvent {
    pub user: PresenceUser,
    pub status: String,
    #[serde(default)]
    pub client_status: Option<ClientStatus>,
    #[serde(default)]
    pub activities: Vec<Activity>,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// Minimal user data in presence updates
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceUser {
    pub id: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
}

/// Client status per platform
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClientStatus {
    #[serde(default)]
    pub desktop: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
    #[serde(default)]
    pub web: Option<String>,
}

/// User activity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Activity {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: u8,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// GUILD_MEMBER_ADD event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildMemberAddEvent {
    pub guild_id: String,
    #[serde(flatten)]
    pub member: Member,
}

/// GUILD_MEMBER_REMOVE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildMemberRemoveEvent {
    pub guild_id: String,
    pub user: User,
}

/// GUILD_MEMBER_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildMemberUpdateEvent {
    pub guild_id: String,
    pub user: User,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub premium_since: Option<String>,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub communication_disabled_until: Option<String>,
}

/// MESSAGE_REACTION_ADD event data
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReactionAddEvent {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub member: Option<Member>,
    pub emoji: ReactionEmoji,
    #[serde(default)]
    pub burst: bool,
}

/// MESSAGE_REACTION_REMOVE event data
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReactionRemoveEvent {
    pub user_id: String,
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub emoji: ReactionEmoji,
}

/// Emoji data in reactions
#[derive(Debug, Clone, Deserialize)]
pub struct ReactionEmoji {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub animated: bool,
}

/// VOICE_STATE_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceStateUpdateEvent {
    pub user_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    pub session_id: String,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub self_deaf: bool,
    #[serde(default)]
    pub self_mute: bool,
    #[serde(default)]
    pub self_video: bool,
    #[serde(default)]
    pub suppress: bool,
    #[serde(default)]
    pub member: Option<Member>,
}

/// CHANNEL_CREATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelCreateEvent {
    #[serde(flatten)]
    pub channel: Channel,
}

/// CHANNEL_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelUpdateEvent {
    #[serde(flatten)]
    pub channel: Channel,
}

/// CHANNEL_DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelDeleteEvent {
    #[serde(flatten)]
    pub channel: Channel,
}

/// GUILD_CREATE event data - sent when joining a guild or when guild becomes
/// available
#[derive(Debug, Clone, Deserialize)]
pub struct GuildCreateEvent {
    #[serde(flatten)]
    pub guild: Guild,
}

/// GUILD_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildUpdateEvent {
    #[serde(flatten)]
    pub guild: Guild,
}

/// GUILD_DELETE event data - sent when leaving/kicked from guild or guild
/// becomes unavailable
#[derive(Debug, Clone, Deserialize)]
pub struct GuildDeleteEvent {
    pub id: String,
    #[serde(default)]
    pub unavailable: bool,
}

/// USER_UPDATE event data - sent when current user's properties change
#[derive(Debug, Clone, Deserialize)]
pub struct UserUpdateEvent {
    #[serde(flatten)]
    pub user: User,
}

/// INTERACTION_CREATE event data - sent when user interacts with application
/// commands
#[derive(Debug, Clone, Deserialize)]
pub struct InteractionCreateEvent {
    pub id: String,
    pub application_id: String,
    #[serde(rename = "type")]
    pub interaction_type: u8,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub member: Option<Member>,
    #[serde(default)]
    pub user: Option<User>,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub version: u8,
    #[serde(default)]
    pub message: Option<Message>,
}

/// MESSAGE_REACTION_REMOVE_ALL event data — all reactions removed from a
/// message
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReactionRemoveAllEvent {
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// MESSAGE_REACTION_REMOVE_EMOJI event data — all reactions of a specific emoji
/// removed
#[derive(Debug, Clone, Deserialize)]
pub struct MessageReactionRemoveEmojiEvent {
    pub channel_id: String,
    pub message_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub emoji: ReactionEmoji,
}

/// MESSAGE_DELETE_BULK event data — multiple messages deleted at once
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeleteBulkEvent {
    pub ids: Vec<String>,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

/// GUILD_ROLE_CREATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildRoleCreateEvent {
    pub guild_id: String,
    pub role: Role,
}

/// GUILD_ROLE_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildRoleUpdateEvent {
    pub guild_id: String,
    pub role: Role,
}

/// GUILD_ROLE_DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildRoleDeleteEvent {
    pub guild_id: String,
    pub role_id: String,
}

/// GUILD_BAN_ADD event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildBanAddEvent {
    pub guild_id: String,
    pub user: User,
}

/// GUILD_BAN_REMOVE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildBanRemoveEvent {
    pub guild_id: String,
    pub user: User,
}

/// THREAD_CREATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadCreateEvent {
    #[serde(flatten)]
    pub channel: Channel,
}

/// THREAD_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadUpdateEvent {
    #[serde(flatten)]
    pub channel: Channel,
}

/// THREAD_DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadDeleteEvent {
    pub id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(rename = "type", default)]
    pub channel_type: u8,
}

/// THREAD_LIST_SYNC event data — sent when gaining access to a channel with
/// active threads
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadListSyncEvent {
    pub guild_id: String,
    #[serde(default)]
    pub channel_ids: Vec<String>,
    #[serde(default)]
    pub threads: Vec<Channel>,
    #[serde(default)]
    pub members: Vec<Value>,
}

/// THREAD_MEMBER_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadMemberUpdateEvent {
    pub id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub join_timestamp: Option<String>,
    #[serde(default)]
    pub flags: u64,
}

/// VOICE_SERVER_UPDATE event data — provides voice connection credentials
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceServerUpdateEvent {
    pub token: String,
    pub guild_id: String,
    /// Voice server WebSocket endpoint (may be null during voice region
    /// migration)
    pub endpoint: Option<String>,
}

/// WEBHOOK_UPDATE event data — fired when webhooks are created/modified/deleted
/// in a channel
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookUpdateEvent {
    pub guild_id: String,
    pub channel_id: String,
}

/// GUILD_AUDIT_LOG_ENTRY_CREATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildAuditLogEntryCreateEvent {
    pub guild_id: String,
    /// Action type (see Discord docs for numeric values)
    pub action_type: u32,
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub changes: Vec<serde_json::Value>,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// AUTO_MODERATION_RULE_CREATE / UPDATE / DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct AutoModerationRuleEvent {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub creator_id: String,
    pub event_type: u8,
    pub trigger_type: u8,
    pub enabled: bool,
    #[serde(default)]
    pub exempt_roles: Vec<String>,
    #[serde(default)]
    pub exempt_channels: Vec<String>,
}

/// AUTO_MODERATION_ACTION_EXECUTION event data
#[derive(Debug, Clone, Deserialize)]
pub struct AutoModerationActionExecutionEvent {
    pub guild_id: String,
    #[serde(default)]
    pub channel_id: Option<String>,
    pub user_id: String,
    pub rule_id: String,
    pub rule_trigger_type: u8,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub matched_keyword: Option<String>,
    #[serde(default)]
    pub matched_content: Option<String>,
}

/// STAGE_INSTANCE_CREATE / UPDATE / DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct StageInstanceEvent {
    pub id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub topic: String,
    /// Privacy level: 1=PUBLIC, 2=GUILD_ONLY
    pub privacy_level: u8,
    #[serde(default)]
    pub discoverable_disabled: bool,
    #[serde(default)]
    pub guild_scheduled_event_id: Option<String>,
}

/// GUILD_SCHEDULED_EVENT_CREATE / UPDATE / DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildScheduledEventEvent {
    pub id: String,
    pub guild_id: String,
    #[serde(default)]
    pub channel_id: Option<String>,
    #[serde(default)]
    pub creator_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub scheduled_start_time: String,
    #[serde(default)]
    pub scheduled_end_time: Option<String>,
    pub privacy_level: u8,
    pub status: u8,
    pub entity_type: u8,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub user_count: Option<u32>,
    #[serde(default)]
    pub image: Option<String>,
}

/// GUILD_SCHEDULED_EVENT_USER_ADD / USER_REMOVE event data
#[derive(Debug, Clone, Deserialize)]
pub struct GuildScheduledEventUserEvent {
    pub guild_scheduled_event_id: String,
    pub user_id: String,
    pub guild_id: String,
}

/// SOUNDBOARD_SOUND_CREATE / UPDATE / DELETE event data
#[derive(Debug, Clone, Deserialize)]
pub struct SoundboardSoundEvent {
    pub sound_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub name: String,
    pub volume: f64,
    #[serde(default)]
    pub emoji_id: Option<String>,
    #[serde(default)]
    pub emoji_name: Option<String>,
    #[serde(default)]
    pub available: bool,
}

/// CHANNEL_PINS_UPDATE event data
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelPinsUpdateEvent {
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    #[serde(default)]
    pub last_pin_timestamp: Option<String>,
}

// ==================== Typed Event Enum ====================

/// Strongly-typed Discord gateway events
#[derive(Debug, Clone)]
pub enum TypedEvent {
    /// Gateway connected and ready
    Ready(ReadyEvent),
    /// Session successfully resumed (after RESUME opcode acknowledged)
    Resumed(ResumedEvent),
    /// A message was created
    MessageCreate(Box<MessageCreateEvent>),
    /// A message was updated
    MessageUpdate(MessageUpdateEvent),
    /// A message was deleted
    MessageDelete(MessageDeleteEvent),
    /// User started typing
    TypingStart(TypingStartEvent),
    /// Relationship added (friend request, etc)
    RelationshipAdd(RelationshipAddEvent),
    /// Relationship removed
    RelationshipRemove(RelationshipRemoveEvent),
    /// User presence changed
    PresenceUpdate(PresenceUpdateEvent),
    /// Member joined a guild
    GuildMemberAdd(GuildMemberAddEvent),
    /// Member left a guild
    GuildMemberRemove(GuildMemberRemoveEvent),
    /// Member updated in a guild
    GuildMemberUpdate(GuildMemberUpdateEvent),
    /// Reaction added to a message
    MessageReactionAdd(MessageReactionAddEvent),
    /// Reaction removed from a message
    MessageReactionRemove(MessageReactionRemoveEvent),
    /// All reactions removed from a message
    MessageReactionRemoveAll(MessageReactionRemoveAllEvent),
    /// All reactions of a specific emoji removed from a message
    MessageReactionRemoveEmoji(MessageReactionRemoveEmojiEvent),
    /// Multiple messages deleted at once
    MessageDeleteBulk(MessageDeleteBulkEvent),
    /// Voice state changed
    VoiceStateUpdate(VoiceStateUpdateEvent),
    /// Channel created
    ChannelCreate(ChannelCreateEvent),
    /// Channel updated
    ChannelUpdate(ChannelUpdateEvent),
    /// Channel deleted
    ChannelDelete(ChannelDeleteEvent),
    /// Channel pins updated
    ChannelPinsUpdate(ChannelPinsUpdateEvent),
    /// Guild joined or became available
    GuildCreate(GuildCreateEvent),
    /// Guild updated
    GuildUpdate(GuildUpdateEvent),
    /// Guild left or became unavailable
    GuildDelete(GuildDeleteEvent),
    /// Guild role created
    GuildRoleCreate(GuildRoleCreateEvent),
    /// Guild role updated
    GuildRoleUpdate(GuildRoleUpdateEvent),
    /// Guild role deleted
    GuildRoleDelete(GuildRoleDeleteEvent),
    /// User banned from guild
    GuildBanAdd(GuildBanAddEvent),
    /// User unbanned from guild
    GuildBanRemove(GuildBanRemoveEvent),
    /// Current user updated
    UserUpdate(UserUpdateEvent),
    /// Interaction received
    InteractionCreate(Box<InteractionCreateEvent>),
    /// Thread created
    ThreadCreate(ThreadCreateEvent),
    /// Thread updated
    ThreadUpdate(ThreadUpdateEvent),
    /// Thread deleted
    ThreadDelete(ThreadDeleteEvent),
    /// Thread list synced
    ThreadListSync(ThreadListSyncEvent),
    /// Thread member updated
    ThreadMemberUpdate(ThreadMemberUpdateEvent),
    /// Voice server update (provides voice connection credentials)
    VoiceServerUpdate(VoiceServerUpdateEvent),
    /// Webhooks created/modified/deleted in a channel
    WebhookUpdate(WebhookUpdateEvent),
    /// Audit log entry created
    GuildAuditLogEntryCreate(GuildAuditLogEntryCreateEvent),
    /// Auto-moderation rule created
    AutoModerationRuleCreate(AutoModerationRuleEvent),
    /// Auto-moderation rule updated
    AutoModerationRuleUpdate(AutoModerationRuleEvent),
    /// Auto-moderation rule deleted
    AutoModerationRuleDelete(AutoModerationRuleEvent),
    /// Auto-moderation action executed
    AutoModerationActionExecution(AutoModerationActionExecutionEvent),
    /// Stage instance created
    StageInstanceCreate(StageInstanceEvent),
    /// Stage instance updated
    StageInstanceUpdate(StageInstanceEvent),
    /// Stage instance deleted
    StageInstanceDelete(StageInstanceEvent),
    /// Guild scheduled event created
    GuildScheduledEventCreate(GuildScheduledEventEvent),
    /// Guild scheduled event updated
    GuildScheduledEventUpdate(GuildScheduledEventEvent),
    /// Guild scheduled event deleted
    GuildScheduledEventDelete(GuildScheduledEventEvent),
    /// User subscribed to guild scheduled event
    GuildScheduledEventUserAdd(GuildScheduledEventUserEvent),
    /// User unsubscribed from guild scheduled event
    GuildScheduledEventUserRemove(GuildScheduledEventUserEvent),
    /// Soundboard sound created
    SoundboardSoundCreate(SoundboardSoundEvent),
    /// Soundboard sound updated
    SoundboardSoundUpdate(SoundboardSoundEvent),
    /// Soundboard sound deleted
    SoundboardSoundDelete(SoundboardSoundEvent),
    /// Unknown/unhandled event with raw data
    Unknown { event_type: String, data: Value },
}

impl TypedEvent {
    /// Parse a raw dispatch event into a typed event
    pub fn from_raw(event_type: &str, data: Value) -> Self {
        macro_rules! parse {
            ($variant:ident) => {
                serde_json::from_value(data.clone()).map(TypedEvent::$variant).unwrap_or_else(|_| TypedEvent::Unknown { event_type: event_type.to_string(), data })
            };
        }
        match event_type {
            "READY" => parse!(Ready),
            "RESUMED" => TypedEvent::Resumed(ResumedEvent {}),
            "MESSAGE_CREATE" => serde_json::from_value(data.clone()).map(|e| TypedEvent::MessageCreate(Box::new(e))).unwrap_or_else(|_| TypedEvent::Unknown { event_type: event_type.to_string(), data }),
            "MESSAGE_UPDATE" => parse!(MessageUpdate),
            "MESSAGE_DELETE" => parse!(MessageDelete),
            "MESSAGE_DELETE_BULK" => parse!(MessageDeleteBulk),
            "TYPING_START" => parse!(TypingStart),
            "RELATIONSHIP_ADD" => parse!(RelationshipAdd),
            "RELATIONSHIP_REMOVE" => parse!(RelationshipRemove),
            "PRESENCE_UPDATE" => parse!(PresenceUpdate),
            "GUILD_MEMBER_ADD" => parse!(GuildMemberAdd),
            "GUILD_MEMBER_REMOVE" => parse!(GuildMemberRemove),
            "GUILD_MEMBER_UPDATE" => parse!(GuildMemberUpdate),
            "MESSAGE_REACTION_ADD" => parse!(MessageReactionAdd),
            "MESSAGE_REACTION_REMOVE" => parse!(MessageReactionRemove),
            "MESSAGE_REACTION_REMOVE_ALL" => parse!(MessageReactionRemoveAll),
            "MESSAGE_REACTION_REMOVE_EMOJI" => parse!(MessageReactionRemoveEmoji),
            "VOICE_STATE_UPDATE" => parse!(VoiceStateUpdate),
            "CHANNEL_CREATE" => parse!(ChannelCreate),
            "CHANNEL_UPDATE" => parse!(ChannelUpdate),
            "CHANNEL_DELETE" => parse!(ChannelDelete),
            "CHANNEL_PINS_UPDATE" => parse!(ChannelPinsUpdate),
            "GUILD_CREATE" => parse!(GuildCreate),
            "GUILD_UPDATE" => parse!(GuildUpdate),
            "GUILD_DELETE" => parse!(GuildDelete),
            "GUILD_ROLE_CREATE" => parse!(GuildRoleCreate),
            "GUILD_ROLE_UPDATE" => parse!(GuildRoleUpdate),
            "GUILD_ROLE_DELETE" => parse!(GuildRoleDelete),
            "GUILD_BAN_ADD" => parse!(GuildBanAdd),
            "GUILD_BAN_REMOVE" => parse!(GuildBanRemove),
            "USER_UPDATE" => parse!(UserUpdate),
            "INTERACTION_CREATE" => serde_json::from_value(data.clone()).map(|e| TypedEvent::InteractionCreate(Box::new(e))).unwrap_or_else(|_| TypedEvent::Unknown { event_type: event_type.to_string(), data }),
            "THREAD_CREATE" => parse!(ThreadCreate),
            "THREAD_UPDATE" => parse!(ThreadUpdate),
            "THREAD_DELETE" => parse!(ThreadDelete),
            "THREAD_LIST_SYNC" => parse!(ThreadListSync),
            "THREAD_MEMBER_UPDATE" => parse!(ThreadMemberUpdate),
            "VOICE_SERVER_UPDATE" => parse!(VoiceServerUpdate),
            "WEBHOOKS_UPDATE" => parse!(WebhookUpdate),
            "GUILD_AUDIT_LOG_ENTRY_CREATE" => parse!(GuildAuditLogEntryCreate),
            "AUTO_MODERATION_RULE_CREATE" => parse!(AutoModerationRuleCreate),
            "AUTO_MODERATION_RULE_UPDATE" => parse!(AutoModerationRuleUpdate),
            "AUTO_MODERATION_RULE_DELETE" => parse!(AutoModerationRuleDelete),
            "AUTO_MODERATION_ACTION_EXECUTION" => parse!(AutoModerationActionExecution),
            "STAGE_INSTANCE_CREATE" => parse!(StageInstanceCreate),
            "STAGE_INSTANCE_UPDATE" => parse!(StageInstanceUpdate),
            "STAGE_INSTANCE_DELETE" => parse!(StageInstanceDelete),
            "GUILD_SCHEDULED_EVENT_CREATE" => parse!(GuildScheduledEventCreate),
            "GUILD_SCHEDULED_EVENT_UPDATE" => parse!(GuildScheduledEventUpdate),
            "GUILD_SCHEDULED_EVENT_DELETE" => parse!(GuildScheduledEventDelete),
            "GUILD_SCHEDULED_EVENT_USER_ADD" => parse!(GuildScheduledEventUserAdd),
            "GUILD_SCHEDULED_EVENT_USER_REMOVE" => parse!(GuildScheduledEventUserRemove),
            "SOUNDBOARD_SOUND_CREATE" => parse!(SoundboardSoundCreate),
            "SOUNDBOARD_SOUND_UPDATE" => parse!(SoundboardSoundUpdate),
            "SOUNDBOARD_SOUND_DELETE" => parse!(SoundboardSoundDelete),
            _ => TypedEvent::Unknown { event_type: event_type.to_string(), data },
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &str {
        match self {
            TypedEvent::Ready(_) => "READY",
            TypedEvent::Resumed(_) => "RESUMED",
            TypedEvent::MessageCreate(_) => "MESSAGE_CREATE",
            TypedEvent::MessageUpdate(_) => "MESSAGE_UPDATE",
            TypedEvent::MessageDelete(_) => "MESSAGE_DELETE",
            TypedEvent::MessageDeleteBulk(_) => "MESSAGE_DELETE_BULK",
            TypedEvent::TypingStart(_) => "TYPING_START",
            TypedEvent::RelationshipAdd(_) => "RELATIONSHIP_ADD",
            TypedEvent::RelationshipRemove(_) => "RELATIONSHIP_REMOVE",
            TypedEvent::PresenceUpdate(_) => "PRESENCE_UPDATE",
            TypedEvent::GuildMemberAdd(_) => "GUILD_MEMBER_ADD",
            TypedEvent::GuildMemberRemove(_) => "GUILD_MEMBER_REMOVE",
            TypedEvent::GuildMemberUpdate(_) => "GUILD_MEMBER_UPDATE",
            TypedEvent::MessageReactionAdd(_) => "MESSAGE_REACTION_ADD",
            TypedEvent::MessageReactionRemove(_) => "MESSAGE_REACTION_REMOVE",
            TypedEvent::MessageReactionRemoveAll(_) => "MESSAGE_REACTION_REMOVE_ALL",
            TypedEvent::MessageReactionRemoveEmoji(_) => "MESSAGE_REACTION_REMOVE_EMOJI",
            TypedEvent::VoiceStateUpdate(_) => "VOICE_STATE_UPDATE",
            TypedEvent::ChannelCreate(_) => "CHANNEL_CREATE",
            TypedEvent::ChannelUpdate(_) => "CHANNEL_UPDATE",
            TypedEvent::ChannelDelete(_) => "CHANNEL_DELETE",
            TypedEvent::ChannelPinsUpdate(_) => "CHANNEL_PINS_UPDATE",
            TypedEvent::GuildCreate(_) => "GUILD_CREATE",
            TypedEvent::GuildUpdate(_) => "GUILD_UPDATE",
            TypedEvent::GuildDelete(_) => "GUILD_DELETE",
            TypedEvent::GuildRoleCreate(_) => "GUILD_ROLE_CREATE",
            TypedEvent::GuildRoleUpdate(_) => "GUILD_ROLE_UPDATE",
            TypedEvent::GuildRoleDelete(_) => "GUILD_ROLE_DELETE",
            TypedEvent::GuildBanAdd(_) => "GUILD_BAN_ADD",
            TypedEvent::GuildBanRemove(_) => "GUILD_BAN_REMOVE",
            TypedEvent::UserUpdate(_) => "USER_UPDATE",
            TypedEvent::InteractionCreate(_) => "INTERACTION_CREATE",
            TypedEvent::ThreadCreate(_) => "THREAD_CREATE",
            TypedEvent::ThreadUpdate(_) => "THREAD_UPDATE",
            TypedEvent::ThreadDelete(_) => "THREAD_DELETE",
            TypedEvent::ThreadListSync(_) => "THREAD_LIST_SYNC",
            TypedEvent::ThreadMemberUpdate(_) => "THREAD_MEMBER_UPDATE",
            TypedEvent::VoiceServerUpdate(_) => "VOICE_SERVER_UPDATE",
            TypedEvent::WebhookUpdate(_) => "WEBHOOKS_UPDATE",
            TypedEvent::GuildAuditLogEntryCreate(_) => "GUILD_AUDIT_LOG_ENTRY_CREATE",
            TypedEvent::AutoModerationRuleCreate(_) => "AUTO_MODERATION_RULE_CREATE",
            TypedEvent::AutoModerationRuleUpdate(_) => "AUTO_MODERATION_RULE_UPDATE",
            TypedEvent::AutoModerationRuleDelete(_) => "AUTO_MODERATION_RULE_DELETE",
            TypedEvent::AutoModerationActionExecution(_) => "AUTO_MODERATION_ACTION_EXECUTION",
            TypedEvent::StageInstanceCreate(_) => "STAGE_INSTANCE_CREATE",
            TypedEvent::StageInstanceUpdate(_) => "STAGE_INSTANCE_UPDATE",
            TypedEvent::StageInstanceDelete(_) => "STAGE_INSTANCE_DELETE",
            TypedEvent::GuildScheduledEventCreate(_) => "GUILD_SCHEDULED_EVENT_CREATE",
            TypedEvent::GuildScheduledEventUpdate(_) => "GUILD_SCHEDULED_EVENT_UPDATE",
            TypedEvent::GuildScheduledEventDelete(_) => "GUILD_SCHEDULED_EVENT_DELETE",
            TypedEvent::GuildScheduledEventUserAdd(_) => "GUILD_SCHEDULED_EVENT_USER_ADD",
            TypedEvent::GuildScheduledEventUserRemove(_) => "GUILD_SCHEDULED_EVENT_USER_REMOVE",
            TypedEvent::SoundboardSoundCreate(_) => "SOUNDBOARD_SOUND_CREATE",
            TypedEvent::SoundboardSoundUpdate(_) => "SOUNDBOARD_SOUND_UPDATE",
            TypedEvent::SoundboardSoundDelete(_) => "SOUNDBOARD_SOUND_DELETE",
            TypedEvent::Unknown { event_type, .. } => event_type,
        }
    }
}
