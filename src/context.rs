use crate::{client::DiscordHttpClient, events::EventEmitter, gateway::Gateway};

/// Core context trait providing access to Discord services
pub trait DiscordContext {
    /// Get HTTP client reference
    fn http(&self) -> &DiscordHttpClient;
    /// Get events emitter reference
    fn events(&self) -> &EventEmitter;
    /// Get gateway reference
    fn gateway(&self) -> Option<&Gateway>;
}
