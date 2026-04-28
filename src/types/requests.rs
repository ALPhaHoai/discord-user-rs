//! Common request structures for Discord API

use serde::Serialize;

use crate::types::message::{PollAnswer, PollMedia};

/// An empty request body for PUT/POST requests that don't need a payload.
/// This is a zero-sized type (ZST) to avoid allocations.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct EmptyRequest;

/// Static instance of EmptyRequest for reuse
pub static EMPTY_REQUEST: EmptyRequest = EmptyRequest;

/// Request body for POST /guilds/{id}/roles (create a role)
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRoleRequest {
    /// Role name (default: "new role")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Permission bitfield as a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    /// RGB color value (0 = default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    /// Whether to display members with this role separately in the sidebar
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    /// Whether this role should be mentionable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentionable: Option<bool>,
}

/// Request body for PATCH /guilds/{id}/roles/{role_id} (edit a role)
///
/// Identical shape to [`CreateRoleRequest`] — all fields optional.
pub type EditRoleRequest = CreateRoleRequest;

/// A file attachment to send with a message.
///
/// Discord's multipart upload uses `files[N]` form fields alongside a
/// `payload_json` field that carries the rest of the message body.
#[derive(Debug, Clone)]
pub struct CreateAttachment {
    /// File name shown in Discord (e.g. `"screenshot.png"`)
    pub filename: String,
    /// Raw file bytes
    pub data: Vec<u8>,
    /// MIME type (e.g. `"image/png"`). Defaults to
    /// `"application/octet-stream"`.
    pub mime_type: String,
    /// Optional description shown as alt-text on images
    pub description: Option<String>,
}

impl CreateAttachment {
    /// Create an attachment from raw bytes, inferring no MIME type.
    pub fn bytes(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self { filename: filename.into(), data, mime_type: "application/octet-stream".to_string(), description: None }
    }

    /// Create an attachment and set an explicit MIME type.
    pub fn with_mime(filename: impl Into<String>, data: Vec<u8>, mime_type: impl Into<String>) -> Self {
        Self { filename: filename.into(), data, mime_type: mime_type.into(), description: None }
    }

    /// Set alt-text description for this attachment (shown for images).
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Request body for POST /channels/{id}/threads or POST
/// /channels/{id}/messages/{msg}/threads
#[derive(Debug, Clone, Serialize)]
pub struct CreateThreadRequest {
    /// Thread name (1–100 characters)
    pub name: String,
    /// Auto-archive duration in minutes (60, 1440, 4320, 10080)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_archive_duration: Option<u32>,
    /// Slowmode rate-limit in seconds (0–21600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
    /// Channel type — only used when creating a thread not from a message.
    /// 11 = PUBLIC_THREAD, 12 = PRIVATE_THREAD
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<u8>,
    /// Whether the thread is invitable (private threads only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitable: Option<bool>,
}

impl CreateThreadRequest {
    /// Create a public thread.
    pub fn public(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            auto_archive_duration: None,
            rate_limit_per_user: None,
            channel_type: Some(11),
            invitable: None,
        }
    }

    /// Create a private thread.
    pub fn private(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            auto_archive_duration: None,
            rate_limit_per_user: None,
            channel_type: Some(12),
            invitable: None,
        }
    }

    /// Set auto-archive duration (minutes).
    pub fn auto_archive(mut self, minutes: u32) -> Self {
        self.auto_archive_duration = Some(minutes);
        self
    }
}

/// Request body for PATCH /channels/{id} when editing a thread
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditThreadRequest {
    /// New thread name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether the thread is archived
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    /// Auto-archive duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_archive_duration: Option<u32>,
    /// Whether the thread is locked (only moderators can unarchive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// Slowmode rate-limit in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
}

/// Request body for POST /channels/{id}/webhooks (create a webhook).
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// Webhook name (1–80 characters)
    pub name: String,
    /// Base64 avatar data URI (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
}

/// Request body for PATCH /webhooks/{id}[/{token}] (edit a webhook).
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditWebhookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Move webhook to a different channel (requires MANAGE_WEBHOOKS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}

/// Request body for POST /webhooks/{id}/{token} (execute a webhook).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExecuteWebhookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Override the webhook's default username for this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Override the webhook's default avatar URL for this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub tts: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub embeds: Vec<serde_json::Value>,
}

/// Request body for POST /guilds/{id}/emojis (create a guild emoji).
#[derive(Debug, Clone, Serialize)]
pub struct CreateEmojiRequest {
    /// Emoji name
    pub name: String,
    /// Base64-encoded image data URI: `"data:image/png;base64,..."`.
    /// PNG, JPG, or GIF (animated). Max 256 KB.
    pub image: String,
    /// Role IDs that can use this emoji (empty = all roles)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<String>,
}

/// Request body for PATCH /guilds/{id}/emojis/{emoji_id} (edit a guild emoji).
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditEmojiRequest {
    /// New emoji name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New role restriction list (empty vec = unrestrict)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

/// Request body for PATCH /guilds/{id}/stickers/{sticker_id} (edit a guild
/// sticker).
///
/// Sticker creation uses multipart form data — use
/// `DiscordHttpClient::post_multipart` with the sticker file as a `files[0]`
/// part and these fields as `payload_json`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditStickerRequest {
    /// New sticker name (2–30 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description (2–100 characters, or empty string to clear).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New autocomplete / search tags (max 200 characters, comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}

/// Request body for POST /guilds (create a new guild).
///
/// Only `name` is required; all other fields are optional.
#[derive(Debug, Clone, Serialize)]
pub struct CreateGuildRequest {
    /// Guild name (2–100 characters)
    pub name: String,
    /// Guild icon as a base64 data URI (`"data:image/png;base64,..."`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Verification level (0=NONE … 4=VERY_HIGH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_level: Option<u8>,
    /// Default message notification level (0=ALL_MESSAGES, 1=ONLY_MENTIONS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_message_notifications: Option<u8>,
    /// Explicit content filter level (0=DISABLED, 1=MEMBERS_WITHOUT_ROLES,
    /// 2=ALL_MEMBERS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_content_filter: Option<u8>,
}

impl CreateGuildRequest {
    /// Create a minimal request with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            icon: None,
            verification_level: None,
            default_message_notifications: None,
            explicit_content_filter: None,
        }
    }
}

/// Request body for PATCH /guilds/{id} (edit guild settings).
///
/// All fields are optional — only the fields you set will be sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditGuildRequest {
    /// New guild name (2–100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description (max 120 chars, community guilds only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Guild icon as a base64 data URI (`"data:image/png;base64,..."`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Guild banner as a base64 data URI (requires BANNER feature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// Guild splash as a base64 data URI (requires INVITE_SPLASH feature)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub splash: Option<String>,
    /// AFK channel snowflake ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afk_channel_id: Option<String>,
    /// AFK timeout in seconds (60, 300, 900, 1800, 3600)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub afk_timeout: Option<u32>,
    /// Verification level (0=NONE … 4=VERY_HIGH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_level: Option<u8>,
    /// Default message notification level (0=ALL_MESSAGES, 1=ONLY_MENTIONS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_message_notifications: Option<u8>,
    /// Explicit content filter level (0=DISABLED, 1=MEMBERS_WITHOUT_ROLES,
    /// 2=ALL_MEMBERS)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_content_filter: Option<u8>,
    /// System channel snowflake ID (for join messages etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_channel_id: Option<String>,
    /// Rules channel snowflake ID (community guilds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_channel_id: Option<String>,
    /// Public updates channel snowflake ID (community guilds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_updates_channel_id: Option<String>,
    /// Preferred locale (e.g. `"en-US"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_locale: Option<String>,
}

/// Request body for the `poll` field inside a `CreateMessage` payload.
///
/// To send a poll, include this in the message body's `poll` field.
#[derive(Debug, Clone, Serialize)]
pub struct CreatePollRequest {
    /// The question text (max 300 characters).
    pub question: PollMedia,
    /// Answer options (1-10 entries).
    pub answers: Vec<PollAnswer>,
    /// How long the poll should run, in hours (max 32 days = 768 hours).
    pub duration: u32,
    /// Whether voters may select more than one answer.
    #[serde(default)]
    pub allow_multiselect: bool,
    /// Layout type: 1 = DEFAULT (only valid value currently).
    #[serde(default = "default_layout_type")]
    pub layout_type: u8,
}

#[allow(dead_code)]
fn default_layout_type() -> u8 {
    1
}

/// Request body for POST or PATCH /guilds/{id}/auto-moderation/rules.
///
/// For creates, `name`, `event_type`, `trigger_type`, and `actions` are
/// required. For edits, all fields are optional — only set fields are sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AutoModerationRuleRequest {
    /// Rule name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Event type: 1=MESSAGE_SEND, 2=MEMBER_UPDATE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<u8>,
    /// Trigger type (immutable after creation):
    /// 1=KEYWORD, 2=SPAM, 3=KEYWORD_PRESET, 4=MENTION_SPAM, 5=MEMBER_PROFILE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_type: Option<u8>,
    /// Trigger metadata — shape depends on `trigger_type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_metadata: Option<serde_json::Value>,
    /// Actions to take when the rule fires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<serde_json::Value>>,
    /// Whether the rule is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Channel IDs exempt from this rule (max 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exempt_channels: Option<Vec<String>>,
    /// Role IDs exempt from this rule (max 20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exempt_roles: Option<Vec<String>>,
}

/// Request body for POST /guilds/{id}/scheduled-events (create a scheduled
/// event).
#[derive(Debug, Clone, Serialize)]
pub struct CreateScheduledEventRequest {
    /// Stage/voice channel ID (required for STAGE_INSTANCE and VOICE types).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Additional metadata (required for EXTERNAL type: `location` field).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_metadata: Option<serde_json::Value>,
    /// Event name (1-100 characters).
    pub name: String,
    /// Privacy level (2 = GUILD_ONLY).
    pub privacy_level: u8,
    /// ISO 8601 scheduled start time.
    pub scheduled_start_time: String,
    /// ISO 8601 scheduled end time (required for EXTERNAL type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_end_time: Option<String>,
    /// Description (1-1000 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Entity type: 1=STAGE_INSTANCE, 2=VOICE, 3=EXTERNAL.
    pub entity_type: u8,
    /// Cover image as a base64 data URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Request body for PATCH /guilds/{id}/scheduled-events/{event_id}.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditScheduledEventRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<u8>,
    /// New status: 2=ACTIVE (start), 3=COMPLETED (end), 4=CANCELLED.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

/// Request body for POST /stage-instances
#[derive(Debug, Clone, Serialize)]
pub struct CreateStageInstanceRequest {
    /// The channel ID of the stage channel.
    pub channel_id: String,
    /// The topic of the stage instance (1-120 characters).
    pub topic: String,
    /// The privacy level: 1 = PUBLIC, 2 = GUILD_ONLY (default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_level: Option<u8>,
    /// Whether to notify @everyone that a stage has started. Requires
    /// `MENTION_EVERYONE`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_start_notification: Option<bool>,
}

/// Request body for PATCH /stage-instances/{channel_id}
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditStageInstanceRequest {
    /// New topic (1-120 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// New privacy level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_level: Option<u8>,
}

/// Request body for PUT /guilds/{id}/voice-states/@me or
/// /voice-states/{user_id}
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditVoiceStateRequest {
    /// The channel the user is currently in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Whether the current user has requested to speak.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_to_speak_timestamp: Option<String>,
    /// Whether to suppress the user (bot/moderator action).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress: Option<bool>,
}

/// Request body for PATCH /users/@me (edit own profile)
///
/// All fields are optional — only included fields are sent to Discord.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditProfileRequest {
    /// New username (rate-limited to 2 changes per hour)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// New avatar as a base64-encoded data URI, e.g.
    /// `"data:image/png;base64,..."`. Pass `None` to clear.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// New banner as a base64-encoded data URI. Pass `None` to clear.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// Bio shown on the profile card (user-accounts only, max 190 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    /// Pronouns shown on the profile card (user-accounts only, max 40 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    /// Accent colour as a decimal integer (used when no banner is set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<u32>,
}

/// Request body for POST /guilds/{id}/soundboard-sounds (create a soundboard
/// sound).
///
/// The audio file must be sent as a `sound_file` multipart field — use
/// `DiscordHttpClient::post_raw_multipart` with these fields baked into the
/// form.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSoundboardSoundRequest {
    /// Sound name (2-32 characters).
    pub name: String,
    /// Base64-encoded audio data URI (mp3 or ogg, max 512 KB).
    pub sound: String,
    /// Playback volume (0.0–1.0, default 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    /// Custom emoji ID to associate with the sound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji_id: Option<String>,
    /// Unicode emoji to associate with the sound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji_name: Option<String>,
}

/// Request body for PATCH /guilds/{id}/soundboard-sounds/{sound_id}.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditSoundboardSoundRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji_name: Option<String>,
}

/// Request body for POST /channels/{id}/send-soundboard-sound.
#[derive(Debug, Clone, Serialize)]
pub struct SendSoundboardSoundRequest {
    /// The ID of the sound to play.
    pub sound_id: String,
    /// The guild ID the sound belongs to (`None` for default sounds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_guild_id: Option<String>,
}

/// Request body for POST/PATCH application command endpoints.
///
/// Used for both create and edit — omit fields you don't want to change on
/// edit.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateCommandRequest {
    /// Command name (1-32 chars; lowercase alphanumeric + hyphens for
    /// CHAT_INPUT).
    pub name: String,
    /// Description (1-100 chars for CHAT_INPUT; empty string for USER/MESSAGE).
    #[serde(default)]
    pub description: String,
    /// Command type: 1=CHAT_INPUT (default), 2=USER, 3=MESSAGE.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub command_type: Option<u8>,
    /// Parameters / subcommands.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<serde_json::Value>,
    /// Whether to enable in DMs (global commands only, default true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dm_permission: Option<bool>,
    /// Default member permissions (permission bitfield string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_member_permissions: Option<String>,
    /// Whether the command is age-restricted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/// Request body for POST /interactions/{id}/{token}/callback.
#[derive(Debug, Clone, Serialize)]
pub struct CreateInteractionResponseRequest {
    /// Interaction callback type.
    /// 1=PONG, 4=CHANNEL_MESSAGE_WITH_SOURCE, 5=DEFERRED_CHANNEL_MESSAGE,
    /// 6=DEFERRED_UPDATE_MESSAGE, 7=UPDATE_MESSAGE, 9=MODAL.
    #[serde(rename = "type")]
    pub response_type: u8,
    /// Response data (omit for PONG and DEFERRED types).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl CreateInteractionResponseRequest {
    /// Send a visible message as the interaction response (type 4).
    ///
    /// Mirrors serenity's `CreateInteractionResponse::Message(...)`.
    pub fn message(content: impl Into<String>) -> Self {
        Self { response_type: 4, data: Some(serde_json::json!({ "content": content.into() })) }
    }

    /// Send an ephemeral (only-visible-to-invoker) message as the response
    /// (type 4 + flag 64).
    ///
    /// Mirrors serenity's `CreateInteractionResponseMessage::ephemeral(true)`.
    pub fn ephemeral(content: impl Into<String>) -> Self {
        Self { response_type: 4, data: Some(serde_json::json!({ "content": content.into(), "flags": 64u64 })) }
    }

    /// Show a "thinking…" loading indicator; follow up with
    /// [`create_followup_message`] (type 5).
    ///
    /// Mirrors serenity's `CreateInteractionResponse::Defer(...)`.
    pub fn defer() -> Self {
        Self { response_type: 5, data: None }
    }

    /// Silently defer a component interaction (keeps the original message
    /// unchanged, type 6).
    ///
    /// Mirrors serenity's `CreateInteractionResponse::Acknowledge`.
    pub fn defer_update() -> Self {
        Self { response_type: 6, data: None }
    }

    /// Edit the original component message in-place (type 7).
    pub fn update_message(content: impl Into<String>) -> Self {
        Self { response_type: 7, data: Some(serde_json::json!({ "content": content.into() })) }
    }

    /// Show a modal dialog (type 9).
    ///
    /// `modal_data` should be a JSON object with `custom_id`, `title`, and
    /// `components` matching Discord's `ModalSubmitInteraction` schema.
    ///
    /// Mirrors serenity's `CreateInteractionResponse::Modal(CreateModal)`.
    pub fn modal(modal_data: serde_json::Value) -> Self {
        Self { response_type: 9, data: Some(modal_data) }
    }
}

/// Request body for followup messages (POST /webhooks/{app_id}/{token}).
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateFollowupMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub embeds: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<serde_json::Value>,
    /// Set to 64 for an ephemeral message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u64>,
}

/// Request body for POST /guilds/{id}/channels (create a guild channel).
#[derive(Debug, Clone, Serialize)]
pub struct CreateChannelRequest {
    /// Channel name (1–100 characters).
    pub name: String,
    /// Channel type (0 = text, 2 = voice, 4 = category, 5 = announcement, etc.)
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<u8>,
    /// Channel topic (text channels only, max 1024 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Slowmode rate limit in seconds (0–21600).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
    /// Bitrate in bps for voice channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    /// Max voice channel user limit (0 = unlimited, 1–99).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u32>,
    /// Whether the channel is NSFW.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    /// ID of the parent category channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Sorting position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

impl CreateChannelRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            channel_type: None,
            topic: None,
            rate_limit_per_user: None,
            bitrate: None,
            user_limit: None,
            nsfw: None,
            parent_id: None,
            position: None,
        }
    }
}

/// Allowed-mentions control for a message payload.
///
/// Mirrors serenity's `CreateAllowedMentions`.  Controls which @mentions in
/// the message content will actually trigger notifications.
///
/// # Example
/// ```ignore
/// AllowedMentions::new()
///     .everyone(false)
///     .replied_user(true)
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct AllowedMentions {
    /// Which categories of entities to parse from the content.
    /// Valid values in the array: `"roles"`, `"users"`, `"everyone"`.
    /// Leave empty and use `users` / `roles` lists for explicit opt-in.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parse: Vec<String>,
    /// Explicit list of user IDs to mention.  Used when `parse` does not
    /// include `"users"`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,
    /// Explicit list of role IDs to mention.  Used when `parse` does not
    /// include `"roles"`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    /// Whether the reply target should be pinged when this message is a reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replied_user: Option<bool>,
}

impl AllowedMentions {
    /// Create a permissive default that pings users, roles, and @everyone
    /// (same as not sending `allowed_mentions` at all).
    pub fn new() -> Self {
        Self { parse: vec!["roles".into(), "users".into(), "everyone".into()], ..Default::default() }
    }

    /// Create a restrictive default — no pings for anyone.
    pub fn none() -> Self {
        Self::default()
    }

    /// Allow (or suppress) @everyone / @here pings.
    pub fn everyone(mut self, allow: bool) -> Self {
        if allow {
            if !self.parse.contains(&"everyone".to_string()) {
                self.parse.push("everyone".into());
            }
        } else {
            self.parse.retain(|s| s != "everyone");
        }
        self
    }

    /// Allow (or suppress) pinging all roles via @role mentions.
    pub fn all_roles(mut self, allow: bool) -> Self {
        if allow {
            if !self.parse.contains(&"roles".to_string()) {
                self.parse.push("roles".into());
            }
        } else {
            self.parse.retain(|s| s != "roles");
        }
        self
    }

    /// Allow (or suppress) pinging all users via @user mentions.
    pub fn all_users(mut self, allow: bool) -> Self {
        if allow {
            if !self.parse.contains(&"users".to_string()) {
                self.parse.push("users".into());
            }
        } else {
            self.parse.retain(|s| s != "users");
        }
        self
    }

    /// Explicitly allow pinging specific users (overrides `all_users`).
    pub fn users(mut self, user_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.parse.retain(|s| s != "users");
        self.users = user_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Explicitly allow pinging specific roles (overrides `all_roles`).
    pub fn roles(mut self, role_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.parse.retain(|s| s != "roles");
        self.roles = role_ids.into_iter().map(Into::into).collect();
        self
    }

    /// Whether to ping the author of the message being replied to.
    pub fn replied_user(mut self, ping: bool) -> Self {
        self.replied_user = Some(ping);
        self
    }
}

/// Request body for PATCH /guilds/{id}/members/{user_id} (edit a guild member).
///
/// All fields are optional — only present fields are sent in the PATCH body.
/// Mirrors serenity's `EditMember` builder.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditGuildMemberRequest {
    /// Change the member's server nickname. `Some("")` clears the nickname.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick: Option<String>,
    /// Replace the member's role list with these role IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Server-mute the member in voice channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
    /// Server-deafen the member in voice channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaf: Option<bool>,
    /// Move member to a different voice channel (`None` to disconnect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// ISO8601 timestamp until which the member is communication-disabled
    /// (timed out). `Some("")` or `Some("null")` lifts an active timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication_disabled_until: Option<String>,
}

impl EditGuildMemberRequest {
    /// Change only the server nickname.
    pub fn nick(nick: impl Into<String>) -> Self {
        Self { nick: Some(nick.into()), ..Default::default() }
    }

    /// Move to a specific voice channel.
    pub fn move_to_channel(channel_id: impl Into<String>) -> Self {
        Self { channel_id: Some(channel_id.into()), ..Default::default() }
    }

    /// Disconnect from voice by clearing the channel.
    pub fn disconnect_voice() -> Self {
        Self { channel_id: Some(String::new()), ..Default::default() }
    }
}

/// Request body for PATCH /channels/{id} (edit a channel).
#[derive(Debug, Clone, Default, Serialize)]
pub struct EditChannelRequest {
    /// New channel name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New channel type (only 0 ↔ 5 conversion is supported).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub channel_type: Option<u8>,
    /// New topic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// New bitrate (voice channels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    /// New user limit (voice channels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_limit: Option<u32>,
    /// New slowmode rate limit in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_user: Option<u32>,
    /// New position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
    /// New NSFW flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    /// Move to a different parent category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}
