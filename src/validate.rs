//! Model-level validation — catches Discord constraint violations before HTTP.
//!
//! Call [`Validate::validate`] on a request struct before sending it.
//! Operations in this crate do this automatically where applicable.
//!
//! # Example
//! ```
//! use discord_user::{error::ModelError, validate::validate_message_content};
//!
//! let long_content = "a".repeat(2001);
//! // Validation catches the error without any HTTP request
//! match validate_message_content(&long_content) {
//!     Err(e) => assert!(matches!(e, ModelError::MessageTooLong(_))),
//!     Ok(()) => panic!("expected error"),
//! }
//! ```

use crate::error::ModelError;

/// Constraint constants matching Discord's documented limits.
pub const MAX_MESSAGE_LENGTH: usize = 2000;
pub const MAX_EMBEDS_PER_MESSAGE: usize = 10;
pub const MAX_EMBED_TOTAL_CHARS: usize = 6000;
pub const MAX_STICKERS_PER_MESSAGE: usize = 3;
pub const MIN_BULK_DELETE: usize = 2;
pub const MAX_BULK_DELETE: usize = 100;
pub const MAX_ROLE_NAME: usize = 100;
pub const MIN_ROLE_NAME: usize = 1;
pub const MAX_CHANNEL_NAME: usize = 100;
pub const MIN_CHANNEL_NAME: usize = 1;
pub const MAX_GUILD_NAME: usize = 100;
pub const MIN_GUILD_NAME: usize = 2;
pub const MAX_STICKER_NAME: usize = 30;
pub const MIN_STICKER_NAME: usize = 2;
pub const MAX_EMOJI_NAME: usize = 32;
pub const MIN_EMOJI_NAME: usize = 2;
pub const MAX_CHANNEL_TOPIC: usize = 1024;
pub const MAX_WEBHOOK_NAME: usize = 80;
pub const MIN_WEBHOOK_NAME: usize = 1;
pub const MAX_INVITE_MAX_AGE: u32 = 604800;
pub const MAX_INVITE_MAX_USES: u32 = 100;

/// Trait for request structs that can validate themselves against Discord
/// limits.
pub trait Validate {
    /// Validate this value against Discord's constraints.
    ///
    /// Returns `Ok(())` if validation passes or `Err(ModelError)` on the first
    /// constraint that is violated.  No network request is ever made if this
    /// returns an error.
    fn validate(&self) -> Result<(), ModelError>;
}

/// Free function: validate raw message content length.
pub fn validate_message_content(content: &str) -> Result<(), ModelError> {
    if content.chars().count() > MAX_MESSAGE_LENGTH {
        return Err(ModelError::MessageTooLong(content.chars().count()));
    }
    Ok(())
}

/// Free function: validate embed count for a single message.
pub fn validate_embed_count(count: usize) -> Result<(), ModelError> {
    if count > MAX_EMBEDS_PER_MESSAGE {
        return Err(ModelError::EmbedAmount(count));
    }
    Ok(())
}

/// Free function: validate sticker count for a single message.
pub fn validate_sticker_count(count: usize) -> Result<(), ModelError> {
    if count > MAX_STICKERS_PER_MESSAGE {
        return Err(ModelError::StickerAmount(count));
    }
    Ok(())
}

/// Free function: validate a bulk-delete message ID list.
pub fn validate_bulk_delete_count(count: usize) -> Result<(), ModelError> {
    if !(MIN_BULK_DELETE..=MAX_BULK_DELETE).contains(&count) {
        return Err(ModelError::BulkDeleteAmount(count));
    }
    Ok(())
}

/// Free function: validate a name against a min/max character range.
pub fn validate_name(name: &str, min: usize, max: usize) -> Result<(), ModelError> {
    let len = name.chars().count();
    if len < min {
        return Err(ModelError::NameTooShort(len, min));
    }
    if len > max {
        return Err(ModelError::NameTooLong(len, max));
    }
    Ok(())
}

/// Validate a guild name (2–100 characters).
pub fn validate_guild_name(name: &str) -> Result<(), ModelError> {
    let len = name.chars().count();
    if !(MIN_GUILD_NAME..=MAX_GUILD_NAME).contains(&len) {
        return Err(ModelError::GuildNameLength(len));
    }
    Ok(())
}

/// Validate a channel topic (0–1024 characters).
pub fn validate_channel_topic(topic: &str) -> Result<(), ModelError> {
    let len = topic.chars().count();
    if len > MAX_CHANNEL_TOPIC {
        return Err(ModelError::ChannelTopicLength(len));
    }
    Ok(())
}

/// Validate a role name (1–100 characters).
pub fn validate_role_name(name: &str) -> Result<(), ModelError> {
    let len = name.chars().count();
    if !(MIN_ROLE_NAME..=MAX_ROLE_NAME).contains(&len) {
        return Err(ModelError::RoleNameLength(len));
    }
    Ok(())
}

/// Validate a webhook name (1–80 characters).
pub fn validate_webhook_name(name: &str) -> Result<(), ModelError> {
    let len = name.chars().count();
    if !(MIN_WEBHOOK_NAME..=MAX_WEBHOOK_NAME).contains(&len) {
        return Err(ModelError::WebhookNameLength(len));
    }
    Ok(())
}

/// Validate an invite's max_age (0–604800 seconds).
pub fn validate_invite_max_age(max_age: u32) -> Result<(), ModelError> {
    if max_age > MAX_INVITE_MAX_AGE {
        return Err(ModelError::InviteMaxAge(max_age));
    }
    Ok(())
}

/// Validate an invite's max_uses (0–100).
pub fn validate_invite_max_uses(max_uses: u32) -> Result<(), ModelError> {
    if max_uses > MAX_INVITE_MAX_USES {
        return Err(ModelError::InviteMaxUses(max_uses));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_content_ok() {
        assert!(validate_message_content("hello").is_ok());
        assert!(validate_message_content(&"a".repeat(2000)).is_ok());
    }

    #[test]
    fn message_content_too_long() {
        let err = validate_message_content(&"a".repeat(2001)).unwrap_err();
        assert!(matches!(err, ModelError::MessageTooLong(2001)));
    }

    #[test]
    fn embed_count_ok() {
        assert!(validate_embed_count(10).is_ok());
        assert!(validate_embed_count(0).is_ok());
    }

    #[test]
    fn embed_count_too_many() {
        let err = validate_embed_count(11).unwrap_err();
        assert!(matches!(err, ModelError::EmbedAmount(11)));
    }

    #[test]
    fn sticker_count_too_many() {
        let err = validate_sticker_count(4).unwrap_err();
        assert!(matches!(err, ModelError::StickerAmount(4)));
    }

    #[test]
    fn bulk_delete_bounds() {
        assert!(validate_bulk_delete_count(2).is_ok());
        assert!(validate_bulk_delete_count(100).is_ok());
        assert!(matches!(validate_bulk_delete_count(1).unwrap_err(), ModelError::BulkDeleteAmount(1)));
        assert!(matches!(validate_bulk_delete_count(101).unwrap_err(), ModelError::BulkDeleteAmount(101)));
    }

    #[test]
    fn name_bounds() {
        assert!(validate_name("ab", 2, 32).is_ok());
        assert!(matches!(validate_name("a", 2, 32).unwrap_err(), ModelError::NameTooShort(1, 2)));
        assert!(matches!(validate_name(&"a".repeat(33), 2, 32).unwrap_err(), ModelError::NameTooLong(33, 32)));
    }
}
