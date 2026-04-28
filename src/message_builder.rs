//! Message builder for ergonomic message sending
//!
//! Provides a fluent API for constructing and sending Discord messages.
//!
//! # Example
//! ```ignore
//! user.message()
//!     .channel("123456789")
//!     .content("Hello, world!")
//!     .reply_to("987654321")
//!     .embed(|e| e.title("My Embed").description("A description").color(0x00FF00))
//!     .send()
//!     .await?;
//! ```

use serde_json::json;

use crate::{
    client::DiscordHttpClient,
    components::CreateActionRow,
    error::Result,
    types::{AllowedMentions, Colour, Embed, EmbedAuthor, EmbedField, EmbedFooter, EmbedMedia, Message, MessageReference},
};

/// Builder for constructing Discord messages with a fluent API
pub struct MessageBuilder<'a> {
    http: &'a DiscordHttpClient,
    channel_id: Option<String>,
    guild_id: Option<String>,
    content: Option<String>,
    reply_reference: Option<MessageReference>,
    embeds: Vec<Embed>,
    tts: bool,
    flags: u64,
    nonce: Option<String>,
    sticker_ids: Vec<String>,
    allowed_mentions: Option<AllowedMentions>,
    components: Vec<CreateActionRow>,
}

impl<'a> MessageBuilder<'a> {
    /// Create a new message builder
    pub fn new(http: &'a DiscordHttpClient) -> Self {
        Self {
            http,
            channel_id: None,
            guild_id: None,
            content: None,
            reply_reference: None,
            embeds: Vec::new(),
            tts: false,
            flags: 0,
            nonce: None,
            sticker_ids: Vec::new(),
            allowed_mentions: None,
            components: Vec::new(),
        }
    }

    /// Set the channel to send the message to (required)
    pub fn channel(mut self, channel_id: impl Into<String>) -> Self {
        self.channel_id = Some(channel_id.into());
        self
    }

    /// Set the guild ID (used in reply references for guild channels)
    pub fn guild(mut self, guild_id: impl Into<String>) -> Self {
        self.guild_id = Some(guild_id.into());
        self
    }

    /// Set the message content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Reply to a specific message
    ///
    /// # Arguments
    /// * `message_id` - The ID of the message to reply to
    ///
    /// Uses the guild_id set via `.guild()` if available. For guild channels,
    /// call `.guild(id)` before `.reply_to()` so the reference includes the
    /// guild.
    pub fn reply_to(mut self, message_id: impl Into<String>) -> Self {
        let msg_id = message_id.into();
        self.reply_reference = Some(MessageReference { message_id: Some(msg_id), channel_id: self.channel_id.clone(), guild_id: self.guild_id.clone() });
        self
    }

    /// Reply to a specific message with full context
    ///
    /// # Arguments
    /// * `message_id` - The ID of the message to reply to
    /// * `channel_id` - The channel ID where the message is
    /// * `guild_id` - Optional guild ID
    pub fn reply_to_full(mut self, message_id: impl Into<String>, channel_id: impl Into<String>, guild_id: Option<impl Into<String>>) -> Self {
        self.reply_reference = Some(MessageReference {
            message_id: Some(message_id.into()),
            channel_id: Some(channel_id.into()),
            guild_id: guild_id.map(|g| g.into()),
        });
        self
    }

    /// Add an embed to the message using a builder closure
    ///
    /// # Example
    /// ```ignore
    /// builder.embed(|e| {
    ///     e.title("My Title")
    ///      .description("My Description")
    ///      .color(0xFF5733)
    /// })
    /// ```
    pub fn embed<F>(mut self, builder_fn: F) -> Self
    where
        F: FnOnce(EmbedBuilder) -> EmbedBuilder,
    {
        let embed = builder_fn(EmbedBuilder::new()).build();
        self.embeds.push(embed);
        self
    }

    /// Add a pre-built embed to the message
    pub fn with_embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Add multiple embeds to the message
    pub fn with_embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.embeds.extend(embeds);
        self
    }

    /// Enable or disable text-to-speech
    pub fn tts(mut self, enabled: bool) -> Self {
        self.tts = enabled;
        self
    }

    /// Set message flags
    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }

    /// Set a nonce for the message
    pub fn nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Add sticker IDs to the message
    pub fn stickers(mut self, sticker_ids: Vec<String>) -> Self {
        self.sticker_ids = sticker_ids;
        self
    }

    /// Add a single sticker ID to the message
    pub fn sticker(mut self, sticker_id: impl Into<String>) -> Self {
        self.sticker_ids.push(sticker_id.into());
        self
    }

    /// Suppress embeds in the message
    pub fn suppress_embeds(mut self) -> Self {
        self.flags |= 1 << 2; // SUPPRESS_EMBEDS flag
        self
    }

    /// Mark message as silent (no notification)
    pub fn silent(mut self) -> Self {
        self.flags |= 1 << 12; // SUPPRESS_NOTIFICATIONS flag
        self
    }

    /// Attach interactive components (buttons, select menus) to this message.
    ///
    /// Each [`CreateActionRow`] holds up to 5 buttons or 1 select menu.
    /// A message may have up to 5 action rows.
    ///
    /// # Example
    /// ```ignore
    /// use discord_user::components::{CreateActionRow, CreateButton, ButtonStyle};
    /// user.message()
    ///     .channel(channel_id)
    ///     .content("Pick one:")
    ///     .components(vec![
    ///         CreateActionRow::buttons(vec![
    ///             CreateButton::new("yes", ButtonStyle::Success).label("Yes"),
    ///             CreateButton::new("no",  ButtonStyle::Danger).label("No"),
    ///         ]),
    ///     ])
    ///     .send().await?;
    /// ```
    pub fn components(mut self, rows: Vec<CreateActionRow>) -> Self {
        self.components = rows;
        self
    }

    /// Control which mentions in the message content trigger notifications.
    ///
    /// By default Discord pings every mentioned entity.  Use this to suppress
    /// @everyone or restrict to specific users/roles.
    ///
    /// # Example
    /// ```ignore
    /// use discord_user::types::AllowedMentions;
    /// user.message()
    ///     .channel(channel_id)
    ///     .content("Hey @everyone!")
    ///     .allowed_mentions(AllowedMentions::none()) // suppress all pings
    ///     .send().await?;
    /// ```
    pub fn allowed_mentions(mut self, am: AllowedMentions) -> Self {
        self.allowed_mentions = Some(am);
        self
    }

    /// Send the message
    ///
    /// # Errors
    /// Returns an error if:
    /// - No channel ID was set
    /// - Neither content, embeds, nor stickers were provided
    /// - Any length limit is exceeded (validated before the HTTP call)
    /// - The API request fails
    pub async fn send(self) -> Result<Message> {
        let channel_id = self.channel_id.ok_or_else(|| crate::error::DiscordError::InvalidRequest("Channel ID is required".into()))?;

        // Validate that we have something to send
        let has_content = self.content.as_ref().is_some_and(|c| !c.is_empty());
        let has_embeds = !self.embeds.is_empty();
        let has_stickers = !self.sticker_ids.is_empty();

        if !has_content && !has_embeds && !has_stickers {
            return Err(crate::error::DiscordError::InvalidRequest("Message must have content, embeds, or stickers".into()));
        }

        // Client-side length validation (mirrors Discord's hard limits)
        if let Some(ref content) = self.content {
            if content.len() > 2000 {
                return Err(crate::error::DiscordError::InvalidRequest(format!("Message content too long: {} chars (max 2000)", content.len())));
            }
        }
        if self.embeds.len() > 10 {
            return Err(crate::error::DiscordError::InvalidRequest(format!("Too many embeds: {} (max 10)", self.embeds.len())));
        }
        if self.sticker_ids.len() > 3 {
            return Err(crate::error::DiscordError::InvalidRequest(format!("Too many stickers: {} (max 3)", self.sticker_ids.len())));
        }
        for embed in &self.embeds {
            if let Some(ref title) = embed.title {
                if title.len() > 256 {
                    return Err(crate::error::DiscordError::InvalidRequest(format!("Embed title too long: {} chars (max 256)", title.len())));
                }
            }
            if let Some(ref desc) = embed.description {
                if desc.len() > 4096 {
                    return Err(crate::error::DiscordError::InvalidRequest(format!("Embed description too long: {} chars (max 4096)", desc.len())));
                }
            }
            for field in &embed.fields {
                if field.name.len() > 256 {
                    return Err(crate::error::DiscordError::InvalidRequest(format!("Embed field name too long: {} chars (max 256)", field.name.len())));
                }
                if field.value.len() > 1024 {
                    return Err(crate::error::DiscordError::InvalidRequest(format!("Embed field value too long: {} chars (max 1024)", field.value.len())));
                }
            }
        }

        let mut payload = json!({
            "tts": self.tts,
            "flags": self.flags,
            "mobile_network_type": "unknown"
        });

        if let Some(content) = self.content {
            payload["content"] = json!(content);
        }

        if !self.embeds.is_empty() {
            payload["embeds"] = json!(self.embeds);
        }

        if let Some(reply_ref) = self.reply_reference {
            payload["message_reference"] = json!({
                "message_id": reply_ref.message_id,
                "channel_id": reply_ref.channel_id,
                "guild_id": reply_ref.guild_id
            });
        }

        if let Some(nonce) = self.nonce {
            payload["nonce"] = json!(nonce);
        }

        if !self.sticker_ids.is_empty() {
            payload["sticker_ids"] = json!(self.sticker_ids);
        }

        if let Some(am) = self.allowed_mentions {
            payload["allowed_mentions"] = serde_json::to_value(&am).unwrap_or(serde_json::Value::Null);
        }

        if !self.components.is_empty() {
            payload["components"] = serde_json::Value::Array(self.components.iter().map(|row| row.to_json()).collect());
        }

        let c_id = channel_id.parse().unwrap_or(1);
        self.http.post(crate::route::Route::CreateMessage { channel_id: c_id }, payload).await
    }
}

/// Builder for constructing Discord embeds
#[derive(Debug, Clone, Default)]
pub struct EmbedBuilder {
    title: Option<String>,
    description: Option<String>,
    url: Option<String>,
    timestamp: Option<String>,
    color: Option<u32>,
    footer: Option<EmbedFooter>,
    image: Option<EmbedMedia>,
    thumbnail: Option<EmbedMedia>,
    author: Option<EmbedAuthor>,
    fields: Vec<EmbedField>,
}

impl EmbedBuilder {
    /// Create a new embed builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the embed title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the embed description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the embed URL (makes the title clickable)
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the embed timestamp (ISO 8601 format)
    pub fn timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set the embed color.
    ///
    /// Accepts a [`Colour`] value or any type that converts into one (including
    /// plain `u32` hex literals for backward compatibility).
    ///
    /// # Example
    /// ```ignore
    /// embed.color(Colour::BLURPLE)
    /// embed.color(0xFF5733u32) // plain hex still works
    /// ```
    pub fn color(mut self, color: impl Into<Colour>) -> Self {
        self.color = Some(color.into().value());
        self
    }

    /// Set the footer text
    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(EmbedFooter { text: text.into(), icon_url: None, proxy_icon_url: None });
        self
    }

    /// Set the footer with text and icon
    pub fn footer_with_icon(mut self, text: impl Into<String>, icon_url: impl Into<String>) -> Self {
        self.footer = Some(EmbedFooter { text: text.into(), icon_url: Some(icon_url.into()), proxy_icon_url: None });
        self
    }

    /// Set the embed image
    pub fn image(mut self, url: impl Into<String>) -> Self {
        self.image = Some(EmbedMedia { url: Some(url.into()), proxy_url: None, height: None, width: None });
        self
    }

    /// Set the embed thumbnail
    pub fn thumbnail(mut self, url: impl Into<String>) -> Self {
        self.thumbnail = Some(EmbedMedia { url: Some(url.into()), proxy_url: None, height: None, width: None });
        self
    }

    /// Set the embed author name
    pub fn author(mut self, name: impl Into<String>) -> Self {
        self.author = Some(EmbedAuthor { name: Some(name.into()), url: None, icon_url: None, proxy_icon_url: None });
        self
    }

    /// Set the embed author with full details
    pub fn author_full(mut self, name: impl Into<String>, url: Option<impl Into<String>>, icon_url: Option<impl Into<String>>) -> Self {
        self.author = Some(EmbedAuthor {
            name: Some(name.into()),
            url: url.map(|u| u.into()),
            icon_url: icon_url.map(|u| u.into()),
            proxy_icon_url: None,
        });
        self
    }

    /// Add a field to the embed
    pub fn field(mut self, name: impl Into<String>, value: impl Into<String>, inline: bool) -> Self {
        self.fields.push(EmbedField { name: name.into(), value: value.into(), inline });
        self
    }

    /// Add an inline field to the embed
    pub fn inline_field(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.field(name, value, true)
    }

    /// Build the embed
    pub fn build(self) -> Embed {
        Embed {
            title: self.title,
            embed_type: Some("rich".into()),
            description: self.description,
            url: self.url,
            timestamp: self.timestamp,
            color: self.color,
            footer: self.footer,
            image: self.image,
            thumbnail: self.thumbnail,
            video: None,
            provider: None,
            author: self.author,
            fields: self.fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_builder() {
        let embed = EmbedBuilder::new().title("Test Title").description("Test Description").color(0xFF5733).field("Field 1", "Value 1", true).field("Field 2", "Value 2", false).build();

        assert_eq!(embed.title, Some("Test Title".into()));
        assert_eq!(embed.description, Some("Test Description".into()));
        assert_eq!(embed.color, Some(0xFF5733));
        assert_eq!(embed.fields.len(), 2);
        assert!(embed.fields[0].inline);
        assert!(!embed.fields[1].inline);
    }
}
