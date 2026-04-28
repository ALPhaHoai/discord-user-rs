//! Event system for Discord dispatch events

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};

use dashmap::{DashMap, DashSet};
use serde_json::Value;

/// A dispatch event from the gateway
#[derive(Debug, Clone)]
pub struct DispatchEvent {
    pub event_type: String,
    pub data: Value,
}

/// Type alias for event callback
pub type EventCallback = Arc<dyn Fn(DispatchEvent) + Send + Sync>;

/// Event listener with unique ID
#[derive(Clone)]
struct EventListener {
    id: String,
    callback: EventCallback,
}

/// Counter for unique listener IDs
static LISTENER_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_listener_id() -> String {
    let id = LISTENER_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("listener_{}", id)
}

/// Event emitter managing subscriptions to Discord events
pub struct EventEmitter {
    /// Event-specific listeners: event_type -> listeners
    /// Using DashMap for high-concurrency access (sharded locking)
    private_events: Arc<DashMap<String, Vec<EventListener>>>,
    /// Listeners that receive all events
    any_events: Arc<RwLock<Vec<EventListener>>>,
    /// Listeners for unhandled events
    unhandled_events: Arc<RwLock<Vec<EventListener>>>,
    /// Set of handled event types
    handled_types: Arc<DashSet<String>>,
}

/// RAII subscription guard for event listeners
///
/// When this struct is dropped, the listener is automatically removed.
pub struct EventSubscription {
    id: String,
    emitter: EventEmitter,
}

impl EventSubscription {
    /// Get the listener ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Detach the subscription, preventing automatic removal on drop
    /// Returns the listener ID
    pub fn detach(self) -> String {
        let id = self.id.clone();
        std::mem::forget(self);
        id
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        self.emitter.remove_listener(&self.id);
    }
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            private_events: Arc::new(DashMap::new()),
            any_events: Arc::new(RwLock::new(Vec::new())),
            unhandled_events: Arc::new(RwLock::new(Vec::new())),
            handled_types: Arc::new(DashSet::new()),
        }
    }

    /// Register a listener for a specific event
    ///
    /// Returns a subscription guard that removes the listener when dropped
    pub async fn on_event<F>(&self, event_name: &str, callback: F) -> EventSubscription
    where
        F: Fn(DispatchEvent) + Send + Sync + 'static,
    {
        let id = generate_listener_id();
        let listener = EventListener { id: id.clone(), callback: Arc::new(callback) };

        // DashMap handles locking internally per shard
        self.private_events.entry(event_name.to_string()).or_default().push(listener);

        self.handled_types.insert(event_name.to_string());

        EventSubscription { id, emitter: self.clone() }
    }

    /// Register a listener for multiple events (space-separated)
    pub async fn on_events<F>(&self, event_names: &str, callback: F) -> Vec<EventSubscription>
    where
        F: Fn(DispatchEvent) + Send + Sync + Clone + 'static,
    {
        let mut subscriptions = Vec::new();
        for name in event_names.split_whitespace() {
            let sub = self.on_event(name, callback.clone()).await;
            subscriptions.push(sub);
        }
        subscriptions
    }

    /// Register a listener for all events ("firehose")
    pub async fn on_any_event<F>(&self, callback: F) -> EventSubscription
    where
        F: Fn(DispatchEvent) + Send + Sync + 'static,
    {
        let id = generate_listener_id();
        let listener = EventListener { id: id.clone(), callback: Arc::new(callback) };

        self.any_events.write().unwrap_or_else(|e| e.into_inner()).push(listener);
        EventSubscription { id, emitter: self.clone() }
    }

    /// Register a listener for unhandled events
    pub async fn on_unhandled_event<F>(&self, callback: F) -> EventSubscription
    where
        F: Fn(DispatchEvent) + Send + Sync + 'static,
    {
        let id = generate_listener_id();
        let listener = EventListener { id: id.clone(), callback: Arc::new(callback) };

        self.unhandled_events.write().unwrap_or_else(|e| e.into_inner()).push(listener);
        EventSubscription { id, emitter: self.clone() }
    }

    /// Remove a listener by ID (synchronous — safe to call from Drop)
    pub fn remove_listener(&self, listener_id: &str) -> bool {
        // Check private events (DashMap is already synchronous)
        for mut r in self.private_events.iter_mut() {
            let listeners = r.value_mut();
            if let Some(pos) = listeners.iter().position(|l| l.id == listener_id) {
                listeners.remove(pos);
                return true;
            }
        }

        // Check any events — acquire write lock once to avoid TOCTOU race
        {
            let mut any = self.any_events.write().unwrap_or_else(|e| e.into_inner());
            if let Some(pos) = any.iter().position(|l| l.id == listener_id) {
                any.remove(pos);
                return true;
            }
        }

        // Check unhandled events — same pattern
        {
            let mut unhandled = self.unhandled_events.write().unwrap_or_else(|e| e.into_inner());
            if let Some(pos) = unhandled.iter().position(|l| l.id == listener_id) {
                unhandled.remove(pos);
                return true;
            }
        }

        false
    }

    /// Remove a listener by ID
    pub async fn off_event(&self, listener_id: &str) -> bool {
        self.remove_listener(listener_id)
    }

    /// Remove all listeners for a specific event by name (space-separated names
    /// supported)
    pub async fn off_event_by_name(&self, event_names: &str) {
        for name in event_names.split_whitespace() {
            self.private_events.remove(name);
            self.handled_types.remove(name);
        }
    }

    /// Remove all listeners for a specific event
    pub async fn off_all(&self, event_name: &str) {
        self.private_events.remove(event_name);
        self.handled_types.remove(event_name);
    }

    /// Dispatch an event to all registered listeners.
    ///
    /// Each listener callback is spawned as a named tokio task, making them
    /// visible in `tokio-console` under the label
    /// `dispatch::event_handler::{event_type}`.
    pub async fn dispatch(&self, event: DispatchEvent) {
        let event_type = event.event_type.clone();

        // 1. Dispatch to event-specific listeners.
        let specific_listeners = self.private_events.get(&event_type).map(|l| l.clone());
        if let Some(listeners) = specific_listeners {
            for listener in listeners {
                let ev = event.clone();
                let task_name = format!("dispatch::event_handler::{}", event_type);
                let _ = tokio::task::Builder::new().name(&task_name).spawn(async move {
                    (listener.callback)(ev);
                });
            }
        }

        // 2. Dispatch to any-event listeners.
        let any_listeners = self.any_events.read().unwrap_or_else(|e| e.into_inner()).clone();
        for listener in any_listeners {
            let ev = event.clone();
            let _ = tokio::task::Builder::new().name("dispatch::event_handler::any").spawn(async move {
                (listener.callback)(ev);
            });
        }

        // 3. Dispatch to unhandled if no specific handler was registered.
        if !self.handled_types.contains(&event_type) {
            let unhandled_listeners = self.unhandled_events.read().unwrap_or_else(|e| e.into_inner()).clone();
            for listener in unhandled_listeners {
                let ev = event.clone();
                let _ = tokio::task::Builder::new().name("dispatch::event_handler::unhandled").spawn(async move {
                    (listener.callback)(ev);
                });
            }
        }
    }

    /// Check if there are any listeners for an event
    pub async fn has_listeners(&self, event_name: &str) -> bool {
        self.private_events.get(event_name).map(|l| !l.is_empty()).unwrap_or(false)
    }

    /// Get count of listeners for an event
    pub async fn listener_count(&self, event_name: &str) -> usize {
        self.private_events.get(event_name).map(|l| l.len()).unwrap_or(0)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        Self {
            private_events: Arc::clone(&self.private_events),
            any_events: Arc::clone(&self.any_events),
            unhandled_events: Arc::clone(&self.unhandled_events),
            handled_types: Arc::clone(&self.handled_types),
        }
    }
}
