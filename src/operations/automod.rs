//! Auto-moderation rule CRUD operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> AutoModerationOps for T {}

/// Extension trait providing auto-moderation rule CRUD.
#[allow(async_fn_in_trait)]
pub trait AutoModerationOps: DiscordContext {
    /// List all auto-moderation rules for a guild.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD permission.
    async fn get_auto_moderation_rules(&self, guild_id: &GuildId) -> Result<Vec<AutoModerationRule>> {
        self.http().get(Route::GetAutoModerationRules { guild_id: guild_id.get() }).await
    }

    /// Get a single auto-moderation rule by ID.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure or if the rule is not
    /// found.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD permission.
    async fn get_auto_moderation_rule(&self, guild_id: &GuildId, rule_id: &AutoModerationRuleId) -> Result<AutoModerationRule> {
        self.http().get(Route::GetAutoModerationRule { guild_id: guild_id.get(), rule_id: rule_id.get() }).await
    }

    /// Create a new auto-moderation rule.
    ///
    /// `req.trigger_type` is required and immutable after creation.
    /// `req.actions` must contain at least one action.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD permission.
    async fn create_auto_moderation_rule(&self, guild_id: &GuildId, req: AutoModerationRuleRequest) -> Result<AutoModerationRule> {
        self.http().post(Route::CreateAutoModerationRule { guild_id: guild_id.get() }, req).await
    }

    /// Edit an existing auto-moderation rule.
    ///
    /// Note: `trigger_type` cannot be changed after creation.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD permission.
    async fn edit_auto_moderation_rule(&self, guild_id: &GuildId, rule_id: &AutoModerationRuleId, req: AutoModerationRuleRequest) -> Result<AutoModerationRule> {
        self.http().patch(Route::EditAutoModerationRule { guild_id: guild_id.get(), rule_id: rule_id.get() }, req).await
    }

    /// Delete an auto-moderation rule permanently.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    ///
    /// # Permissions
    /// Requires MANAGE_GUILD permission.
    async fn delete_auto_moderation_rule(&self, guild_id: &GuildId, rule_id: &AutoModerationRuleId) -> Result<()> {
        self.http().delete(Route::DeleteAutoModerationRule { guild_id: guild_id.get(), rule_id: rule_id.get() }).await
    }
}
