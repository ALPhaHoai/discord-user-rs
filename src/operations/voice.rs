//! Voice region and voice state operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> VoiceOps for T {}

/// Extension trait providing voice region listing and voice state editing.
#[allow(async_fn_in_trait)]
pub trait VoiceOps: DiscordContext {
    /// List all available voice regions globally.
    ///
    /// Use these region IDs when creating or editing a guild's `region` field.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_voice_regions(&self) -> Result<Vec<VoiceRegion>> {
        self.http().get(Route::GetVoiceRegions).await
    }

    /// List voice regions available for a specific guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_guild_voice_regions(&self, guild_id: &GuildId) -> Result<Vec<VoiceRegion>> {
        self.http().get(Route::GetGuildVoiceRegions { guild_id: guild_id.get() }).await
    }

    /// Edit the current user's voice state in a guild.
    ///
    /// Use this to request to speak in a Stage channel (set
    /// `request_to_speak_timestamp` to the current ISO 8601 time) or to
    /// move the user to a different channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_my_voice_state(&self, guild_id: &GuildId, req: EditVoiceStateRequest) -> Result<()> {
        self.http().patch_no_response(Route::EditMyVoiceState { guild_id: guild_id.get() }, req).await
    }

    /// Edit another user's voice state in a guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MUTE_MEMBERS permission.
    async fn edit_voice_state(&self, guild_id: &GuildId, user_id: &UserId, req: EditVoiceStateRequest) -> Result<()> {
        self.http().patch_no_response(Route::EditVoiceState { guild_id: guild_id.get(), user_id: user_id.get() }, req).await
    }
}
