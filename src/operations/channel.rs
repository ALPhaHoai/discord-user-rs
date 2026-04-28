//! Channel operations for DiscordUser

use serde_json::json;

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> ChannelOps for T {}

/// Extension trait providing channel operations
#[allow(async_fn_in_trait)]
pub trait ChannelOps: DiscordContext {
    /// Get or create a private channel (DM)
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_my_private_channel(&self, recipients: Vec<UserId>) -> Result<Channel> {
        let ids: Vec<String> = recipients.iter().map(|id| id.get().to_string()).collect();
        self.http().post(Route::CreateDm, json!({ "recipients": ids })).await
    }

    /// Trigger a typing indicator in a channel (lasts ~10 seconds).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn broadcast_typing(&self, channel_id: &ChannelId) -> Result<()> {
        self.http().post_empty(Route::TriggerTyping { channel_id: channel_id.get() }).await
    }

    /// Set a voice channel status message (visible in the channel header).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires being connected to the voice channel, or MANAGE_CHANNELS.
    async fn set_channel_status(&self, channel_id: &ChannelId, status: &str) -> Result<()> {
        self.http().put(Route::UpdateVoiceStatus { channel_id: channel_id.get() }, json!({ "status": status })).await
    }

    /// Fetch a channel's current settings and metadata.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the channel is not
    /// found.
    async fn get_channel(&self, channel_id: &ChannelId) -> Result<Channel> {
        self.http().get(Route::GetChannel { channel_id: channel_id.get() }).await
    }

    /// Create a new channel in a guild (POST /guilds/{id}/channels).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_CHANNELS permission in the guild.
    async fn create_channel(&self, guild_id: &GuildId, req: CreateChannelRequest) -> Result<Channel> {
        self.http().post(Route::CreateGuildChannel { guild_id: guild_id.get() }, req).await
    }

    /// Edit a channel's settings (PATCH /channels/{id}).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_CHANNELS permission.
    async fn edit_channel(&self, channel_id: &ChannelId, req: EditChannelRequest) -> Result<Channel> {
        self.http().patch(Route::EditChannel { channel_id: channel_id.get() }, req).await
    }

    /// Delete or close a channel (DELETE /channels/{id}).
    ///
    /// For guild channels this permanently deletes the channel.
    /// For DM channels it closes the conversation (can be re-opened).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_CHANNELS for guild channels.
    async fn delete_channel(&self, channel_id: &ChannelId) -> Result<Channel> {
        self.http().delete_with_response(Route::DeleteChannel { channel_id: channel_id.get() }).await
    }

    /// Get guild information
    ///
    /// # Arguments
    /// * `guild_id` - The guild ID
    /// * `with_counts` - Whether to include approximate member and presence
    ///   counts
    async fn get_guild(&self, guild_id: &GuildId, with_counts: bool) -> Result<Guild> {
        self.http().get(Route::GetGuild { guild_id: guild_id.get(), with_counts }).await
    }

    /// Get user profile
    ///
    /// # Arguments
    /// * `user_id` - The user ID to get profile for
    /// * `guild_id` - Optional guild ID for guild-specific profile data
    ///
    /// # Returns
    /// User profile data including bio, banner, connected accounts, etc.
    async fn get_user_profile(&self, user_id: &UserId, guild_id: Option<&GuildId>) -> Result<UserProfile> {
        self.http().get(Route::GetUserProfile { user_id: user_id.get(), guild_id: guild_id.map(|g| g.get()) }).await
    }
}
