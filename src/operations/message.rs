//! Message operations for DiscordUser

use std::time::Duration;

#[cfg(feature = "collector")]
use async_stream;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    context::DiscordContext,
    error::{DiscordError, Result, WithContext},
    types::*,
};

impl<T: DiscordContext + Send + Sync> MessageOps for T {}

/// Extension trait providing message operations
#[allow(async_fn_in_trait)]
pub trait MessageOps: DiscordContext {
    /// Create a message builder for constructing messages with a fluent API
    ///
    /// # Example
    /// ```ignore
    /// user.message()
    ///     .channel("123456789")
    ///     .content("Hello, world!")
    ///     .reply_to("987654321")
    ///     .embed(|e| e.title("My Embed").description("Description").color(0x00FF00))
    ///     .send()
    ///     .await?;
    /// ```
    fn message(&self) -> crate::message_builder::MessageBuilder<'_> {
        crate::message_builder::MessageBuilder::new(self.http())
    }

    /// Send a message to a channel.
    ///
    /// Validates `content` length before making any HTTP request:
    /// returns [`DiscordError::Model`] with [`ModelError::MessageTooLong`] if
    /// the content exceeds 2000 characters.
    async fn send_message(&self, channel_id: &ChannelId, content: &str, reply: Option<MessageReference>) -> Result<Message> {
        crate::validate::validate_message_content(content).map_err(crate::error::DiscordError::Model)?;

        let payload = SendMessageRequest {
            content,
            tts: false,
            flags: 0,
            message_reference: reply,
            mobile_network_type: "unknown",
            ..Default::default()
        };

        self.http().post(crate::route::Route::CreateMessage { channel_id: channel_id.get() }, payload).await.with_context(|| format!("Failed to send message to channel {}", channel_id.get()))
    }

    /// Send a message and wait for a response (RPC pattern)
    async fn send_message_request(&self, channel_id: &ChannelId, content: Value, timeout_secs: u64) -> Result<Value> {
        let tracking_id = Uuid::new_v4().to_string();

        let mut payload = if content.is_object() { content.clone() } else { json!({ "content": content }) };

        if let Some(obj) = payload.as_object_mut() {
            obj.insert("tracking_id".to_string(), json!(tracking_id));
            obj.insert("___type".to_string(), json!("request"));
        }

        let content_str = serde_json::to_string(&payload)?;

        // Set up listener for response
        let tracking_id_clone = tracking_id.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Value>(1);

        let subscription = self
            .events()
            .on_event("MESSAGE_CREATE", move |event| {
                if let Some(content) = event.data.get("content").and_then(|c| c.as_str()) {
                    if let Ok(parsed) = serde_json::from_str::<Value>(content) {
                        if parsed.get("tracking_id").and_then(|t| t.as_str()) == Some(&tracking_id_clone) && parsed.get("___type").and_then(|t| t.as_str()) == Some("response") {
                            let _ = tx.try_send(parsed);
                        }
                    }
                }
            })
            .await;

        // Send the message
        let payload = SendMessageRequest { content: &content_str, tts: false, flags: 0, mobile_network_type: "unknown", ..Default::default() };

        let _msg: Message = self.http().post(crate::route::Route::CreateMessage { channel_id: channel_id.get() }, payload).await.with_context(|| format!("Failed to send RPC message to channel {}", channel_id.get()))?;

        // Wait for response
        let result = tokio::time::timeout(Duration::from_secs(timeout_secs), rx.recv()).await;

        // Clean up listener (RAII - subscription dropped here)
        drop(subscription);

        match result {
            Ok(Some(response)) => Ok(response),
            Ok(None) => Err(DiscordError::Timeout),
            Err(_) => Err(DiscordError::Timeout),
        }
    }

    /// Delete a message.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_MESSAGES to delete other users' messages.
    async fn delete_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<()> {
        self.http().delete(crate::route::Route::DeleteMessage { channel_id: channel_id.get(), message_id: message_id.get() }).await.with_context(|| format!("Failed to delete message {} in channel {}", message_id.get(), channel_id.get()))
    }

    /// Get a single message by ID
    ///
    /// # Arguments
    /// * `channel_id` - The channel the message is in
    /// * `message_id` - The message ID to fetch
    async fn get_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<Message> {
        self.http().get(crate::route::Route::GetMessage { channel_id: channel_id.get(), message_id: message_id.get() }).await.with_context(|| format!("Failed to get message {} in channel {}", message_id.get(), channel_id.get()))
    }

    /// Get messages from a channel (single page, max 100)
    ///
    /// For fetching more than 100 messages, use
    /// `get_channel_messages_paginated`.
    async fn get_channel_messages(&self, channel_id: &ChannelId, limit: u32) -> Result<Vec<Message>> {
        self.get_channel_messages_before(channel_id, limit.min(100), None).await
    }

    /// Get messages from a channel with a `before` cursor
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `limit` - Max messages to return (1-100)
    /// * `before` - Only return messages before this message ID
    async fn get_channel_messages_before(&self, channel_id: &ChannelId, limit: u32, before: Option<&MessageId>) -> Result<Vec<Message>> {
        self.http().get(crate::route::Route::GetMessages { channel_id: channel_id.get(), limit: Some(limit.min(100)), before: before.map(|id| id.get()), after: None }).await.with_context(|| format!("Failed to get messages for channel {}", channel_id.get()))
    }

    /// Get messages from a channel with an `after` cursor
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `limit` - Max messages to return (1-100)
    /// * `after` - Only return messages after this message ID
    async fn get_channel_messages_after(&self, channel_id: &ChannelId, limit: u32, after: Option<&MessageId>) -> Result<Vec<Message>> {
        self.http().get(crate::route::Route::GetMessages { channel_id: channel_id.get(), limit: Some(limit.min(100)), before: None, after: after.map(|id| id.get()) }).await.with_context(|| format!("Failed to get messages for channel {}", channel_id.get()))
    }

    /// Fetch up to `total` messages from a channel, automatically paginating
    ///
    /// Fetches in pages of 100, walking backwards (newest first) using `before`
    /// cursors. Stops when `total` messages are collected or no more messages
    /// exist.
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `total` - Maximum total messages to fetch
    async fn get_channel_messages_paginated(&self, channel_id: &ChannelId, total: u32) -> Result<Vec<Message>> {
        let mut all_messages: Vec<Message> = Vec::new();
        let mut before: Option<MessageId> = None;

        while (all_messages.len() as u32) < total {
            let page_size = (total - all_messages.len() as u32).min(100);
            let page = self.get_channel_messages_before(channel_id, page_size, before.as_ref()).await?;

            if page.is_empty() {
                break;
            }

            // Discord returns newest-first; the last message is the oldest
            if let Some(last) = page.last() {
                before = Some(last.id.parse::<MessageId>().map_err(|e| crate::error::DiscordError::InvalidRequest(format!("Invalid message ID '{}': {}", last.id, e)))?);
            }

            let page_len = page.len();
            all_messages.extend(page);

            // Incomplete page means we've reached the end
            if (page_len as u32) < page_size {
                break;
            }
        }

        Ok(all_messages)
    }

    /// Pin a message in a channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_MESSAGES permission.
    async fn pin_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<()> {
        self.http().put(crate::route::Route::PinMessage { channel_id: channel_id.get(), message_id: message_id.get() }, EMPTY_REQUEST).await
    }

    /// Unpin a message from a channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_MESSAGES permission.
    async fn unpin_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<()> {
        self.http().delete(crate::route::Route::UnpinMessage { channel_id: channel_id.get(), message_id: message_id.get() }).await
    }

    /// Get all pinned messages in a channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_pins(&self, channel_id: &ChannelId) -> Result<Vec<Message>> {
        self.http().get(crate::route::Route::GetPins { channel_id: channel_id.get() }).await
    }

    /// Crosspost (publish) a message to all channels following this
    /// announcement channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires SEND_MESSAGES in an announcement channel, or MANAGE_MESSAGES.
    async fn crosspost_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<Message> {
        self.http().post(crate::route::Route::CrosspostMessage { channel_id: channel_id.get(), message_id: message_id.get() }, EMPTY_REQUEST).await
    }

    /// Send a message with file attachments via multipart upload.
    ///
    /// # Arguments
    /// * `channel_id` - Target channel
    /// * `content` - Text content (may be empty string)
    /// * `attachments` - One or more files to attach (max 10)
    /// * `reply` - Optional message reference for replies
    ///
    /// # Example
    /// ```ignore
    /// let file = CreateAttachment::bytes("hello.txt", b"Hello, world!".to_vec());
    /// user.send_files(&channel_id, "Here's a file", vec![file], None).await?;
    /// ```
    async fn send_files(&self, channel_id: &ChannelId, content: &str, attachments: Vec<crate::types::CreateAttachment>, reply: Option<MessageReference>) -> Result<Message> {
        use serde_json::json;

        // Build attachment metadata array for payload_json
        let attachment_meta: Vec<serde_json::Value> = attachments
            .iter()
            .enumerate()
            .map(|(i, att)| {
                let mut obj = json!({ "id": i, "filename": att.filename });
                if let Some(ref desc) = att.description {
                    obj["description"] = json!(desc);
                }
                obj
            })
            .collect();

        let mut payload = json!({
            "content": content,
            "attachments": attachment_meta,
            "flags": 0,
            "mobile_network_type": "unknown"
        });

        if let Some(r) = reply {
            payload["message_reference"] = serde_json::to_value(r).unwrap_or_default();
        }

        self.http().post_multipart(crate::route::Route::CreateMessage { channel_id: channel_id.get() }, payload, attachments).await
    }

    /// React to a message with a thumbs-up emoji (👍).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn thumb_up_message(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<()> {
        self.http().put(crate::route::Route::AddReaction { channel_id: channel_id.get(), message_id: message_id.get(), emoji: "%F0%9F%91%8D" }, EMPTY_REQUEST).await
    }

    /// Edit a message
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `message_id` - The message ID to edit
    /// * `content` - New content for the message (None to keep current)
    ///
    /// # Example
    /// ```ignore
    /// user.edit_message(&ChannelId::new("123").unwrap(), &MessageId::new("456").unwrap(), Some("Updated content")).await?;
    /// ```
    async fn edit_message(&self, channel_id: &ChannelId, message_id: &MessageId, content: Option<&str>) -> Result<Message> {
        let payload = EditMessageRequest { content, ..Default::default() };

        self.http().patch(crate::route::Route::EditMessage { channel_id: channel_id.get(), message_id: message_id.get() }, payload).await.with_context(|| format!("Failed to edit message {} in channel {}", message_id.get(), channel_id.get()))
    }

    /// Bulk delete messages (2-100 messages, not older than 14 days)
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `message_ids` - Array of message IDs to delete
    ///
    /// # Note
    /// Requires the MANAGE_MESSAGES permission. Messages older than 14 days
    /// cannot be bulk deleted.
    ///
    /// # Errors
    /// Returns [`DiscordError::Model`] with [`ModelError::BulkDeleteAmount`] if
    /// the count is outside the 2–100 range.  Returns
    /// [`DiscordError::Http`] on HTTP failure.
    async fn bulk_delete_messages(&self, channel_id: &ChannelId, message_ids: Vec<MessageId>) -> Result<()> {
        crate::validate::validate_bulk_delete_count(message_ids.len()).map_err(crate::error::DiscordError::Model)?;
        let ids: Vec<String> = message_ids.iter().map(|id| id.get().to_string()).collect();
        self.http().post(crate::route::Route::BulkDeleteMessages { channel_id: channel_id.get() }, json!({ "messages": ids })).await
    }

    /// Add a reaction to a message
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `message_id` - The message ID
    /// * `emoji` - The emoji to react with. For unicode emoji, use the emoji
    ///   character. For custom emoji, use the format `name:id` (e.g.,
    ///   "emoji_name:123456789")
    ///
    /// # Example
    /// ```ignore
    /// // Unicode emoji
    /// user.add_reaction(&channel_id, &message_id, "👍").await?;
    /// // Custom emoji
    /// user.add_reaction(&channel_id, &message_id, "custom_emoji:123456789").await?;
    /// ```
    async fn add_reaction(&self, channel_id: &ChannelId, message_id: &MessageId, emoji: &str) -> Result<()> {
        let encoded_emoji = urlencoding::encode(emoji);
        self.http().put(crate::route::Route::AddReaction { channel_id: channel_id.get(), message_id: message_id.get(), emoji: &encoded_emoji }, EMPTY_REQUEST).await
    }

    /// Remove a reaction from a message
    ///
    /// # Arguments
    /// * `channel_id` - The channel ID
    /// * `message_id` - The message ID
    /// * `emoji` - The emoji to remove. For unicode emoji, use the emoji
    ///   character. For custom emoji, use the format `name:id`
    /// * `user_id` - The user ID whose reaction to remove. Use "@me" for own
    ///   reaction.
    async fn remove_reaction(&self, channel_id: &ChannelId, message_id: &MessageId, emoji: &str, user_id: &str) -> Result<()> {
        let encoded_emoji = urlencoding::encode(emoji);
        if user_id == "@me" {
            self.http().delete(crate::route::Route::RemoveOwnReaction { channel_id: channel_id.get(), message_id: message_id.get(), emoji: &encoded_emoji }).await
        } else {
            let uid = user_id.parse::<u64>().map_err(|e| crate::error::DiscordError::InvalidRequest(format!("Invalid user_id '{}': {}", user_id, e)))?;
            self.http().delete(crate::route::Route::RemoveUserReaction { channel_id: channel_id.get(), message_id: message_id.get(), emoji: &encoded_emoji, user_id: uid }).await
        }
    }

    /// Send a message with an embedded poll.
    ///
    /// The poll is included as the `poll` field in the message payload.
    /// Returns the created `Message` which includes the attached `Poll`.
    async fn send_poll(&self, channel_id: &ChannelId, content: &str, poll: CreatePollRequest) -> Result<Message> {
        let body = serde_json::json!({
            "content": content,
            "poll": poll,
        });
        self.http().post(crate::route::Route::CreateMessage { channel_id: channel_id.get() }, body).await
    }

    /// End a poll early before its scheduled expiry.
    ///
    /// Returns the updated `Message` with finalized poll results.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn end_poll(&self, channel_id: &ChannelId, message_id: &MessageId) -> Result<Message> {
        self.http().post(crate::route::Route::EndPoll { channel_id: channel_id.get(), message_id: message_id.get() }, EMPTY_REQUEST).await
    }

    /// Get the users who voted for a specific answer in a poll.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_poll_answer_voters(&self, channel_id: &ChannelId, message_id: &MessageId, answer_id: u64) -> Result<serde_json::Value> {
        self.http().get(crate::route::Route::GetPollAnswerVoters { channel_id: channel_id.get(), message_id: message_id.get(), answer_id }).await
    }

    /// Lazily iterate over all messages in a channel, paginating backwards
    /// (newest-first) using `before` cursors.
    ///
    /// Each page of up to 100 messages is fetched only when the stream is
    /// polled. The stream ends when no more messages are available.
    ///
    /// Requires the `collector` feature (uses `async_stream`).
    ///
    /// Mirrors serenity's `ChannelId::messages_iter()`.
    ///
    /// # Example
    /// ```ignore
    /// use futures::StreamExt;
    /// let mut stream = user.messages_iter(&channel_id, None);
    /// while let Some(Ok(msg)) = stream.next().await {
    ///     println!("{}: {}", msg.author.username, msg.content);
    /// }
    /// ```
    #[cfg(feature = "collector")]
    fn messages_iter<'a>(&'a self, channel_id: &'a ChannelId, before: Option<MessageId>) -> impl futures::Stream<Item = Result<Message>> + 'a {
        async_stream::try_stream! {
            let mut cursor: Option<MessageId> = before;
            loop {
                let page = match &cursor {
                    Some(id) => {
                        self.get_channel_messages_before(channel_id, 100, Some(id)).await?
                    }
                    None => {
                        self.get_channel_messages_before(channel_id, 100, None).await?
                    }
                };
                if page.is_empty() { break; }
                // Update cursor to the oldest message in this page (last item, since Discord returns newest-first)
                if let Some(last) = page.last() {
                    cursor = last.id.parse().ok();
                }
                let done = page.len() < 100;
                for msg in page {
                    yield msg;
                }
                if done { break; }
            }
        }
    }
}
