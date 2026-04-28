//! Text (prefix) command framework.
//!
//! A lightweight dispatcher for prefix-based commands (e.g. `!ping`).
//! Register handlers with [`CommandFramework::command`], then pass incoming
//! message content to [`CommandFramework::dispatch`].
//!
//! # Example
//! ```
//! use discord_user::framework::{CommandContext, CommandFramework};
//!
//! let mut fw = CommandFramework::new("!");
//!
//! fw.command("ping", |ctx| {
//!     println!("Pong! from {}", ctx.author_id);
//! });
//!
//! // Simulate an incoming message
//! let handled = fw.dispatch("!ping", "user123", "chan1");
//! assert!(handled);
//!
//! let unhandled = fw.dispatch("!unknown", "user123", "chan1");
//! assert!(!unhandled);
//! ```

use std::{collections::HashMap, sync::Arc};

/// Context passed to every command handler.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The command name that was matched (without prefix, e.g. `"ping"`).
    pub command: String,
    /// Arguments following the command name, split on whitespace.
    pub args: Vec<String>,
    /// Raw full message content (including prefix + command + args).
    pub raw: String,
    /// User ID of the message author.
    pub author_id: String,
    /// Channel ID where the message was sent.
    pub channel_id: String,
}

type Handler = Arc<dyn Fn(CommandContext) + Send + Sync>;
type BeforeHook = Arc<dyn Fn(&CommandContext) -> bool + Send + Sync>;
type AfterHook = Arc<dyn Fn(&CommandContext) + Send + Sync>;
type UnrecognisedHook = Arc<dyn Fn(&CommandContext) + Send + Sync>;

/// A prefix-based text command dispatcher.
///
/// Commands are matched case-insensitively by default.
pub struct CommandFramework {
    prefix: String,
    commands: HashMap<String, Handler>,
    before: Option<BeforeHook>,
    after: Option<AfterHook>,
    on_unrecognised: Option<UnrecognisedHook>,
    case_insensitive: bool,
    allow_dm: bool,
}

impl CommandFramework {
    /// Create a new framework with the given command prefix (e.g. `"!"`).
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            commands: HashMap::new(),
            before: None,
            after: None,
            on_unrecognised: None,
            case_insensitive: true,
            allow_dm: true,
        }
    }

    /// Whether to match command names case-insensitively (default: `true`).
    pub fn case_insensitive(mut self, value: bool) -> Self {
        self.case_insensitive = value;
        self
    }

    /// Whether to respond to commands sent in DM channels (default: `true`).
    pub fn allow_dm(mut self, value: bool) -> Self {
        self.allow_dm = value;
        self
    }

    /// Register a handler for a command name (without prefix).
    ///
    /// If `case_insensitive` is `true` (the default), the name is stored
    /// lower-cased and matched case-insensitively at dispatch time.
    pub fn command<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(CommandContext) + Send + Sync + 'static,
    {
        let key = if self.case_insensitive { name.to_lowercase() } else { name.to_string() };
        self.commands.insert(key, Arc::new(handler));
    }

    /// Set a `before` hook that runs before every command.
    ///
    /// Return `true` to allow the command to proceed, `false` to abort.
    pub fn before<F>(&mut self, hook: F)
    where
        F: Fn(&CommandContext) -> bool + Send + Sync + 'static,
    {
        self.before = Some(Arc::new(hook));
    }

    /// Set an `after` hook that runs after every command that was allowed by
    /// the `before` hook.
    pub fn after<F>(&mut self, hook: F)
    where
        F: Fn(&CommandContext) + Send + Sync + 'static,
    {
        self.after = Some(Arc::new(hook));
    }

    /// Set a handler for messages that start with the prefix but don't match
    /// any registered command.
    pub fn on_unrecognised<F>(&mut self, hook: F)
    where
        F: Fn(&CommandContext) + Send + Sync + 'static,
    {
        self.on_unrecognised = Some(Arc::new(hook));
    }

    /// Dispatch a message.
    ///
    /// Returns `true` if the message was handled by a registered command,
    /// `false` if it didn't start with the prefix or wasn't recognised.
    ///
    /// # Arguments
    /// * `content`    – Raw message text.
    /// * `author_id`  – Snowflake string of the message author.
    /// * `channel_id` – Snowflake string of the channel.
    pub fn dispatch(&self, content: &str, author_id: &str, channel_id: &str) -> bool {
        // Must start with prefix
        let rest = match content.strip_prefix(&self.prefix) {
            Some(r) => r.trim_start(),
            None => return false,
        };

        // Split into command name + args
        let mut parts = rest.splitn(2, char::is_whitespace);
        let cmd_raw = match parts.next() {
            Some(c) if !c.is_empty() => c,
            _ => return false,
        };
        let args_str = parts.next().unwrap_or("").trim();
        let args: Vec<String> = if args_str.is_empty() { vec![] } else { args_str.split_whitespace().map(str::to_string).collect() };

        let key = if self.case_insensitive { cmd_raw.to_lowercase() } else { cmd_raw.to_string() };

        let ctx = CommandContext {
            command: key.clone(),
            args,
            raw: content.to_string(),
            author_id: author_id.to_string(),
            channel_id: channel_id.to_string(),
        };

        if let Some(handler) = self.commands.get(&key) {
            // Run before hook — abort if it returns false
            if let Some(before) = &self.before {
                if !before(&ctx) {
                    return false;
                }
            }
            handler(ctx.clone());
            if let Some(after) = &self.after {
                after(&ctx);
            }
            true
        } else {
            // Unrecognised command
            if let Some(hook) = &self.on_unrecognised {
                hook(&ctx);
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    };

    use super::*;

    #[test]
    fn basic_dispatch() {
        let fired = Arc::new(AtomicBool::new(false));
        let fired2 = Arc::clone(&fired);
        let mut fw = CommandFramework::new("!");
        fw.command("ping", move |_ctx| {
            fired2.store(true, Ordering::Relaxed);
        });
        assert!(fw.dispatch("!ping", "u1", "c1"));
        assert!(fired.load(Ordering::Relaxed));
    }

    #[test]
    fn unrecognised_command() {
        let fired = Arc::new(AtomicBool::new(false));
        let fired2 = Arc::clone(&fired);
        let mut fw = CommandFramework::new("!");
        fw.on_unrecognised(move |_ctx| {
            fired2.store(true, Ordering::Relaxed);
        });
        assert!(!fw.dispatch("!unknown", "u1", "c1"));
        assert!(fired.load(Ordering::Relaxed));
    }

    #[test]
    fn no_prefix_not_dispatched() {
        let mut fw = CommandFramework::new("!");
        fw.command("ping", |_| {});
        assert!(!fw.dispatch("ping", "u1", "c1"));
    }

    #[test]
    fn args_are_split() {
        let args_out = Arc::new(std::sync::Mutex::new(vec![]));
        let args_clone = Arc::clone(&args_out);
        let mut fw = CommandFramework::new("!");
        fw.command("echo", move |ctx| {
            *args_clone.lock().unwrap() = ctx.args.clone();
        });
        fw.dispatch("!echo hello world", "u1", "c1");
        assert_eq!(*args_out.lock().unwrap(), vec!["hello", "world"]);
    }

    #[test]
    fn before_hook_can_abort() {
        let fired = Arc::new(AtomicBool::new(false));
        let fired2 = Arc::clone(&fired);
        let mut fw = CommandFramework::new("!");
        fw.before(|_ctx| false); // always abort
        fw.command("ping", move |_| {
            fired2.store(true, Ordering::Relaxed);
        });
        assert!(!fw.dispatch("!ping", "u1", "c1"));
        assert!(!fired.load(Ordering::Relaxed));
    }

    #[test]
    fn after_hook_fires() {
        let count = Arc::new(AtomicUsize::new(0));
        let c2 = Arc::clone(&count);
        let mut fw = CommandFramework::new("!");
        fw.command("ping", |_| {});
        fw.after(move |_ctx| {
            c2.fetch_add(1, Ordering::Relaxed);
        });
        fw.dispatch("!ping", "u1", "c1");
        assert_eq!(count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn case_insensitive_match() {
        let fired = Arc::new(AtomicBool::new(false));
        let fired2 = Arc::clone(&fired);
        let mut fw = CommandFramework::new("!");
        fw.command("ping", move |_| {
            fired2.store(true, Ordering::Relaxed);
        });
        assert!(fw.dispatch("!PING", "u1", "c1"));
        assert!(fired.load(Ordering::Relaxed));
    }
}
