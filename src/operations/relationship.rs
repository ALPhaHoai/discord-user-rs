//! Relationship operations for DiscordUser

use serde_json::json;

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> RelationshipOps for T {}

/// Extension trait providing relationship operations
#[allow(async_fn_in_trait)]
pub trait RelationshipOps: DiscordContext {
    /// Fetch the current user's full relationship list (friends, blocked users,
    /// pending outgoing requests, and incoming requests).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn get_my_relationship(&self) -> Result<Vec<Relationship>> {
        self.http().get(Route::GetRelationships).await
    }

    /// Add or update a relationship with a user.
    ///
    /// Use `RelationshipType::Friend` to send a friend request, or
    /// `RelationshipType::Blocked` to block the user.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn add_relationship(&self, user_id: &UserId, type_: RelationshipType) -> Result<()> {
        let payload = json!({ "type": type_ as u8 });
        self.http().put(Route::AddRelationship { user_id: user_id.get() }, payload).await
    }

    /// Remove a relationship with a user (unfriend, unblock, or cancel a
    /// pending request).
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure.
    async fn remove_relationship(&self, user_id: &UserId) -> Result<()> {
        self.http().delete(Route::RemoveRelationship { user_id: user_id.get() }).await
    }

    /// Send a friend request by username (and optional discriminator for legacy
    /// accounts).
    ///
    /// For pomelo (new-style) accounts, pass `discriminator: None`.
    /// For legacy accounts with a 4-digit tag, pass the discriminator number.
    ///
    /// # Errors
    /// Returns [`DiscordError::Http`] on HTTP failure, or if the username is
    /// not found.
    async fn send_friend_request(&self, username: &str, discriminator: Option<u16>) -> Result<()> {
        let mut payload = json!({ "username": username });
        if let Some(disc) = discriminator {
            payload["discriminator"] = json!(disc.to_string());
        }

        self.http().post(Route::GetRelationships, payload).await // Actually POST to /users/@me/relationships
    }
}
