//! Discord message types

use serde::{Deserialize, Serialize};

use super::{ChannelId, GuildId, Member, MessageFlags, MessageId, MessageType, User, UserId};

/// Discord message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    #[serde(default)]
    pub guild_id: Option<String>,
    pub author: User,
    #[serde(default)]
    pub member: Option<Member>,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub edited_timestamp: Option<String>,
    #[serde(default)]
    pub tts: bool,
    #[serde(default)]
    pub mention_everyone: bool,
    #[serde(default)]
    pub mentions: Vec<User>,
    #[serde(default)]
    pub mention_roles: Vec<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub nonce: Option<serde_json::Value>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub webhook_id: Option<String>,
    #[serde(default, rename = "type")]
    pub message_type: MessageType,
    #[serde(default)]
    pub flags: MessageFlags,
    #[serde(default)]
    pub referenced_message: Option<Box<Message>>,
    #[serde(default)]
    pub components: Vec<serde_json::Value>,
    /// Attached poll, if this message contains one.
    #[serde(default)]
    pub poll: Option<Poll>,
}

impl Message {
    /// Typed accessor for the message's channel ID.
    pub fn channel_id_typed(&self) -> Option<ChannelId> {
        self.channel_id.parse().ok().map(ChannelId::new)
    }

    /// Typed accessor for the message author's user ID.
    pub fn author_id(&self) -> Option<UserId> {
        self.author.id.parse().ok().map(UserId::new)
    }

    /// Typed accessor for the guild ID (None for DMs).
    pub fn guild_id_typed(&self) -> Option<GuildId> {
        self.guild_id.as_deref()?.parse().ok().map(GuildId::new)
    }

    /// Typed accessor for the message's own ID.
    pub fn message_id(&self) -> Option<MessageId> {
        self.id.parse().ok().map(MessageId::new)
    }

    /// Create a `MessageReference` pointing to this message, suitable for use
    /// as a reply in [`SendMessageRequest`].
    ///
    /// Returns `None` if the channel ID cannot be parsed.
    pub fn as_reference(&self) -> MessageReference {
        MessageReference {
            message_id: Some(self.id.clone()),
            channel_id: Some(self.channel_id.clone()),
            guild_id: self.guild_id.clone(),
        }
    }

    /// Create a [`SendMessageRequest`]-compatible reply builder pre-filled with
    /// a message reference pointing to this message.
    pub fn reply_builder<'a>(&self, content: &'a str) -> crate::types::SendMessageRequest<'a> {
        crate::types::SendMessageRequest {
            content,
            tts: false,
            flags: 0,
            message_reference: Some(self.as_reference()),
            nonce: None,
            mobile_network_type: "unknown",
        }
    }

    /// Whether the message is pinned in its channel.
    ///
    /// Mirrors serenity's `Message::pinned`.
    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// Whether this is a text-to-speech message.
    pub fn is_tts(&self) -> bool {
        self.tts
    }

    /// Whether the message is a system message (join notification, boost, pin,
    /// etc.) rather than a user-authored message.
    ///
    /// Mirrors serenity's `Message::is_system()`.
    pub fn is_system(&self) -> bool {
        !matches!(self.message_type, MessageType::Default | MessageType::Reply | MessageType::ChatInputCommand | MessageType::ContextMenuCommand)
    }

    /// Whether this is an ephemeral message visible only to the target user.
    ///
    /// Mirrors serenity's `Message::flags` check for `EPHEMERAL`.
    pub fn is_ephemeral(&self) -> bool {
        self.flags.contains(MessageFlags::EPHEMERAL)
    }

    /// Whether the message @mentions @everyone or @here.
    pub fn mentions_everyone(&self) -> bool {
        self.mention_everyone
    }

    /// Whether the message content mentions a specific user by their ID.
    ///
    /// Checks the `mentions` array (populated by Discord when content contains
    /// `<@user_id>`).  Mirrors serenity's `Message::mentions_user()`.
    pub fn mentions_user_id(&self, user_id: &str) -> bool {
        self.mentions.iter().any(|u| u.id == user_id)
    }

    /// Whether this message is a reply to another message.
    pub fn is_reply(&self) -> bool {
        matches!(self.message_type, MessageType::Reply) || self.referenced_message.is_some()
    }

    /// Whether the message has been crossposted to follower channels.
    pub fn is_crosspost(&self) -> bool {
        self.flags.contains(MessageFlags::IS_CROSSPOST)
    }
}

/// A Discord poll attached to a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    /// The question text.
    pub question: PollMedia,
    /// The available answers.
    pub answers: Vec<PollAnswer>,
    /// ISO 8601 expiry time.
    pub expiry: Option<String>,
    /// Whether this is a multi-select poll.
    #[serde(default)]
    pub allow_multiselect: bool,
    /// Layout type: 1 = DEFAULT.
    #[serde(default = "default_layout")]
    pub layout_type: u8,
    /// Results (present when poll has ended or `with_results=true`).
    #[serde(default)]
    pub results: Option<PollResults>,
}

fn default_layout() -> u8 {
    1
}

/// Text/emoji content used for poll questions and answers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PollMedia {
    /// Display text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Emoji — either `{"id": "snowflake"}` for custom or `{"name": "🎉"}` for
    /// Unicode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<serde_json::Value>,
}

/// A single answer option in a poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAnswer {
    /// Answer index (1-based, assigned by Discord).
    pub answer_id: u64,
    /// Display content.
    pub poll_media: PollMedia,
}

/// Vote counts after a poll closes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResults {
    /// Whether the results are finalized.
    #[serde(default)]
    pub is_finalized: bool,
    /// Per-answer vote counts.
    pub answer_counts: Vec<PollAnswerCount>,
}

/// Vote count for a single poll answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollAnswerCount {
    pub id: u64,
    pub count: u64,
    /// Whether the current user voted for this answer.
    #[serde(default)]
    pub me_voted: bool,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    pub size: u64,
    pub url: String,
    pub proxy_url: String,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub ephemeral: bool,
}

/// Message embed
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Embed {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, rename = "type")]
    pub embed_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub footer: Option<EmbedFooter>,
    #[serde(default)]
    pub image: Option<EmbedMedia>,
    #[serde(default)]
    pub thumbnail: Option<EmbedMedia>,
    #[serde(default)]
    pub video: Option<EmbedMedia>,
    #[serde(default)]
    pub provider: Option<EmbedProvider>,
    #[serde(default)]
    pub author: Option<EmbedAuthor>,
    #[serde(default)]
    pub fields: Vec<EmbedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedFooter {
    pub text: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedMedia {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedProvider {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedAuthor {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub proxy_icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

/// Message reaction
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Reaction {
    pub count: u32,
    #[serde(default)]
    pub me: bool,
    pub emoji: Emoji,
}

/// Discord emoji
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Emoji {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub animated: bool,
}

/// Message reference for replies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
}

/// Request payload for sending a message
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest<'a> {
    pub content: &'a str,
    pub tts: bool,
    pub flags: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<&'a str>,
    pub mobile_network_type: &'a str,
}

impl<'a> Default for SendMessageRequest<'a> {
    fn default() -> Self {
        Self { content: "", tts: false, flags: 0, message_reference: None, nonce: None, mobile_network_type: "unknown" }
    }
}

/// Request payload for editing a message
#[derive(Debug, Clone, Serialize, Default)]
pub struct EditMessageRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u64>,
}
