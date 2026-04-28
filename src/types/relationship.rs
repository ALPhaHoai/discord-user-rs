//! Discord relationship types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::User;
use crate::RelationshipType;

/// Relationship between users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
    #[serde(default)]
    pub user: Option<User>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub since: Option<DateTime<Utc>>,
    #[serde(default)]
    pub user_ignored: bool,
    #[serde(default)]
    pub is_spam_request: bool,
}

impl Relationship {
    /// Check if this is a friend relationship
    pub fn is_friend(&self) -> bool {
        self.relationship_type == RelationshipType::Friend
    }

    /// Check if this is a pending incoming friend request
    pub fn is_pending_incoming(&self) -> bool {
        self.relationship_type == RelationshipType::PendingIncoming
    }

    /// Check if this is a pending outgoing friend request
    pub fn is_pending_outgoing(&self) -> bool {
        self.relationship_type == RelationshipType::PendingOutgoing
    }

    /// Check if user is blocked
    pub fn is_blocked(&self) -> bool {
        self.relationship_type == RelationshipType::Blocked
    }

    /// Get the user ID for this relationship
    pub fn get_user_id(&self) -> Option<&str> {
        self.user.as_ref().map(|u| u.id.as_str())
    }
}
