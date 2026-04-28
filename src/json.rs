//! JSON parsing shim — conditionally backed by `serde_json` (default) or
//! `simd-json` (when the `simd_json` feature flag is enabled).
//!
//! # Usage
//! ```ignore
//! use crate::json;
//! let value: MyType = json::from_str(&text)?;
//! ```
//!
//! The feature flag is purely an implementation detail; callers always use the
//! same `json::from_str` / `json::to_string` API surface regardless of which
//! backend is active.
//!
//! # SIMD requirements
//! `simd-json` requires a CPU with AVX2 support (most x86-64 chips from 2013
//! onward).  On unsupported hardware the crate falls back automatically to a
//! scalar path.

// ── serde_json backend (default) ─────────────────────────────────────────────

#[cfg(not(feature = "simd_json"))]
pub use serde_json::{from_str, from_value, to_string, to_string_pretty, to_value, Error, Result, Value};
// ── simd-json backend ─────────────────────────────────────────────────────────
#[cfg(feature = "simd_json")]
pub use serde_json::{from_value, to_string, to_string_pretty, to_value, Error, Result, Value};

/// Deserialize `T` from a JSON string.
///
/// When the `simd_json` feature is enabled this uses `simd_json::from_slice`
/// (which mutates a temporary byte buffer for in-place parsing).  Otherwise
/// delegates to `serde_json::from_str`.
#[cfg(feature = "simd_json")]
pub fn from_str<T>(s: &str) -> serde_json::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    use serde::de::Error as _;
    // simd-json requires a mutable byte slice; we copy the string into a Vec.
    let mut bytes = s.as_bytes().to_vec();
    simd_json::from_slice::<T>(&mut bytes).map_err(|e| serde_json::Error::custom(e.to_string()))
}
