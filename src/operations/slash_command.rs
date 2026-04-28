//! Application command (slash command) CRUD and interaction response
//! operations.

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> SlashCommandOps for T {}

/// Extension trait providing application command CRUD and interaction
/// callbacks.
#[allow(async_fn_in_trait)]
pub trait SlashCommandOps: DiscordContext {
    // ── Global commands ──────────────────────────────────────────────────────

    /// List all global application commands for this application.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_global_commands(&self, application_id: &ApplicationId) -> Result<Vec<ApplicationCommand>> {
        self.http().get(Route::GetGlobalCommands { application_id: application_id.get() }).await
    }

    /// Create a new global application command.
    ///
    /// Global commands propagate to all guilds after up to 1 hour.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn create_global_command(&self, application_id: &ApplicationId, req: CreateCommandRequest) -> Result<ApplicationCommand> {
        self.http().post(Route::CreateGlobalCommand { application_id: application_id.get() }, req).await
    }

    /// Get a single global application command by ID.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the command is not
    /// found.
    async fn get_global_command(&self, application_id: &ApplicationId, command_id: &CommandId) -> Result<ApplicationCommand> {
        self.http().get(Route::GetGlobalCommand { application_id: application_id.get(), command_id: command_id.get() }).await
    }

    /// Edit an existing global application command.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_global_command(&self, application_id: &ApplicationId, command_id: &CommandId, req: CreateCommandRequest) -> Result<ApplicationCommand> {
        self.http().patch(Route::EditGlobalCommand { application_id: application_id.get(), command_id: command_id.get() }, req).await
    }

    /// Delete a global application command.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn delete_global_command(&self, application_id: &ApplicationId, command_id: &CommandId) -> Result<()> {
        self.http().delete(Route::DeleteGlobalCommand { application_id: application_id.get(), command_id: command_id.get() }).await
    }

    /// Bulk overwrite all global commands (replaces the entire list
    /// atomically).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn bulk_overwrite_global_commands(&self, application_id: &ApplicationId, commands: Vec<CreateCommandRequest>) -> Result<Vec<ApplicationCommand>> {
        self.http().put(Route::BulkOverwriteGlobalCommands { application_id: application_id.get() }, commands).await
    }

    // ── Guild commands ───────────────────────────────────────────────────────

    /// List all application commands registered in a specific guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_guild_commands(&self, application_id: &ApplicationId, guild_id: &GuildId) -> Result<Vec<ApplicationCommand>> {
        self.http().get(Route::GetGuildCommands { application_id: application_id.get(), guild_id: guild_id.get() }).await
    }

    /// Create a new guild-specific application command.
    ///
    /// Guild commands are available immediately (no propagation delay).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn create_guild_command(&self, application_id: &ApplicationId, guild_id: &GuildId, req: CreateCommandRequest) -> Result<ApplicationCommand> {
        self.http().post(Route::CreateGuildCommand { application_id: application_id.get(), guild_id: guild_id.get() }, req).await
    }

    /// Edit an existing guild-specific application command.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_guild_command(&self, application_id: &ApplicationId, guild_id: &GuildId, command_id: &CommandId, req: CreateCommandRequest) -> Result<ApplicationCommand> {
        self.http().patch(Route::EditGuildCommand { application_id: application_id.get(), guild_id: guild_id.get(), command_id: command_id.get() }, req).await
    }

    /// Delete a guild-specific application command.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn delete_guild_command(&self, application_id: &ApplicationId, guild_id: &GuildId, command_id: &CommandId) -> Result<()> {
        self.http().delete(Route::DeleteGuildCommand { application_id: application_id.get(), guild_id: guild_id.get(), command_id: command_id.get() }).await
    }

    /// Bulk overwrite all guild commands for a guild (replaces the entire list
    /// atomically).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn bulk_overwrite_guild_commands(&self, application_id: &ApplicationId, guild_id: &GuildId, commands: Vec<CreateCommandRequest>) -> Result<Vec<ApplicationCommand>> {
        self.http().put(Route::BulkOverwriteGuildCommands { application_id: application_id.get(), guild_id: guild_id.get() }, commands).await
    }

    // ── Interaction responses ─────────────────────────────────────────────────

    /// Respond to an incoming interaction.
    ///
    /// Must be called within 3 seconds of receiving the interaction.
    /// Common response types:
    /// - `4` = `CHANNEL_MESSAGE_WITH_SOURCE` (send a message)
    /// - `5` = `DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE` (show loading state)
    /// - `6` = `DEFERRED_UPDATE_MESSAGE` (for component interactions)
    /// - `7` = `UPDATE_MESSAGE` (edit the original component message)
    /// - `9` = `MODAL` (show a modal form)
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the 3-second window
    /// has passed.
    async fn create_interaction_response(&self, interaction_id: &InteractionId, interaction_token: &str, req: CreateInteractionResponseRequest) -> Result<()> {
        self.http().post_no_response(Route::CreateInteractionResponse { interaction_id: interaction_id.get(), interaction_token }, req).await
    }

    /// Get the original response message for an interaction.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_original_interaction_response(&self, application_id: &ApplicationId, interaction_token: &str) -> Result<Message> {
        self.http().get(Route::GetOriginalInteractionResponse { application_id: application_id.get(), interaction_token }).await
    }

    /// Edit the original response message for an interaction.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_original_interaction_response(&self, application_id: &ApplicationId, interaction_token: &str, req: CreateFollowupMessageRequest) -> Result<Message> {
        self.http().patch(Route::EditOriginalInteractionResponse { application_id: application_id.get(), interaction_token }, req).await
    }

    /// Delete the original response message for an interaction.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn delete_original_interaction_response(&self, application_id: &ApplicationId, interaction_token: &str) -> Result<()> {
        self.http().delete(Route::DeleteOriginalInteractionResponse { application_id: application_id.get(), interaction_token }).await
    }

    /// Create a followup message after an initial deferred response.
    ///
    /// Interaction tokens are valid for 15 minutes after the original response.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn create_followup_message(&self, application_id: &ApplicationId, interaction_token: &str, req: CreateFollowupMessageRequest) -> Result<Message> {
        self.http().post(Route::CreateFollowupMessage { application_id: application_id.get(), interaction_token }, req).await
    }

    /// Edit an existing followup message.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_followup_message(&self, application_id: &ApplicationId, interaction_token: &str, message_id: &MessageId, req: CreateFollowupMessageRequest) -> Result<Message> {
        self.http().patch(Route::EditFollowupMessage { application_id: application_id.get(), interaction_token, message_id: message_id.get() }, req).await
    }

    /// Delete a followup message.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn delete_followup_message(&self, application_id: &ApplicationId, interaction_token: &str, message_id: &MessageId) -> Result<()> {
        self.http().delete(Route::DeleteFollowupMessage { application_id: application_id.get(), interaction_token, message_id: message_id.get() }).await
    }

    // ── Typed convenience wrappers ────────────────────────────────────────────

    /// Respond to an interaction with a visible message (type 4).
    ///
    /// Mirrors serenity's `interaction.create_response(ctx,
    /// CreateInteractionResponse::Message(...))`.
    async fn respond_with_message(&self, interaction_id: &InteractionId, interaction_token: &str, content: &str) -> Result<()> {
        self.create_interaction_response(interaction_id, interaction_token, CreateInteractionResponseRequest::message(content)).await
    }

    /// Respond to an interaction with an ephemeral message visible only to the
    /// invoker (type 4, flag 64).
    ///
    /// Mirrors serenity's `CreateInteractionResponseMessage::ephemeral(true)`.
    async fn respond_ephemeral(&self, interaction_id: &InteractionId, interaction_token: &str, content: &str) -> Result<()> {
        self.create_interaction_response(interaction_id, interaction_token, CreateInteractionResponseRequest::ephemeral(content)).await
    }

    /// Defer an interaction response, showing a "thinking…" indicator (type 5).
    /// Follow up later using [`create_followup_message`].
    ///
    /// Mirrors serenity's `CreateInteractionResponse::Defer(...)`.
    async fn defer_interaction(&self, interaction_id: &InteractionId, interaction_token: &str) -> Result<()> {
        self.create_interaction_response(interaction_id, interaction_token, CreateInteractionResponseRequest::defer()).await
    }
}
