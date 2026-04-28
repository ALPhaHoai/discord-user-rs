//! Cache-fallback-to-HTTP pattern.
//!
//! The [`CacheHttp`] trait abstracts over callers that may or may not have an
//! in-memory cache available.  Helpers on the trait try the cache first and
//! only make an HTTP request on a cache miss, reducing Discord API calls.
//!
//! # Implementations
//! - `DiscordUser` — has both cache and HTTP; cache is tried first.
//! - `DiscordHttpClient` — HTTP only; every call hits the API.
//!
//! # Example
//! ```ignore
//! async fn show_guild(ctx: &impl CacheHttp, id: &GuildId) {
//!     if let Ok(guild) = ctx.guild(id).await {
//!         println!("{}", guild.name.unwrap_or_default());
//!     }
//! }
//! // Works with DiscordUser (cache-first) or bare &DiscordHttpClient (HTTP only):
//! show_guild(&user, &guild_id).await;
//! show_guild(user.http(), &guild_id).await;
//! ```

use crate::{
    cache::Cache,
    client::DiscordHttpClient,
    error::Result,
    route::Route,
    types::{Channel, ChannelId, Guild, GuildId, User, UserId, UserProfile},
};

// ── Trait ────────────────────────────────────────────────────────────────────

/// Provides access to an HTTP client and an optional in-memory cache.
///
/// Implementors that expose a cache will have lookups served from it when
/// possible; others always fall back to HTTP.
#[allow(async_fn_in_trait)]
pub trait CacheHttp {
    /// Underlying HTTP client used for API requests.
    fn http(&self) -> &DiscordHttpClient;

    /// Optional in-memory cache.  Returns `None` for HTTP-only implementors.
    fn cache(&self) -> Option<&Cache> {
        None
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Get a guild — from cache if available, otherwise via HTTP.
    async fn guild(&self, guild_id: &GuildId) -> Result<Guild> {
        if let Some(guild) = self.cache().and_then(|c| c.guild(&guild_id.to_string())) {
            return Ok(guild);
        }
        self.http().get(Route::GetGuild { guild_id: guild_id.get(), with_counts: false }).await
    }

    /// Get a channel — from cache if available, otherwise via HTTP.
    ///
    /// Note: the channel cache is not yet populated from gateway events;
    /// this currently always falls through to HTTP.
    async fn channel(&self, channel_id: &ChannelId) -> Result<Channel> {
        self.http().get(Route::GetChannel { channel_id: channel_id.get() }).await
    }

    /// Get a user — from cache if available, otherwise via HTTP (user profile).
    async fn user(&self, user_id: &UserId) -> Result<User> {
        if let Some(user) = self.cache().and_then(|c| c.user(&user_id.to_string())) {
            return Ok(user);
        }
        let profile: UserProfile = self.http().get(Route::GetUserProfile { user_id: user_id.get(), guild_id: None }).await?;
        Ok(profile.user)
    }
}

// ── impl for DiscordHttpClient (HTTP-only) ───────────────────────────────────

impl CacheHttp for DiscordHttpClient {
    fn http(&self) -> &DiscordHttpClient {
        self
    }
    // cache() returns None (default)
}

// ── impl for DiscordUser lives in discord_user.rs to avoid circular imports ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{Cache, CacheSettings};

    /// A minimal test double that implements CacheHttp with a cache but no
    /// real HTTP client (we only test cache-hit path here).
    struct FakeCtx {
        http: DiscordHttpClient,
        cache: Cache,
    }

    impl CacheHttp for FakeCtx {
        fn http(&self) -> &DiscordHttpClient {
            &self.http
        }
        fn cache(&self) -> Option<&Cache> {
            Some(&self.cache)
        }
    }

    fn fake_guild(id: &str) -> Guild {
        serde_json::from_value(serde_json::json!({
            "id": id, "name": "Cached Guild", "icon": null,
            "splash": null, "banner": null, "description": null,
            "owner_id": null, "member_count": null,
            "premium_subscription_count": 0, "premium_tier": 0,
            "verification_level": 0, "nsfw_level": 0, "nsfw": false,
            "features": [], "roles": [], "channels": [], "emojis": [],
            "stickers": [], "joined_at": null, "large": false, "lazy": false
        }))
        .unwrap()
    }

    fn fake_user(id: &str) -> User {
        serde_json::from_value(serde_json::json!({
            "id": id, "username": "cached_user",
            "discriminator": "0000", "avatar": null
        }))
        .unwrap()
    }

    fn make_ctx(settings: CacheSettings) -> FakeCtx {
        FakeCtx { http: DiscordHttpClient::new("fake_token", None, false), cache: Cache::with_settings(settings) }
    }

    #[tokio::test]
    async fn guild_returns_cached_value() {
        let ctx = make_ctx(CacheSettings::default());
        let guild = fake_guild("111222333444555666");
        ctx.cache.upsert_guild(guild);
        let id: GuildId = "111222333444555666".parse().unwrap();
        let result = ctx.guild(&id).await.expect("cache hit should succeed");
        assert_eq!(result.id, "111222333444555666");
        assert_eq!(result.name.as_deref(), Some("Cached Guild"));
    }

    #[tokio::test]
    async fn user_returns_cached_value() {
        let ctx = make_ctx(CacheSettings::default());
        ctx.cache.upsert_user(fake_user("999888777666555444"));
        let id: UserId = "999888777666555444".parse().unwrap();
        let result = ctx.user(&id).await.expect("cache hit should succeed");
        assert_eq!(result.username, "cached_user");
    }
}
