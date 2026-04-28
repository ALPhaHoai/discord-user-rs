//! Stream-based event collectors.
//!
//! Collectors let you await one or more gateway events that match a set of
//! filter criteria, with an optional timeout.  Each collector registers an
//! `EventEmitter` listener that forwards matching events over an mpsc channel;
//! the public API exposes that channel via `CollectorStream`.
//!
//! The `EventSubscription` returned by `on_event` is stored inside
//! `CollectorStream` — the listener is automatically removed when the stream
//! is dropped.
//!
//! # Example — await the next matching message
//! ```ignore
//! use discord_user::collector::MessageCollector;
//! use std::time::Duration;
//!
//! let mut col = MessageCollector::new()
//!     .author_id("123456789")
//!     .channel_id("987654321")
//!     .timeout(Duration::from_secs(30))
//!     .build(user.events().clone())
//!     .await;
//!
//! if let Some(msg) = col.next().await {
//!     println!("Got: {}", msg["content"]);
//! }
//! ```

use std::time::Duration;

use futures::Stream;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::events::{EventEmitter, EventSubscription};

// ── MessageCollector ─────────────────────────────────────────────────────────

/// Collects `MESSAGE_CREATE` events that match optional filters.
#[derive(Default)]
pub struct MessageCollector {
    author_id: Option<String>,
    channel_id: Option<String>,
    guild_id: Option<String>,
    timeout: Option<Duration>,
}

impl MessageCollector {
    /// Create a new `MessageCollector` with no filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Only yield messages from this author (user ID string).
    pub fn author_id(mut self, id: impl Into<String>) -> Self {
        self.author_id = Some(id.into());
        self
    }

    /// Only yield messages sent in this channel.
    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    /// Only yield messages from this guild.
    pub fn guild_id(mut self, id: impl Into<String>) -> Self {
        self.guild_id = Some(id.into());
        self
    }

    /// Stop yielding after this duration elapses without a new match.
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    /// Register the listener on `events` and return a `CollectorStream`.
    ///
    /// The `EventEmitter` must be cloned from the context before passing
    /// (e.g. `ctx.events().clone()`).
    pub async fn build(self, events: EventEmitter) -> CollectorStream {
        let (tx, rx) = mpsc::channel::<Value>(64);
        let author_id = self.author_id;
        let channel_id = self.channel_id;
        let guild_id = self.guild_id;
        let timeout = self.timeout;

        let subscription = events
            .on_event("MESSAGE_CREATE", move |ev| {
                let data = &ev.data;
                let a_ok = author_id.as_deref().map(|id| data["author"]["id"].as_str() == Some(id)).unwrap_or(true);
                let c_ok = channel_id.as_deref().map(|id| data["channel_id"].as_str() == Some(id)).unwrap_or(true);
                let g_ok = guild_id.as_deref().map(|id| data["guild_id"].as_str() == Some(id)).unwrap_or(true);
                if a_ok && c_ok && g_ok {
                    let _ = tx.try_send(ev.data.clone());
                }
            })
            .await;

        CollectorStream { rx, timeout, _subscription: subscription }
    }
}

// ── ReactionCollector ────────────────────────────────────────────────────────

/// Collects `MESSAGE_REACTION_ADD` events that match optional filters.
#[derive(Default)]
pub struct ReactionCollector {
    user_id: Option<String>,
    channel_id: Option<String>,
    message_id: Option<String>,
    timeout: Option<Duration>,
}

impl ReactionCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = Some(id.into());
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    /// Register the listener and return a `CollectorStream`.
    pub async fn build(self, events: EventEmitter) -> CollectorStream {
        let (tx, rx) = mpsc::channel::<Value>(64);
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let message_id = self.message_id;
        let timeout = self.timeout;

        let subscription = events
            .on_event("MESSAGE_REACTION_ADD", move |ev| {
                let data = &ev.data;
                let u_ok = user_id.as_deref().map(|id| data["user_id"].as_str() == Some(id)).unwrap_or(true);
                let c_ok = channel_id.as_deref().map(|id| data["channel_id"].as_str() == Some(id)).unwrap_or(true);
                let m_ok = message_id.as_deref().map(|id| data["message_id"].as_str() == Some(id)).unwrap_or(true);
                if u_ok && c_ok && m_ok {
                    let _ = tx.try_send(ev.data.clone());
                }
            })
            .await;

        CollectorStream { rx, timeout, _subscription: subscription }
    }
}

// ── ComponentInteractionCollector ────────────────────────────────────────────

/// Collects `INTERACTION_CREATE` events for component interactions (buttons,
/// selects) that match optional filters.
#[derive(Default)]
pub struct ComponentInteractionCollector {
    user_id: Option<String>,
    channel_id: Option<String>,
    message_id: Option<String>,
    custom_ids: Vec<String>,
    timeout: Option<Duration>,
}

impl ComponentInteractionCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.channel_id = Some(id.into());
        self
    }

    pub fn message_id(mut self, id: impl Into<String>) -> Self {
        self.message_id = Some(id.into());
        self
    }

    /// Only yield interactions where `data.custom_id` is one of these values.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_ids.push(id.into());
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    pub async fn build(self, events: EventEmitter) -> CollectorStream {
        let (tx, rx) = mpsc::channel::<Value>(64);
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let message_id = self.message_id;
        let custom_ids = self.custom_ids;
        let timeout = self.timeout;

        let subscription = events
            .on_event("INTERACTION_CREATE", move |ev| {
                let data = &ev.data;
                // Component interactions have type 3 (button) or type 5 (select)
                let interaction_type = data["type"].as_u64().unwrap_or(0);
                if interaction_type != 3 && interaction_type != 5 {
                    return;
                }
                let u_ok = user_id.as_deref().map(|id| data["user"]["id"].as_str() == Some(id) || data["member"]["user"]["id"].as_str() == Some(id)).unwrap_or(true);
                let c_ok = channel_id.as_deref().map(|id| data["channel_id"].as_str() == Some(id)).unwrap_or(true);
                let m_ok = message_id.as_deref().map(|id| data["message"]["id"].as_str() == Some(id)).unwrap_or(true);
                let cid_ok = if custom_ids.is_empty() { true } else { data["data"]["custom_id"].as_str().map(|cid| custom_ids.iter().any(|c| c == cid)).unwrap_or(false) };
                if u_ok && c_ok && m_ok && cid_ok {
                    let _ = tx.try_send(ev.data.clone());
                }
            })
            .await;

        CollectorStream { rx, timeout, _subscription: subscription }
    }
}

// ── CollectorStream
// ───────────────────────────────────────────────────────────

/// The output of any collector's `build()` call.
///
/// Use [`next`](CollectorStream::next) to await single items or
/// [`into_stream`](CollectorStream::into_stream) to drive it as a `Stream`.
///
/// The underlying event listener lives as long as this struct does — dropping
/// `CollectorStream` unsubscribes the listener automatically.
pub struct CollectorStream {
    rx: mpsc::Receiver<Value>,
    timeout: Option<Duration>,
    _subscription: EventSubscription,
}

impl CollectorStream {
    /// Await the next matching event.
    ///
    /// Returns `None` when the timeout elapses or the sender is closed.
    pub async fn next(&mut self) -> Option<Value> {
        match self.timeout {
            Some(d) => tokio::time::timeout(d, self.rx.recv()).await.ok().flatten(),
            None => self.rx.recv().await,
        }
    }

    /// Convert into a `Stream` that yields all matching events until timeout.
    pub fn into_stream(mut self) -> impl Stream<Item = Value> {
        async_stream::stream! {
            while let Some(v) = self.next().await {
                yield v;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EventEmitter;

    #[tokio::test]
    async fn message_collector_filters_author() {
        let emitter = EventEmitter::new();
        let mut col = MessageCollector::new().author_id("user_a").timeout(Duration::from_millis(100)).build(emitter.clone()).await;

        // Emit a non-matching message (different author)
        emitter
            .dispatch(crate::events::DispatchEvent {
                event_type: "MESSAGE_CREATE".to_string(),
                data: serde_json::json!({ "author": { "id": "user_b" }, "channel_id": "c1", "content": "no" }),
            })
            .await;

        // Emit a matching message
        emitter
            .dispatch(crate::events::DispatchEvent {
                event_type: "MESSAGE_CREATE".to_string(),
                data: serde_json::json!({ "author": { "id": "user_a" }, "channel_id": "c1", "content": "yes" }),
            })
            .await;

        // Allow spawned event handler tasks to run
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;

        let received = col.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap()["content"], "yes");
    }

    #[tokio::test]
    async fn collector_stream_times_out() {
        let emitter = EventEmitter::new();
        let mut col = MessageCollector::new().timeout(Duration::from_millis(10)).build(emitter.clone()).await;

        // No events dispatched — should time out
        let result = col.next().await;
        assert!(result.is_none());
    }
}
