//! Operations module - provides specialized operation traits for DiscordUser
//!
//! This module splits the DiscordUser operations into focused, maintainable
//! modules:
//! - `message` - Message sending, editing, deleting, and reactions
//! - `relationship` - Friend requests and relationship management
//! - `guild` - Guild roles, invites, members, and stickers
//! - `channel` - DM channels, voice status, and channel/guild info
//! - `status` - User presence and custom status

mod automod;
mod channel;
mod guild;
mod message;
mod relationship;
mod scheduled_event;
mod slash_command;
mod soundboard;
mod stage;
mod status;
mod thread;
mod voice;
mod webhook;

pub use automod::*;
pub use channel::*;
pub use guild::*;
pub use message::*;
pub use relationship::*;
pub use scheduled_event::*;
pub use slash_command::*;
pub use soundboard::*;
pub use stage::*;
pub use status::*;
pub use thread::*;
pub use voice::*;
pub use webhook::*;
