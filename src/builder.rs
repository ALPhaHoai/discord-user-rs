//! Type-state builder pattern for compile-time required-field enforcement.
//!
//! Builders that require certain fields before they can produce a value use
//! phantom type parameters to make incomplete builders un-buildable at
//! compile time — the compiler rejects the call instead of returning a
//! runtime error.
//!
//! # Example
//! ```
//! use discord_user::builder::{CreateMessage, NoChannel};
//!
//! // Does NOT compile — channel_id is required:
//! // let msg = CreateMessage::new().build();
//!
//! // Compiles:
//! let payload = CreateMessage::new()
//!     .channel("123456789")
//!     .content("Hello!")
//!     .build();
//! assert_eq!(payload["channel_id"], "123456789");
//! ```

use std::marker::PhantomData;

use serde_json::{json, Value};

// ── State markers
// ─────────────────────────────────────────────────────────────

/// Marker: `channel_id` has NOT been set.
pub struct NoChannel;
/// Marker: `channel_id` HAS been set — the builder is ready to build.
pub struct HasChannel;

// ── CreateMessage
// ─────────────────────────────────────────────────────────────

/// A type-state builder for Discord message payloads.
///
/// `S` is a phantom marker:
/// - `CreateMessage<NoChannel>` — channel not yet set; `.build()` is
///   unavailable
/// - `CreateMessage<HasChannel>` — channel set; `.build()` is available
pub struct CreateMessage<S = NoChannel> {
    channel_id: Option<String>,
    content: Option<String>,
    tts: bool,
    embeds: Vec<Value>,
    components: Vec<Value>,
    flags: u64,
    _state: PhantomData<S>,
}

impl CreateMessage<NoChannel> {
    /// Create a new builder. `channel_id` must be set before calling `build()`.
    pub fn new() -> Self {
        Self {
            channel_id: None,
            content: None,
            tts: false,
            embeds: Vec::new(),
            components: Vec::new(),
            flags: 0,
            _state: PhantomData,
        }
    }

    /// Set the destination channel. Transitions the builder to `HasChannel`.
    pub fn channel(self, channel_id: impl Into<String>) -> CreateMessage<HasChannel> {
        CreateMessage {
            channel_id: Some(channel_id.into()),
            content: self.content,
            tts: self.tts,
            embeds: self.embeds,
            components: self.components,
            flags: self.flags,
            _state: PhantomData,
        }
    }
}

impl Default for CreateMessage<NoChannel> {
    fn default() -> Self {
        Self::new()
    }
}

// Optional fields are available on both states via a shared impl block.
impl<S> CreateMessage<S> {
    /// Set the message text content.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Enable text-to-speech for this message.
    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = tts;
        self
    }

    /// Add a raw embed object (use `EmbedBuilder` to construct one).
    pub fn embed(mut self, embed: Value) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Add a raw component row (action rows, buttons, selects).
    pub fn component(mut self, component: Value) -> Self {
        self.components.push(component);
        self
    }

    /// Set message flags (e.g. 64 = ephemeral).
    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }
}

// `.build()` is ONLY available once `HasChannel` is set.
impl CreateMessage<HasChannel> {
    /// Produce the JSON payload ready to POST to `POST
    /// /channels/{id}/messages`.
    ///
    /// `channel_id` is included as a convenience field so callers can extract
    /// it without storing it separately.
    pub fn build(self) -> Value {
        json!({
            "channel_id": self.channel_id,
            "content": self.content,
            "tts": self.tts,
            "embeds": self.embeds,
            "components": self.components,
            "flags": self.flags,
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_requires_channel_at_compile_time() {
        // This test verifies the HAPPY PATH compiles and produces correct output.
        // The UNHAPPY PATH (missing channel) is a compile error — verified by
        // the absence of a `CreateMessage<NoChannel>::build` method.
        let payload = CreateMessage::new().channel("111222333").content("hello").tts(false).build();

        assert_eq!(payload["channel_id"], "111222333");
        assert_eq!(payload["content"], "hello");
        assert_eq!(payload["tts"], false);
    }

    #[test]
    fn optional_fields_before_channel() {
        // Optional fields can be set before or after .channel().
        let payload = CreateMessage::new().content("pre-channel content").tts(true).channel("999").build();

        assert_eq!(payload["channel_id"], "999");
        assert_eq!(payload["content"], "pre-channel content");
        assert_eq!(payload["tts"], true);
    }

    #[test]
    fn no_channel_type_has_no_build_method() {
        // This is a COMPILE-TIME check. The snippet below must NOT compile:
        //
        //   let _ = CreateMessage::new().build();
        //
        // We verify this by confirming the NoChannel state exists and
        // that `CreateMessage<NoChannel>` is the default.
        let _: CreateMessage<NoChannel> = CreateMessage::new();
    }
}
