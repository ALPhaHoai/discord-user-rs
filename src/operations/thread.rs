//! Thread operations for DiscordUser

use serde::{Deserialize, Serialize};

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> ThreadOps for T {}

/// A thread member entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMember {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub join_timestamp: String,
    pub flags: u64,
}

/// Response from GET /guilds/{id}/threads/active
#[derive(Debug, Clone, Deserialize)]
pub struct ActiveThreads {
    pub threads: Vec<Channel>,
    pub members: Vec<ThreadMember>,
}

/// Extension trait providing thread management operations
#[allow(async_fn_in_trait)]
pub trait ThreadOps: DiscordContext {
    /// Create a thread not attached to a message.
    ///
    /// Set `req.channel_type` to 11 (PUBLIC_THREAD) or 12 (PRIVATE_THREAD).
    /// Use [`CreateThreadRequest::public`] / [`CreateThreadRequest::private`]
    /// helpers.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires CREATE_PUBLIC_THREADS or CREATE_PRIVATE_THREADS as appropriate.
    async fn create_thread(&self, channel_id: &ChannelId, req: CreateThreadRequest) -> Result<Channel> {
        self.http().post(Route::CreateThread { channel_id: channel_id.get() }, req).await
    }

    /// Create a thread attached to an existing message.
    ///
    /// The thread type is PUBLIC_THREAD by default (no `type` field needed).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires CREATE_PUBLIC_THREADS permission.
    async fn create_thread_from_message(&self, channel_id: &ChannelId, message_id: &MessageId, req: CreateThreadRequest) -> Result<Channel> {
        self.http().post(Route::CreateThreadFromMessage { channel_id: channel_id.get(), message_id: message_id.get() }, req).await
    }

    /// Edit a thread's settings (name, archived state, locked, auto-archive
    /// duration, slowmode).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_THREADS permission (or be the thread creator for private
    /// threads).
    async fn edit_thread(&self, channel_id: &ChannelId, req: EditThreadRequest) -> Result<Channel> {
        self.http().patch(Route::EditChannel { channel_id: channel_id.get() }, req).await
    }

    /// Join a thread as the current user.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn join_thread(&self, channel_id: &ChannelId) -> Result<()> {
        self.http().put(Route::JoinThread { channel_id: channel_id.get() }, EMPTY_REQUEST).await
    }

    /// Leave a thread as the current user.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn leave_thread(&self, channel_id: &ChannelId) -> Result<()> {
        self.http().delete(Route::LeaveThread { channel_id: channel_id.get() }).await
    }

    /// Add a member to a thread.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_THREADS or be the thread creator.
    async fn add_thread_member(&self, channel_id: &ChannelId, user_id: &UserId) -> Result<()> {
        self.http().put(Route::AddThreadMember { channel_id: channel_id.get(), user_id: user_id.get() }, EMPTY_REQUEST).await
    }

    /// Remove a member from a thread.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_THREADS or be the thread creator.
    async fn remove_thread_member(&self, channel_id: &ChannelId, user_id: &UserId) -> Result<()> {
        self.http().delete(Route::RemoveThreadMember { channel_id: channel_id.get(), user_id: user_id.get() }).await
    }

    /// Get all members of a thread.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_thread_members(&self, channel_id: &ChannelId) -> Result<Vec<ThreadMember>> {
        self.http().get(Route::GetThreadMembers { channel_id: channel_id.get() }).await
    }

    /// Get all active (non-archived) threads in a guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_active_threads(&self, guild_id: &GuildId) -> Result<ActiveThreads> {
        self.http().get(Route::GetActiveThreads { guild_id: guild_id.get() }).await
    }

    /// Archive a thread by setting `archived: true`.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_THREADS or be the thread creator.
    async fn archive_thread(&self, channel_id: &ChannelId) -> Result<Channel> {
        self.edit_thread(channel_id, EditThreadRequest { archived: Some(true), ..Default::default() }).await
    }

    /// Unarchive a thread by setting `archived: false`.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_THREADS or be the thread creator.
    async fn unarchive_thread(&self, channel_id: &ChannelId) -> Result<Channel> {
        self.edit_thread(channel_id, EditThreadRequest { archived: Some(false), ..Default::default() }).await
    }
}
