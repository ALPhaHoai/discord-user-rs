//! Miscellaneous utility helpers.

use crate::error::{DiscordError, Result};

/// Validate a Discord token format before attempting to connect.
///
/// Discord issues two token shapes:
/// - **User / bot token**: `<base64_user_id>.<timestamp_b64>.<hmac>` — three
///   dot-separated segments where the first segment decodes to a numeric user
///   ID and the other two are base64url strings.
/// - **MFA token**: `mfa.<long_base64url_string>` — two segments where the
///   first is the literal `mfa`.
///
/// This function checks structural validity only — it does **not** make any
/// HTTP call to confirm the token is still active.
///
/// # Errors
/// Returns [`DiscordError::InvalidToken`] when the token is structurally
/// invalid (wrong number of segments, non-base64 characters, etc.).
///
/// # Example
/// ```
/// use discord_user::utils::validate_token;
///
/// // Well-formed user token → Ok
/// // validate_token("NzY4NDQwNDEwOTM4NzUyMzI0.G_abc1.XYZ_abc123").unwrap();
///
/// // Malformed token → Err
/// assert!(validate_token("not-a-token").is_err());
/// assert!(validate_token("").is_err());
/// ```
pub fn validate_token(token: &str) -> Result<()> {
    let token = token.trim();

    if token.is_empty() {
        return Err(DiscordError::InvalidToken);
    }

    let parts: Vec<&str> = token.splitn(3, '.').collect();

    match parts.as_slice() {
        // MFA token: "mfa.<base64url>"
        [first, rest] if *first == "mfa" => {
            if rest.is_empty() || !is_base64url_like(rest) {
                return Err(DiscordError::InvalidToken);
            }
            Ok(())
        }
        // Standard token: "<base64_id>.<ts_b64>.<hmac>"
        [id_b64, ts_b64, hmac] => {
            if id_b64.is_empty() || ts_b64.is_empty() || hmac.is_empty() {
                return Err(DiscordError::InvalidToken);
            }
            // First segment must be valid base64 that decodes to a numeric user ID
            if !is_base64url_like(id_b64) {
                return Err(DiscordError::InvalidToken);
            }
            // Decode and check it looks like a snowflake (digits only)
            match base64::Engine::decode(&base64::engine::general_purpose::STANDARD_NO_PAD, id_b64) {
                Ok(bytes) => {
                    let s = String::from_utf8(bytes).map_err(|_| DiscordError::InvalidToken)?;
                    if s.is_empty() || !s.chars().all(|c| c.is_ascii_digit()) {
                        return Err(DiscordError::InvalidToken);
                    }
                }
                Err(_) => return Err(DiscordError::InvalidToken),
            }
            if !is_base64url_like(ts_b64) || !is_base64url_like(hmac) {
                return Err(DiscordError::InvalidToken);
            }
            Ok(())
        }
        _ => Err(DiscordError::InvalidToken),
    }
}

/// Returns `true` if every character in `s` is a valid base64url or standard
/// base64 character (letters, digits, `+`, `/`, `-`, `_`, `=`).
fn is_base64url_like(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '-' | '_' | '='))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_token_is_invalid() {
        assert!(validate_token("").is_err());
    }

    #[test]
    fn no_dots_is_invalid() {
        assert!(validate_token("notavalidtoken").is_err());
    }

    #[test]
    fn two_parts_non_mfa_is_invalid() {
        assert!(validate_token("abc.def").is_err());
    }

    #[test]
    fn mfa_token_valid() {
        // "mfa." followed by any base64url characters
        assert!(validate_token("mfa.abcABC0123456789_-").is_ok());
    }

    #[test]
    fn mfa_token_empty_payload_invalid() {
        assert!(validate_token("mfa.").is_err());
    }

    #[test]
    fn well_formed_user_token() {
        // Build a real-looking token: base64("123456789") + dummy segments
        use base64::Engine;
        let id_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode("123456789");
        let token = format!("{}.Gc5abc.SomeHmacHere", id_b64);
        assert!(validate_token(&token).is_ok());
    }

    #[test]
    fn non_numeric_id_is_invalid() {
        use base64::Engine;
        let id_b64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode("not-a-number");
        let token = format!("{}.Gc5abc.SomeHmacHere", id_b64);
        assert!(validate_token(&token).is_err());
    }
}
