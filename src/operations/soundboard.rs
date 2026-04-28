//! Soundboard operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> SoundboardOps for T {}

/// Extension trait providing soundboard send, list, and guild CRUD operations.
#[allow(async_fn_in_trait)]
pub trait SoundboardOps: DiscordContext {
    /// List Discord's built-in default soundboard sounds (no guild required).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn list_default_soundboard_sounds(&self) -> Result<Vec<SoundboardSound>> {
        self.http().get(Route::ListDefaultSoundboardSounds).await
    }

    /// List all soundboard sounds uploaded to a guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_guild_soundboard_sounds(&self, guild_id: &GuildId) -> Result<Vec<SoundboardSound>> {
        self.http().get(Route::GetGuildSoundboardSounds { guild_id: guild_id.get() }).await
    }

    /// Get a single guild soundboard sound by ID.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the sound is not
    /// found.
    async fn get_guild_soundboard_sound(&self, guild_id: &GuildId, sound_id: &SoundboardSoundId) -> Result<SoundboardSound> {
        self.http().get(Route::GetGuildSoundboardSound { guild_id: guild_id.get(), sound_id: sound_id.get() }).await
    }

    /// Create a guild soundboard sound.
    ///
    /// `req.sound` must be a base64 data URI (`"data:audio/mp3;base64,..."`)
    /// for an mp3 or ogg file (max 512 KB).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD_EXPRESSIONS permission.
    async fn create_guild_soundboard_sound(&self, guild_id: &GuildId, req: CreateSoundboardSoundRequest) -> Result<SoundboardSound> {
        self.http().post(Route::CreateGuildSoundboardSound { guild_id: guild_id.get() }, req).await
    }

    /// Edit a guild soundboard sound's name, volume, or emoji.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD_EXPRESSIONS permission.
    async fn edit_guild_soundboard_sound(&self, guild_id: &GuildId, sound_id: &SoundboardSoundId, req: EditSoundboardSoundRequest) -> Result<SoundboardSound> {
        self.http().patch(Route::EditGuildSoundboardSound { guild_id: guild_id.get(), sound_id: sound_id.get() }, req).await
    }

    /// Delete a guild soundboard sound permanently.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD_EXPRESSIONS permission.
    async fn delete_guild_soundboard_sound(&self, guild_id: &GuildId, sound_id: &SoundboardSoundId) -> Result<()> {
        self.http().delete(Route::DeleteGuildSoundboardSound { guild_id: guild_id.get(), sound_id: sound_id.get() }).await
    }

    /// Send a soundboard sound in a voice channel.
    ///
    /// The current user must be connected to the voice channel.
    /// For default sounds, leave `req.source_guild_id` as `None`.
    async fn send_soundboard_sound(&self, channel_id: &ChannelId, req: SendSoundboardSoundRequest) -> Result<()> {
        self.http().post_no_response(Route::SendSoundboardSound { channel_id: channel_id.get() }, req).await
    }
}
