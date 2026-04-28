//! Webhook operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> WebhookOps for T {}

/// Extension trait providing webhook operations
#[allow(async_fn_in_trait)]
pub trait WebhookOps: DiscordContext {
    /// Get all webhooks in a channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_WEBHOOKS permission.
    async fn get_channel_webhooks(&self, channel_id: &ChannelId) -> Result<Vec<Webhook>> {
        self.http().get(Route::GetChannelWebhooks { channel_id: channel_id.get() }).await
    }

    /// Get all webhooks in a guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_WEBHOOKS permission.
    async fn get_guild_webhooks(&self, guild_id: &GuildId) -> Result<Vec<Webhook>> {
        self.http().get(Route::GetGuildWebhooks { guild_id: guild_id.get() }).await
    }

    /// Create a webhook in a channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_WEBHOOKS permission.
    async fn create_webhook(&self, channel_id: &ChannelId, req: CreateWebhookRequest) -> Result<Webhook> {
        self.http().post(Route::CreateWebhook { channel_id: channel_id.get() }, req).await
    }

    /// Get a webhook by ID (requires authentication).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the webhook is not
    /// found.
    async fn get_webhook(&self, webhook_id: &WebhookId) -> Result<Webhook> {
        self.http().get(Route::GetWebhook { webhook_id: webhook_id.get() }).await
    }

    /// Get a webhook using its ID and token (no authentication required).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the webhook/token
    /// pair is invalid.
    async fn get_webhook_with_token(&self, webhook_id: &WebhookId, token: &str) -> Result<Webhook> {
        self.http().get(Route::GetWebhookWithToken { webhook_id: webhook_id.get(), token }).await
    }

    /// Edit a webhook's name, avatar, or channel.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_WEBHOOKS permission.
    async fn edit_webhook(&self, webhook_id: &WebhookId, req: EditWebhookRequest) -> Result<Webhook> {
        self.http().patch(Route::EditWebhook { webhook_id: webhook_id.get() }, req).await
    }

    /// Edit a webhook using its token (no authentication required).
    ///
    /// Note: changing the channel is not supported via the token-authenticated
    /// endpoint.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn edit_webhook_with_token(&self, webhook_id: &WebhookId, token: &str, req: EditWebhookRequest) -> Result<Webhook> {
        self.http().patch(Route::EditWebhookWithToken { webhook_id: webhook_id.get(), token }, req).await
    }

    /// Delete a webhook permanently.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_WEBHOOKS permission.
    async fn delete_webhook(&self, webhook_id: &WebhookId) -> Result<()> {
        self.http().delete(Route::DeleteWebhook { webhook_id: webhook_id.get() }).await
    }

    /// Delete a webhook using its token (no authentication required).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn delete_webhook_with_token(&self, webhook_id: &WebhookId, token: &str) -> Result<()> {
        self.http().delete(Route::DeleteWebhookWithToken { webhook_id: webhook_id.get(), token }).await
    }

    /// Execute a webhook — sends a message via the webhook.
    ///
    /// Discord returns 204 No Content by default. To receive the posted
    /// `Message` back, append `?wait=true` — not yet implemented here.
    async fn execute_webhook(&self, webhook_id: &WebhookId, token: &str, req: ExecuteWebhookRequest) -> Result<()> {
        self.http().post_no_response(Route::ExecuteWebhook { webhook_id: webhook_id.get(), token }, req).await
    }
}
