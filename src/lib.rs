//! Discord Control - A Discord self-bot client library for Rust
//!
//! This library provides functionality to interact with Discord as a user
//! account, including WebSocket gateway connection, message operations, and
//! event handling.
//!
//! # Warning
//! This library implements Discord self-botting which violates Discord's Terms
//! of Service. Use at your own risk.

#[cfg(feature = "builder")]
pub mod builder;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "cache")]
pub mod cache_http;
pub mod client;
#[cfg(feature = "collector")]
pub mod collector;
pub mod components;
pub mod content_safe;
pub mod discord_user;
pub mod error;
pub mod events;
pub mod fmt;
#[cfg(feature = "framework")]
pub mod framework;
pub mod gateway;
pub mod json;
pub mod mention;
pub mod message_builder;
pub mod modal;
pub mod operations;
pub mod permissions;
pub mod proto;
pub mod route;
pub mod typed_events;
pub mod typemap;
pub mod types;
pub mod utils;
pub mod validate;

pub use context::DiscordContext;
pub use discord_user::{ConnectionInfo, DiscordUser, DiscordUserBuilder};
pub use error::{DiscordError, Result};
pub use events::DispatchEvent;
pub use message_builder::{EmbedBuilder, MessageBuilder};
pub use typed_events::TypedEvent;
pub use types::*;

pub mod context;
