//! Content safety and mention sanitization.
//!
//! Use [`ContentSafeOptions`] to configure which mention types to neutralize,
//! then call [`content_safe()`] to sanitize a message string before displaying
//! or re-posting it.
//!
//! All substitutions insert a zero-width space (`\u{200B}`) after the `@` so
//! the mention is broken but still human-readable.
//!
//! # Example
//! ```
//! use discord_user::content_safe::{content_safe, ContentSafeOptions};
//!
//! let opts = ContentSafeOptions::default(); // cleans everything
//! let cleaned = content_safe("Hey @everyone, check <@123456789>!", &opts, &[]);
//! assert!(!cleaned.contains("@everyone"));
//! ```

use std::collections::HashMap;

/// Configuration for [`content_safe()`].
#[derive(Debug, Clone)]
pub struct ContentSafeOptions {
    /// Replace `@everyone` with `@\u{200B}everyone`.
    pub clean_everyone: bool,
    /// Replace `@here` with `@\u{200B}here`.
    pub clean_here: bool,
    /// Replace user mentions `<@id>` / `<@!id>` with a display name (if
    /// a lookup map is provided) or strip them to `@[unknown]`.
    pub clean_user: bool,
    /// Replace role mentions `<@&id>` with `@[role]`.
    pub clean_role: bool,
    /// Replace channel mentions `<#id>` with `#[channel]`.
    pub clean_channel: bool,
}

impl Default for ContentSafeOptions {
    fn default() -> Self {
        Self { clean_everyone: true, clean_here: true, clean_user: true, clean_role: true, clean_channel: true }
    }
}

impl ContentSafeOptions {
    /// Create options that clean nothing.
    pub fn none() -> Self {
        Self { clean_everyone: false, clean_here: false, clean_user: false, clean_role: false, clean_channel: false }
    }

    /// Enable/disable `@everyone` cleaning.
    pub fn clean_everyone(mut self, v: bool) -> Self {
        self.clean_everyone = v;
        self
    }
    /// Enable/disable `@here` cleaning.
    pub fn clean_here(mut self, v: bool) -> Self {
        self.clean_here = v;
        self
    }
    /// Enable/disable user mention cleaning.
    pub fn clean_user(mut self, v: bool) -> Self {
        self.clean_user = v;
        self
    }
    /// Enable/disable role mention cleaning.
    pub fn clean_role(mut self, v: bool) -> Self {
        self.clean_role = v;
        self
    }
    /// Enable/disable channel mention cleaning.
    pub fn clean_channel(mut self, v: bool) -> Self {
        self.clean_channel = v;
        self
    }
}

/// Sanitize `content` according to `opts`.
///
/// # Arguments
/// * `content` - The raw message string.
/// * `opts`    - Which mention types to neutralize.
/// * `users`   - Optional `(user_id_str, display_name)` pairs used to replace
///   user mentions with human-readable names.  If a mention ID is not found in
///   this map the fallback `@[unknown]` is used.
pub fn content_safe(content: &str, opts: &ContentSafeOptions, users: &[(String, String)]) -> String {
    let user_map: HashMap<&str, &str> = users.iter().map(|(id, name)| (id.as_str(), name.as_str())).collect();

    let mut out = content.to_string();

    // @everyone / @here — insert zero-width space to break the ping
    if opts.clean_everyone {
        out = out.replace("@everyone", "@\u{200B}everyone");
    }
    if opts.clean_here {
        out = out.replace("@here", "@\u{200B}here");
    }

    // User mentions: <@123> and <@!123> (nickname mention)
    if opts.clean_user {
        out = replace_mentions(&out, r"<@!?(\d+)>", |id| if let Some(name) = user_map.get(id) { format!("@{}", name) } else { "@[unknown]".to_string() });
    }

    // Role mentions: <@&123>
    if opts.clean_role {
        out = replace_mentions(&out, r"<@&(\d+)>", |_id| "@[role]".to_string());
    }

    // Channel mentions: <#123>
    if opts.clean_channel {
        out = replace_mentions(&out, r"<#(\d+)>", |_id| "#[channel]".to_string());
    }

    out
}

/// Walk `text` replacing every capture of `pattern` using
/// `replacement_fn(capture_group_1)`. Uses a simple hand-rolled scanner to
/// avoid pulling in the `regex` crate.
fn replace_mentions<F>(text: &str, pattern: &str, replacement_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    // Determine the literal prefix and suffix from the pattern so we can scan
    // without regex.  The patterns we use are fixed-structure:
    //   "<@!?(\d+)>"  → prefix="<@" or "<@!", suffix=">"
    //   "<@&(\d+)>"   → prefix="<@&", suffix=">"
    //   "<#(\d+)>"    → prefix="<#",  suffix=">"
    let (prefix, has_optional_bang, suffix) = match pattern {
        r"<@!?(\d+)>" => ("<@", true, ">"),
        r"<@&(\d+)>" => ("<@&", false, ">"),
        r"<#(\d+)>" => ("<#", false, ">"),
        _ => return text.to_string(),
    };

    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut result = String::with_capacity(text.len());
    let mut i = 0;

    while i < n {
        // Try to match prefix at position i
        if text[i..].starts_with(prefix) {
            let after_prefix = i + prefix.len();
            // Handle optional '!' for user nicknames
            let digit_start = if has_optional_bang && text[after_prefix..].starts_with('!') { after_prefix + 1 } else { after_prefix };

            // Collect digit run
            let mut j = digit_start;
            while j < n && bytes[j].is_ascii_digit() {
                j += 1;
            }

            // Check closing suffix
            if j > digit_start && text[j..].starts_with(suffix) {
                let id = &text[digit_start..j];
                result.push_str(&replacement_fn(id));
                i = j + suffix.len();
                continue;
            }
        }
        // No match: emit the byte as-is and advance one char
        if let Some(ch) = text[i..].chars().next() {
            result.push(ch);
            i += ch.len_utf8();
        } else {
            // This shouldn't happen based on the bounds check, but safe fallback
            result.push_str(&text[i..]);
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_everyone_and_here() {
        let opts = ContentSafeOptions::default();
        let out = content_safe("hey @everyone and @here!", &opts, &[]);
        assert!(!out.contains("@everyone"));
        assert!(!out.contains("@here"));
        assert!(out.contains("@\u{200B}everyone"));
        assert!(out.contains("@\u{200B}here"));
    }

    #[test]
    fn cleans_user_mention_with_lookup() {
        let opts = ContentSafeOptions::default();
        let users = vec![("123".to_string(), "Alice".to_string())];
        let out = content_safe("Hello <@123>!", &opts, &users);
        assert_eq!(out, "Hello @Alice!");
    }

    #[test]
    fn cleans_user_nick_mention() {
        let opts = ContentSafeOptions::default();
        let out = content_safe("Hello <@!456>!", &opts, &[]);
        assert_eq!(out, "Hello @[unknown]!");
    }

    #[test]
    fn cleans_role_mention() {
        let opts = ContentSafeOptions::default();
        let out = content_safe("Hey <@&789>!", &opts, &[]);
        assert_eq!(out, "Hey @[role]!");
    }

    #[test]
    fn cleans_channel_mention() {
        let opts = ContentSafeOptions::default();
        let out = content_safe("See <#111>.", &opts, &[]);
        assert_eq!(out, "See #[channel].");
    }

    #[test]
    fn none_opts_leaves_content_unchanged() {
        let opts = ContentSafeOptions::none();
        let input = "hey @everyone <@123> <@&456> <#789>";
        assert_eq!(content_safe(input, &opts, &[]), input);
    }
}
