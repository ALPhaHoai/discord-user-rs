//! Guild scheduled event CRUD operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> ScheduledEventOps for T {}

/// Extension trait providing guild scheduled event CRUD.
#[allow(async_fn_in_trait)]
pub trait ScheduledEventOps: DiscordContext {
    /// List all scheduled events for a guild.
    ///
    /// Set `with_user_count` to `true` to include subscriber counts in the
    /// response.
    async fn get_guild_scheduled_events(&self, guild_id: &GuildId) -> Result<Vec<ScheduledEvent>> {
        self.http().get(Route::GetGuildScheduledEvents { guild_id: guild_id.get() }).await
    }

    /// Get a single scheduled event by ID.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the event is not
    /// found.
    async fn get_guild_scheduled_event(&self, guild_id: &GuildId, event_id: &ScheduledEventId) -> Result<ScheduledEvent> {
        self.http().get(Route::GetGuildScheduledEvent { guild_id: guild_id.get(), event_id: event_id.get() }).await
    }

    /// Create a scheduled event in a guild.
    ///
    /// `entity_type` controls where the event is hosted:
    /// - `1` = Stage instance (requires `channel_id`)
    /// - `2` = Voice channel (requires `channel_id`)
    /// - `3` = External location (requires `entity_metadata.location` +
    ///   `scheduled_end_time`)
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_EVENTS permission.
    async fn create_guild_scheduled_event(&self, guild_id: &GuildId, req: CreateScheduledEventRequest) -> Result<ScheduledEvent> {
        self.http().post(Route::CreateGuildScheduledEvent { guild_id: guild_id.get() }, req).await
    }

    /// Edit an existing scheduled event.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_EVENTS permission.
    async fn edit_guild_scheduled_event(&self, guild_id: &GuildId, event_id: &ScheduledEventId, req: EditScheduledEventRequest) -> Result<ScheduledEvent> {
        self.http().patch(Route::EditGuildScheduledEvent { guild_id: guild_id.get(), event_id: event_id.get() }, req).await
    }

    /// Delete (cancel) a scheduled event permanently.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_EVENTS permission.
    async fn delete_guild_scheduled_event(&self, guild_id: &GuildId, event_id: &ScheduledEventId) -> Result<()> {
        self.http().delete(Route::DeleteGuildScheduledEvent { guild_id: guild_id.get(), event_id: event_id.get() }).await
    }

    /// Get users who have subscribed to a scheduled event.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_guild_scheduled_event_users(&self, guild_id: &GuildId, event_id: &ScheduledEventId) -> Result<Vec<serde_json::Value>> {
        self.http().get(Route::GetGuildScheduledEventUsers { guild_id: guild_id.get(), event_id: event_id.get() }).await
    }
}
