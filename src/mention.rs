//! Mention and ID parsing utilities.
//!
//! Parse Discord mention syntax into typed ID wrappers.
//!
//! # Example
//! ```
//! use discord_user::mention::*;
//!
//! let uid = parse_user_mention("<@123456789012345678>").unwrap();
//! assert_eq!(uid.get(), 123456789012345678);
//!
//! let rid = parse_role_mention("<@&987654321098765432>").unwrap();
//! assert_eq!(rid.get(), 987654321098765432);
//!
//! let cid = parse_channel_mention("<#111222333444555666>").unwrap();
//! assert_eq!(cid.get(), 111222333444555666);
//! ```

use crate::types::{ChannelId, EmojiId, RoleId, UserId, WebhookId};

/// Parse a user mention `<@id>` or `<@!id>` (nickname variant).
/// Returns `None` if the string doesn't match the expected format.
pub fn parse_user_mention(s: &str) -> Option<UserId> {
    let inner = strip_wrapping(s, "<@", ">")?;
    // Strip optional '!' for nick mentions
    let digits = inner.strip_prefix('!').unwrap_or(inner);
    digits.parse().ok()
}

/// Parse a role mention `<@&id>`.
pub fn parse_role_mention(s: &str) -> Option<RoleId> {
    let digits = strip_wrapping(s, "<@&", ">")?;
    digits.parse().ok()
}

/// Parse a channel mention `<#id>`.
pub fn parse_channel_mention(s: &str) -> Option<ChannelId> {
    let digits = strip_wrapping(s, "<#", ">")?;
    digits.parse().ok()
}

/// Parse a custom emoji mention `<:name:id>` or `<a:name:id>` (animated).
/// Returns `(name, EmojiId, animated)`.
pub fn parse_emoji(s: &str) -> Option<(String, EmojiId, bool)> {
    let (animated, inner) = if s.starts_with("<a:") { (true, strip_wrapping(s, "<a:", ">")?) } else { (false, strip_wrapping(s, "<:", ">")?) };
    let mut parts = inner.splitn(2, ':');
    let name = parts.next()?.to_string();
    let id: EmojiId = parts.next()?.parse().ok()?;
    Some((name, id, animated))
}

/// Parse a Discord invite URL or code.
///
/// Accepts:
/// - `https://discord.gg/CODE`
/// - `https://discord.com/invite/CODE`
/// - bare `CODE` (alphanumeric, 2–30 chars)
pub fn parse_invite(s: &str) -> Option<String> {
    let s = s.trim();
    for prefix in &["https://discord.gg/", "https://discord.com/invite/", "http://discord.gg/"] {
        if let Some(code) = s.strip_prefix(prefix) {
            let code = code.trim_end_matches('/');
            if is_valid_invite_code(code) {
                return Some(code.to_string());
            }
        }
    }
    // Bare code
    if is_valid_invite_code(s) {
        return Some(s.to_string());
    }
    None
}

/// Parse a `username#discriminator` tag (legacy format, e.g. `Alice#1234`).
/// Returns `(username, discriminator)` where discriminator is 4 ASCII digits.
pub fn parse_user_tag(s: &str) -> Option<(String, String)> {
    let hash = s.rfind('#')?;
    let username = s[..hash].to_string();
    let disc = &s[hash + 1..];
    if disc.len() == 4 && disc.chars().all(|c| c.is_ascii_digit()) {
        Some((username, disc.to_string()))
    } else {
        None
    }
}

/// Parse a webhook URL `https://discord.com/api/webhooks/{id}/{token}`.
/// Returns `(WebhookId, token_string)`.
pub fn parse_webhook_url(url: &str) -> Option<(WebhookId, String)> {
    const PREFIX: &str = "https://discord.com/api/webhooks/";
    let rest = url.trim().strip_prefix(PREFIX)?;
    let mut parts = rest.splitn(2, '/');
    let id: WebhookId = parts.next()?.parse().ok()?;
    let token = parts.next()?.trim_end_matches('/').to_string();
    if token.is_empty() {
        None
    } else {
        Some((id, token))
    }
}

/// Parse a raw snowflake string into a `UserId`.
pub fn parse_user_id(s: &str) -> Option<UserId> {
    s.trim().parse().ok()
}

/// Parse a raw snowflake string into a `ChannelId`.
pub fn parse_channel_id(s: &str) -> Option<ChannelId> {
    s.trim().parse().ok()
}

/// Parse a raw snowflake string into a `RoleId`.
pub fn parse_role_id(s: &str) -> Option<RoleId> {
    s.trim().parse().ok()
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn strip_wrapping<'a>(s: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    s.strip_prefix(prefix)?.strip_suffix(suffix)
}

fn is_valid_invite_code(s: &str) -> bool {
    let len = s.len();
    (2..=30).contains(&len) && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_mention() {
        let id = parse_user_mention("<@123456789012345678>").unwrap();
        assert_eq!(id.get(), 123456789012345678);
    }

    #[test]
    fn user_nick_mention() {
        let id = parse_user_mention("<@!123456789012345678>").unwrap();
        assert_eq!(id.get(), 123456789012345678);
    }

    #[test]
    fn role_mention() {
        let id = parse_role_mention("<@&987654321098765432>").unwrap();
        assert_eq!(id.get(), 987654321098765432);
    }

    #[test]
    fn channel_mention() {
        let id = parse_channel_mention("<#111222333444555666>").unwrap();
        assert_eq!(id.get(), 111222333444555666);
    }

    #[test]
    fn emoji_static() {
        let (name, id, animated) = parse_emoji("<:wave:749054660769218631>").unwrap();
        assert_eq!(name, "wave");
        assert_eq!(id.get(), 749054660769218631);
        assert!(!animated);
    }

    #[test]
    fn emoji_animated() {
        let (name, _id, animated) = parse_emoji("<a:dance:749054660769218631>").unwrap();
        assert_eq!(name, "dance");
        assert!(animated);
    }

    #[test]
    fn invite_url() {
        assert_eq!(parse_invite("https://discord.gg/rust"), Some("rust".to_string()));
        assert_eq!(parse_invite("https://discord.com/invite/rust"), Some("rust".to_string()));
        assert_eq!(parse_invite("rust"), Some("rust".to_string()));
    }

    #[test]
    fn user_tag() {
        let (name, disc) = parse_user_tag("Alice#1234").unwrap();
        assert_eq!(name, "Alice");
        assert_eq!(disc, "1234");
    }

    #[test]
    fn webhook_url() {
        let url = "https://discord.com/api/webhooks/123456789012345678/abc-TOKEN";
        let (id, token) = parse_webhook_url(url).unwrap();
        assert_eq!(id.get(), 123456789012345678);
        assert_eq!(token, "abc-TOKEN");
    }

    #[test]
    fn invalid_mention_returns_none() {
        assert!(parse_user_mention("not a mention").is_none());
        assert!(parse_role_mention("<@123>").is_none()); // missing &
        assert!(parse_channel_mention("").is_none());
    }
}
