//! Discord enums matching the JavaScript constants

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// User online status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    #[default]
    Online,
    Idle,
    #[serde(rename = "dnd")]
    DoNotDisturb,
    Invisible,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Online => "online",
            UserStatus::Idle => "idle",
            UserStatus::DoNotDisturb => "dnd",
            UserStatus::Invisible => "invisible",
        }
    }
}

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum MessageType {
    #[default]
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelNameChange = 4,
    ChannelIconChange = 5,
    ChannelPinnedMessage = 6,
    UserJoin = 7,
    GuildBoost = 8,
    GuildBoostTier1 = 9,
    GuildBoostTier2 = 10,
    GuildBoostTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryRequalified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ChatInputCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
    AutoModerationAction = 24,
    RoleSubscriptionPurchase = 25,
    InteractionPremiumUpsell = 26,
    StageStart = 27,
    StageEnd = 28,
    StageSpeaker = 29,
    StageTopic = 31,
    GuildApplicationPremiumSubscription = 32,
}

/// Relationship type between users
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum RelationshipType {
    #[default]
    None = 0,
    Friend = 1,
    Blocked = 2,
    PendingIncoming = 3,
    PendingOutgoing = 4,
    Implicit = 5,
}

/// Audit log action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActionType {
    GuildUpdate = 1,
    ChannelCreate = 10,
    ChannelUpdate = 11,
    ChannelDelete = 12,
    ChannelOverwriteCreate = 13,
    ChannelOverwriteUpdate = 14,
    ChannelOverwriteDelete = 15,
    MemberKick = 20,
    MemberPrune = 21,
    MemberBanAdd = 22,
    MemberBanRemove = 23,
    MemberUpdate = 24,
    MemberRoleUpdate = 25,
    MemberMove = 26,
    MemberDisconnect = 27,
    BotAdd = 28,
    RoleCreate = 30,
    RoleUpdate = 31,
    RoleDelete = 32,
    InviteCreate = 40,
    InviteUpdate = 41,
    InviteDelete = 42,
    WebhookCreate = 50,
    WebhookUpdate = 51,
    WebhookDelete = 52,
    EmojiCreate = 60,
    EmojiUpdate = 61,
    EmojiDelete = 62,
    MessageDelete = 72,
    MessageBulkDelete = 73,
    MessagePin = 74,
    MessageUnpin = 75,
    IntegrationCreate = 80,
    IntegrationUpdate = 81,
    IntegrationDelete = 82,
    StageInstanceCreate = 83,
    StageInstanceUpdate = 84,
    StageInstanceDelete = 85,
    StickerCreate = 90,
    StickerUpdate = 91,
    StickerDelete = 92,
    GuildScheduledEventCreate = 100,
    GuildScheduledEventUpdate = 101,
    GuildScheduledEventDelete = 102,
    ThreadCreate = 110,
    ThreadUpdate = 111,
    ThreadDelete = 112,
    ApplicationCommandPermissionUpdate = 121,
    AutoModerationRuleCreate = 140,
    AutoModerationRuleUpdate = 141,
    AutoModerationRuleDelete = 142,
    AutoModerationBlockMessage = 143,
    AutoModerationFlagToChannel = 144,
    AutoModerationUserCommunicationDisabled = 145,
    CreatorMonetizationRequestCreated = 150,
    CreatorMonetizationTermsAccepted = 151,
    OnboardingPromptCreate = 163,
    OnboardingPromptUpdate = 164,
    OnboardingPromptDelete = 165,
    OnboardingCreate = 166,
    OnboardingUpdate = 167,
    HomeSettingsCreate = 190,
    HomeSettingsUpdate = 191,
    VoiceChannelStatusUpdated = 192,
}

/// Channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum ChannelType {
    #[default]
    GuildText = 0,
    Dm = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
    GuildMedia = 16,
}

/// The current state of the gateway connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConnectionStage {
    /// Connected and ready
    Connected,
    /// Connecting to the gateway
    Connecting,
    /// Disconnected from the gateway
    #[default]
    Disconnected,
    /// Handshaking (Hello received)
    Handshake,
    /// Identifying with the gateway
    Identifying,
    /// Resuming a previous session
    Resuming,
}

impl ConnectionStage {
    /// Check if the stage is currently connecting (any state between
    /// Disconnected and Connected)
    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionStage::Connecting | ConnectionStage::Handshake | ConnectionStage::Identifying | ConnectionStage::Resuming)
    }
}

bitflags::bitflags! {
    /// Gateway intents to filter received events
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct GatewayIntents: u32 {
        const GUILDS = 1 << 0;
        const GUILD_MEMBERS = 1 << 1;
        const GUILD_MODERATION = 1 << 2;
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        const GUILD_INTEGRATIONS = 1 << 4;
        const GUILD_WEBHOOKS = 1 << 5;
        const GUILD_INVITES = 1 << 6;
        const GUILD_VOICE_STATES = 1 << 7;
        const GUILD_PRESENCES = 1 << 8;
        const GUILD_MESSAGES = 1 << 9;
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        const GUILD_MESSAGE_TYPING = 1 << 11;
        const DIRECT_MESSAGES = 1 << 12;
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        const DIRECT_MESSAGE_TYPING = 1 << 14;
        const MESSAGE_CONTENT = 1 << 15;
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        const AUTO_MODERATION_EXECUTION = 1 << 21;
        const GUILD_MESSAGE_POLLS = 1 << 24;
        const DIRECT_MESSAGE_POLLS = 1 << 25;
    }
}

impl Default for GatewayIntents {
    fn default() -> Self {
        // Default for self-bots (standard mask observed in previous implementations)
        Self::from_bits_truncate(16381)
    }
}

bitflags::bitflags! {
    /// Message flags (bitfield on the Message object)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct MessageFlags: u64 {
        /// Message was crossposted to followers
        const CROSSPOSTED = 1 << 0;
        /// Message is a crosspost from another channel
        const IS_CROSSPOST = 1 << 1;
        /// Embed links suppressed for this message
        const SUPPRESS_EMBEDS = 1 << 2;
        /// Source message for this crosspost was deleted
        const SOURCE_MESSAGE_DELETED = 1 << 3;
        /// Message is from the urgent system
        const URGENT = 1 << 4;
        /// Message has an associated thread
        const HAS_THREAD = 1 << 5;
        /// Message is ephemeral (only visible to the invoking user)
        const EPHEMERAL = 1 << 6;
        /// Message is an interaction response and the bot is still loading
        const LOADING = 1 << 7;
        /// Message failed to mention some roles in a thread
        const FAILED_GUILD_CHANNEL_FOLLOWUP = 1 << 8;
        /// Message will not trigger push / desktop notifications
        const SUPPRESS_NOTIFICATIONS = 1 << 12;
        /// Message is a voice message
        const IS_VOICE_MESSAGE = 1 << 13;
    }
}

bitflags::bitflags! {
    /// User public flags (badges and special attributes)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct UserPublicFlags: u64 {
        /// Discord Employee
        const STAFF = 1 << 0;
        /// Partnered Server Owner
        const PARTNER = 1 << 1;
        /// HypeSquad Events Member
        const HYPESQUAD = 1 << 2;
        /// Bug Hunter Level 1
        const BUG_HUNTER_LEVEL_1 = 1 << 3;
        /// House Bravery Member
        const HYPESQUAD_ONLINE_HOUSE_1 = 1 << 6;
        /// House Brilliance Member
        const HYPESQUAD_ONLINE_HOUSE_2 = 1 << 7;
        /// House Balance Member
        const HYPESQUAD_ONLINE_HOUSE_3 = 1 << 8;
        /// Early Nitro Supporter
        const PREMIUM_EARLY_SUPPORTER = 1 << 9;
        /// Team Pseudo-User
        const TEAM_PSEUDO_USER = 1 << 10;
        /// Bug Hunter Level 2
        const BUG_HUNTER_LEVEL_2 = 1 << 14;
        /// Verified Bot
        const VERIFIED_BOT = 1 << 16;
        /// Early Verified Bot Developer
        const VERIFIED_DEVELOPER = 1 << 17;
        /// Moderator Programs Alumni
        const CERTIFIED_MODERATOR = 1 << 18;
        /// Bot uses only HTTP interactions
        const BOT_HTTP_INTERACTIONS = 1 << 19;
        /// Active Developer
        const ACTIVE_DEVELOPER = 1 << 22;
    }
}

bitflags::bitflags! {
    /// Guild / channel permission flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct Permissions: u64 {
        const CREATE_INSTANT_INVITE = 1 << 0;
        const KICK_MEMBERS = 1 << 1;
        const BAN_MEMBERS = 1 << 2;
        const ADMINISTRATOR = 1 << 3;
        const MANAGE_CHANNELS = 1 << 4;
        const MANAGE_GUILD = 1 << 5;
        const ADD_REACTIONS = 1 << 6;
        const VIEW_AUDIT_LOG = 1 << 7;
        const PRIORITY_SPEAKER = 1 << 8;
        const STREAM = 1 << 9;
        const VIEW_CHANNEL = 1 << 10;
        const SEND_MESSAGES = 1 << 11;
        const SEND_TTS_MESSAGES = 1 << 12;
        const MANAGE_MESSAGES = 1 << 13;
        const EMBED_LINKS = 1 << 14;
        const ATTACH_FILES = 1 << 15;
        const READ_MESSAGE_HISTORY = 1 << 16;
        const MENTION_EVERYONE = 1 << 17;
        const USE_EXTERNAL_EMOJIS = 1 << 18;
        const VIEW_GUILD_INSIGHTS = 1 << 19;
        const CONNECT = 1 << 20;
        const SPEAK = 1 << 21;
        const MUTE_MEMBERS = 1 << 22;
        const DEAFEN_MEMBERS = 1 << 23;
        const MOVE_MEMBERS = 1 << 24;
        const USE_VAD = 1 << 25;
        const CHANGE_NICKNAME = 1 << 26;
        const MANAGE_NICKNAMES = 1 << 27;
        const MANAGE_ROLES = 1 << 28;
        const MANAGE_WEBHOOKS = 1 << 29;
        const MANAGE_GUILD_EXPRESSIONS = 1 << 30;
        const USE_APPLICATION_COMMANDS = 1 << 31;
        const REQUEST_TO_SPEAK = 1 << 32;
        const MANAGE_EVENTS = 1 << 33;
        const MANAGE_THREADS = 1 << 34;
        const CREATE_PUBLIC_THREADS = 1 << 35;
        const CREATE_PRIVATE_THREADS = 1 << 36;
        const USE_EXTERNAL_STICKERS = 1 << 37;
        const SEND_MESSAGES_IN_THREADS = 1 << 38;
        const USE_EMBEDDED_ACTIVITIES = 1 << 39;
        const MODERATE_MEMBERS = 1 << 40;
        const VIEW_CREATOR_MONETIZATION_ANALYTICS = 1 << 41;
        const USE_SOUNDBOARD = 1 << 42;
        const USE_EXTERNAL_SOUNDS = 1 << 45;
        const SEND_VOICE_MESSAGES = 1 << 46;
    }
}

impl Permissions {
    /// All permissions combined into one value.
    pub const fn preset_all() -> Self {
        Self::from_bits_truncate(u64::MAX)
    }

    /// No permissions (empty set).
    pub const fn preset_none() -> Self {
        Self::empty()
    }

    /// Permissions that a typical text-channel member needs:
    /// VIEW_CHANNEL, READ_MESSAGE_HISTORY, SEND_MESSAGES, EMBED_LINKS,
    /// ATTACH_FILES, ADD_REACTIONS, USE_EXTERNAL_EMOJIS,
    /// USE_APPLICATION_COMMANDS, SEND_MESSAGES_IN_THREADS.
    pub const fn preset_text() -> Self {
        Self::from_bits_truncate(Self::VIEW_CHANNEL.bits() | Self::READ_MESSAGE_HISTORY.bits() | Self::SEND_MESSAGES.bits() | Self::EMBED_LINKS.bits() | Self::ATTACH_FILES.bits() | Self::ADD_REACTIONS.bits() | Self::USE_EXTERNAL_EMOJIS.bits() | Self::USE_APPLICATION_COMMANDS.bits() | Self::SEND_MESSAGES_IN_THREADS.bits())
    }

    /// Permissions that a typical voice-channel member needs:
    /// VIEW_CHANNEL, CONNECT, SPEAK, STREAM, USE_VAD,
    /// USE_EMBEDDED_ACTIVITIES, SEND_VOICE_MESSAGES.
    pub const fn preset_voice() -> Self {
        Self::from_bits_truncate(Self::VIEW_CHANNEL.bits() | Self::CONNECT.bits() | Self::SPEAK.bits() | Self::STREAM.bits() | Self::USE_VAD.bits() | Self::USE_EMBEDDED_ACTIVITIES.bits() | Self::SEND_VOICE_MESSAGES.bits())
    }

    /// Full moderation permissions (non-administrator):
    /// KICK_MEMBERS, BAN_MEMBERS, MANAGE_MESSAGES, MANAGE_NICKNAMES,
    /// MODERATE_MEMBERS, VIEW_AUDIT_LOG, MUTE_MEMBERS, DEAFEN_MEMBERS,
    /// MOVE_MEMBERS.
    pub const fn preset_moderation() -> Self {
        Self::from_bits_truncate(Self::KICK_MEMBERS.bits() | Self::BAN_MEMBERS.bits() | Self::MANAGE_MESSAGES.bits() | Self::MANAGE_NICKNAMES.bits() | Self::MODERATE_MEMBERS.bits() | Self::VIEW_AUDIT_LOG.bits() | Self::MUTE_MEMBERS.bits() | Self::DEAFEN_MEMBERS.bits() | Self::MOVE_MEMBERS.bits())
    }

    /// Full administrator equivalent (all bits set via ADMINISTRATOR flag
    /// which implicitly grants everything).
    pub const fn preset_administrator() -> Self {
        Self::ADMINISTRATOR
    }

    /// Combined text + voice for a regular member.
    pub const fn preset_general_member() -> Self {
        Self::from_bits_truncate(Self::preset_text().bits() | Self::preset_voice().bits() | Self::CHANGE_NICKNAME.bits())
    }
}

/// Reconnection type to attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectType {
    /// Resume existing session
    Resume,
    /// Start a new session
    Reidentify,
}

/// Interaction callback type constants.
///
/// Mirrors serenity's `InteractionResponseType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InteractionResponseType {
    /// Respond to a PING (only for webhook-based interactions).
    Pong = 1,
    /// Send a normal channel message as the response.
    ChannelMessageWithSource = 4,
    /// Show a "thinking…" loading state; follow up later with a message.
    DeferredChannelMessageWithSource = 5,
    /// Acknowledge a component interaction without changing the message (for
    /// followups).
    DeferredUpdateMessage = 6,
    /// Edit the original component message in-place.
    UpdateMessage = 7,
    /// Display an autocomplete suggestions list.
    ApplicationCommandAutocompleteResult = 8,
    /// Show a modal dialog to the user.
    Modal = 9,
    /// Upsell premium in response to an interaction.
    PremiumRequired = 10,
}

macro_rules! impl_serde_as_u64 {
    ($($ty:ty),* $(,)?) => {
        $(
            impl serde::Serialize for $ty {
                fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                    s.serialize_u64(self.bits())
                }
            }
            impl<'de> serde::Deserialize<'de> for $ty {
                fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                    let bits = u64::deserialize(d)?;
                    Ok(Self::from_bits_truncate(bits))
                }
            }
        )*
    };
}

impl_serde_as_u64!(MessageFlags, UserPublicFlags, Permissions);
