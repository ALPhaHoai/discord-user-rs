//! Discord message formatting helpers.
//!
//! These functions produce the markdown-like syntax Discord uses to render
//! formatted text. They mirror serenity's `MessageBuilder` push_* API but as
//! free functions so they compose naturally with `format!` and `String::new()`.
//!
//! # Example
//! ```
//! use discord_user::fmt;
//!
//! let msg = format!(
//!     "{} caused a {} in {}",
//!     fmt::bold("Alice"),
//!     fmt::code("panic!"),
//!     fmt::channel_mention(123456789)
//! );
//! // → "**Alice** caused a `panic!` in <#123456789>"
//! ```

/// Wrap text in **bold** markers.
pub fn bold(text: &str) -> String {
    format!("**{}**", text)
}

/// Wrap text in *italic* markers.
pub fn italic(text: &str) -> String {
    format!("*{}*", text)
}

/// Wrap text in __underline__ markers.
pub fn underline(text: &str) -> String {
    format!("__{}__", text)
}

/// Wrap text in ~~strikethrough~~ markers.
pub fn strike(text: &str) -> String {
    format!("~~{}~~", text)
}

/// Wrap text in ||spoiler|| markers.
pub fn spoiler(text: &str) -> String {
    format!("||{}||", text)
}

/// Wrap text in `inline code` backticks.
pub fn code(text: &str) -> String {
    format!("`{}`", text)
}

/// Wrap text in a fenced code block with an optional language hint.
///
/// ```rust
/// use discord_user::fmt;
/// assert_eq!(
///     fmt::codeblock("fn main() {}", Some("rust")),
///     "```rust\nfn main() {}\n```"
/// );
/// ```
pub fn codeblock(text: &str, language: Option<&str>) -> String {
    format!("```{}\n{}\n```", language.unwrap_or(""), text)
}

/// Prefix each line with `> ` to produce a block quote.
pub fn quote(text: &str) -> String {
    text.lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n")
}

/// Produce a multi-line block quote with `>>> ` prefix (all following lines
/// are quoted until end of message).
pub fn quote_block(text: &str) -> String {
    format!(">>> {}", text)
}

/// Mention a user: `<@user_id>`.
pub fn user_mention(user_id: u64) -> String {
    format!("<@{}>", user_id)
}

/// Mention a role: `<@&role_id>`.
pub fn role_mention(role_id: u64) -> String {
    format!("<@&{}>", role_id)
}

/// Mention a channel: `<#channel_id>`.
pub fn channel_mention(channel_id: u64) -> String {
    format!("<#{}>", channel_id)
}

/// Mention everyone in the channel: `@everyone`.
pub fn everyone() -> &'static str {
    "@everyone"
}

/// Mention all online members: `@here`.
pub fn here() -> &'static str {
    "@here"
}

/// Render a custom emoji: `<:name:id>` or `<a:name:id>` for animated.
pub fn custom_emoji(name: &str, id: u64, animated: bool) -> String {
    if animated {
        format!("<a:{}:{}>", name, id)
    } else {
        format!("<:{}:{}>", name, id)
    }
}

/// Escape Discord markdown characters in `text` so they render literally.
///
/// Escapes: `\`, `*`, `_`, `~`, `|`, `` ` ``, `>`, `[`, `]`.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        match ch {
            '\\' | '*' | '_' | '~' | '|' | '`' | '>' | '[' | ']' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Combine bold + italic: `***text***`.
pub fn bold_italic(text: &str) -> String {
    format!("***{}***", text)
}

/// Combine underline + bold: `__**text**__`.
pub fn underline_bold(text: &str) -> String {
    format!("__**{}**__", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        assert_eq!(bold("hi"), "**hi**");
    }
    #[test]
    fn test_italic() {
        assert_eq!(italic("hi"), "*hi*");
    }
    #[test]
    fn test_code() {
        assert_eq!(code("x"), "`x`");
    }
    #[test]
    fn test_spoiler() {
        assert_eq!(spoiler("hi"), "||hi||");
    }
    #[test]
    fn test_codeblock_no_lang() {
        assert_eq!(codeblock("x", None), "```\nx\n```");
    }
    #[test]
    fn test_codeblock_lang() {
        assert_eq!(codeblock("x", Some("rust")), "```rust\nx\n```");
    }
    #[test]
    fn test_quote() {
        assert_eq!(quote("a\nb"), "> a\n> b");
    }
    #[test]
    fn test_escape() {
        assert_eq!(escape("**bold**"), "\\*\\*bold\\*\\*");
    }
    #[test]
    fn test_user_mention() {
        assert_eq!(user_mention(123), "<@123>");
    }
    #[test]
    fn test_channel_mention() {
        assert_eq!(channel_mention(456), "<#456>");
    }
    #[test]
    fn test_custom_emoji() {
        assert_eq!(custom_emoji("wave", 789, false), "<:wave:789>");
    }
    #[test]
    fn test_custom_emoji_animated() {
        assert_eq!(custom_emoji("dance", 789, true), "<a:dance:789>");
    }
}
