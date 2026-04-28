//! TypeMap — type-safe shared state for event handlers.
//!
//! Provides a `TypeMap` that maps user-defined key types to associated values,
//! allowing different event handlers to share strongly-typed data without
//! unsafe casts or stringly-typed hashmaps.
//!
//! # Pattern
//!
//! 1. Define a key type and implement [`TypeMapKey`] to declare the value type.
//! 2. Insert the value at startup via [`TypeMap::insert`].
//! 3. In any handler, retrieve the value with [`TypeMap::get`] /
//!    [`TypeMap::get_mut`].
//!
//! # Example
//! ```
//! use std::sync::{
//!     atomic::{AtomicU64, Ordering},
//!     Arc,
//! };
//!
//! use discord_user::typemap::{TypeMap, TypeMapKey};
//!
//! struct MessageCounter;
//! impl TypeMapKey for MessageCounter {
//!     type Value = Arc<AtomicU64>;
//! }
//!
//! let mut map = TypeMap::new();
//! map.insert::<MessageCounter>(Arc::new(AtomicU64::new(0)));
//!
//! let counter = map.get::<MessageCounter>().unwrap();
//! counter.fetch_add(1, Ordering::Relaxed);
//! assert_eq!(
//!     map.get::<MessageCounter>().unwrap().load(Ordering::Relaxed),
//!     1
//! );
//! ```

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Implement this trait on a marker type to associate it with a value type in
/// a [`TypeMap`].
///
/// # Example
/// ```
/// use discord_user::typemap::TypeMapKey;
///
/// struct BotPrefix;
/// impl TypeMapKey for BotPrefix {
///     type Value = String;
/// }
/// ```
pub trait TypeMapKey: 'static {
    /// The concrete type stored under this key.
    type Value: Send + Sync + 'static;
}

/// A type-safe heterogeneous map.
///
/// Internally stores boxed `Any` values keyed by [`TypeId`].  The
/// [`TypeMapKey`] trait connects a zero-sized marker type to a concrete value
/// type so callers never need to write casts.
#[derive(Default)]
pub struct TypeMap {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl TypeMap {
    /// Create an empty `TypeMap`.
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a value for key `K`, replacing any existing value.
    pub fn insert<K: TypeMapKey>(&mut self, value: K::Value) {
        self.map.insert(TypeId::of::<K>(), Box::new(value));
    }

    /// Get a shared reference to the value for key `K`, or `None` if not set.
    pub fn get<K: TypeMapKey>(&self) -> Option<&K::Value> {
        self.map.get(&TypeId::of::<K>()).and_then(|b| b.downcast_ref::<K::Value>())
    }

    /// Get an exclusive reference to the value for key `K`, or `None` if not
    /// set.
    pub fn get_mut<K: TypeMapKey>(&mut self) -> Option<&mut K::Value> {
        self.map.get_mut(&TypeId::of::<K>()).and_then(|b| b.downcast_mut::<K::Value>())
    }

    /// Remove the value for key `K`, returning it if it was present.
    pub fn remove<K: TypeMapKey>(&mut self) -> Option<K::Value> {
        self.map.remove(&TypeId::of::<K>()).and_then(|b| b.downcast::<K::Value>().ok().map(|v| *v))
    }

    /// Returns `true` if a value has been inserted for key `K`.
    pub fn contains_key<K: TypeMapKey>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<K>())
    }

    /// Number of entries in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    };

    use super::*;

    struct Counter;
    impl TypeMapKey for Counter {
        type Value = Arc<AtomicU64>;
    }

    struct Greeting;
    impl TypeMapKey for Greeting {
        type Value = String;
    }

    #[test]
    fn insert_and_get() {
        let mut map = TypeMap::new();
        map.insert::<Greeting>("hello".to_string());
        assert_eq!(map.get::<Greeting>().unwrap(), "hello");
    }

    #[test]
    fn missing_key_returns_none() {
        let map = TypeMap::new();
        assert!(map.get::<Greeting>().is_none());
    }

    #[test]
    fn atomic_shared_across_clones() {
        let mut map = TypeMap::new();
        map.insert::<Counter>(Arc::new(AtomicU64::new(0)));
        // Simulate two handlers sharing the same Arc
        let c1 = Arc::clone(map.get::<Counter>().unwrap());
        let c2 = Arc::clone(map.get::<Counter>().unwrap());
        c1.fetch_add(1, Ordering::Relaxed);
        c2.fetch_add(1, Ordering::Relaxed);
        assert_eq!(map.get::<Counter>().unwrap().load(Ordering::Relaxed), 2);
    }

    #[test]
    fn remove_returns_value() {
        let mut map = TypeMap::new();
        map.insert::<Greeting>("world".to_string());
        let val = map.remove::<Greeting>().unwrap();
        assert_eq!(val, "world");
        assert!(!map.contains_key::<Greeting>());
    }

    #[test]
    fn multiple_keys_independent() {
        let mut map = TypeMap::new();
        map.insert::<Greeting>("hi".to_string());
        map.insert::<Counter>(Arc::new(AtomicU64::new(42)));
        assert_eq!(map.get::<Greeting>().unwrap(), "hi");
        assert_eq!(map.get::<Counter>().unwrap().load(Ordering::Relaxed), 42);
        assert_eq!(map.len(), 2);
    }
}
