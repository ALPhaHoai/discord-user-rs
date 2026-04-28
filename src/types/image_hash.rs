//! ImageHash newtype for Discord CDN asset hashes

use serde::{Deserialize, Serialize};

/// A Discord CDN image hash (e.g. `"a_1234abcd"` for animated, `"1234abcd"` for
/// static).
///
/// Animated hashes begin with `a_`. Use [`is_animated`](ImageHash::is_animated)
/// to detect them, and the `url_*` helpers to build correctly-typed CDN URLs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImageHash(pub String);

/// Image size must be a power of two between 16 and 4096.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
    Size16 = 16,
    Size32 = 32,
    Size64 = 64,
    Size128 = 128,
    Size256 = 256,
    Size512 = 512,
    Size1024 = 1024,
    Size2048 = 2048,
    Size4096 = 4096,
}

const CDN_BASE: &str = "https://cdn.discordapp.com";

impl ImageHash {
    /// Create a new ImageHash from a raw hash string.
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Returns `true` if this hash represents an animated image (starts with
    /// `a_`).
    pub fn is_animated(&self) -> bool {
        self.0.starts_with("a_")
    }

    /// Raw hash string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Build a user avatar URL.
    ///
    /// Returns a `.gif` URL for animated hashes and `.webp` otherwise.
    /// Pass `None` for `size` to omit the query parameter.
    pub fn user_avatar_url(&self, user_id: u64, size: Option<ImageSize>) -> String {
        let ext = if self.is_animated() { "gif" } else { "webp" };
        Self::with_size(format!("{CDN_BASE}/avatars/{user_id}/{}.{ext}", self.0), size)
    }

    /// Build a guild icon URL.
    pub fn guild_icon_url(&self, guild_id: u64, size: Option<ImageSize>) -> String {
        let ext = if self.is_animated() { "gif" } else { "webp" };
        Self::with_size(format!("{CDN_BASE}/icons/{guild_id}/{}.{ext}", self.0), size)
    }

    /// Build a guild banner URL.
    pub fn guild_banner_url(&self, guild_id: u64, size: Option<ImageSize>) -> String {
        let ext = if self.is_animated() { "gif" } else { "webp" };
        Self::with_size(format!("{CDN_BASE}/banners/{guild_id}/{}.{ext}", self.0), size)
    }

    fn with_size(url: String, size: Option<ImageSize>) -> String {
        match size {
            Some(s) => format!("{}?size={}", url, s as u32),
            None => url,
        }
    }
}

impl std::fmt::Display for ImageHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ImageHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ImageHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animated_detection() {
        assert!(ImageHash::new("a_1234abcd").is_animated());
        assert!(!ImageHash::new("1234abcd").is_animated());
    }

    #[test]
    fn test_avatar_url_animated() {
        let hash = ImageHash::new("a_abc123");
        let url = hash.user_avatar_url(12345, None);
        assert!(url.contains(".gif"));
        assert!(url.contains("/avatars/12345/"));
    }

    #[test]
    fn test_avatar_url_static_with_size() {
        let hash = ImageHash::new("abc123");
        let url = hash.user_avatar_url(12345, Some(ImageSize::Size256));
        assert!(url.contains(".webp"));
        assert!(url.ends_with("?size=256"));
    }
}
