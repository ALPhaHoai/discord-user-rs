//! Colour newtype for Discord embed colours

use serde::{Deserialize, Serialize};

/// A Discord colour value (24-bit RGB packed into a u32).
///
/// Can be constructed from a hex literal (`Colour(0xFF5733)`), from RGB
/// components (`Colour::from_rgb(255, 87, 51)`), or via the named constants
/// (`Colour::BLURPLE`, `Colour::RED`, etc.).
///
/// `EmbedBuilder::color()` accepts any `Into<Colour>`, so a plain `u32` still
/// works as before.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Colour(pub u32);

impl Colour {
    // ── Discord / community named colours ────────────────────────────────────

    /// Discord Blurple (#5865F2)
    pub const BLURPLE: Self = Self(0x5865F2);
    /// Discord Green (#57F287)
    pub const GREEN: Self = Self(0x57F287);
    /// Discord Yellow (#FEE75C)
    pub const YELLOW: Self = Self(0xFEE75C);
    /// Discord Fuchsia (#EB459E)
    pub const FUCHSIA: Self = Self(0xEB459E);
    /// Discord Red (#ED4245)
    pub const RED: Self = Self(0xED4245);
    /// Discord White (#FFFFFF)
    pub const WHITE: Self = Self(0xFFFFFF);
    /// Discord Dark Grey (#2C2F33)
    pub const DARK_GREY: Self = Self(0x2C2F33);
    /// Discord Light Grey (#99AAB5)
    pub const LIGHT_GREY: Self = Self(0x99AAB5);
    /// Pure black (#000000)
    pub const BLACK: Self = Self(0x000000);
    /// Discord Dark Teal (#1ABC9C)
    pub const DARK_TEAL: Self = Self(0x1ABC9C);
    /// Discord Teal (#11806A)
    pub const TEAL: Self = Self(0x11806A);
    /// Discord Dark Green (#1F8B4C)
    pub const DARK_GREEN: Self = Self(0x1F8B4C);
    /// Discord Dark Blue (#206694)
    pub const DARK_BLUE: Self = Self(0x206694);
    /// Discord Purple (#9B59B6)
    pub const PURPLE: Self = Self(0x9B59B6);
    /// Discord Dark Purple (#71368A)
    pub const DARK_PURPLE: Self = Self(0x71368A);
    /// Discord Magenta (#E91E63)
    pub const MAGENTA: Self = Self(0xE91E63);
    /// Discord Dark Magenta (#AD1457)
    pub const DARK_MAGENTA: Self = Self(0xAD1457);
    /// Discord Gold (#F1C40F)
    pub const GOLD: Self = Self(0xF1C40F);
    /// Discord Dark Gold (#C27C0E)
    pub const DARK_GOLD: Self = Self(0xC27C0E);
    /// Discord Orange (#E67E22)
    pub const ORANGE: Self = Self(0xE67E22);
    /// Discord Dark Orange (#A84300)
    pub const DARK_ORANGE: Self = Self(0xA84300);

    // ── Constructors ─────────────────────────────────────────────────────────

    /// Construct a colour from its red, green, and blue components.
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Red component (0–255).
    pub const fn r(self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Green component (0–255).
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Blue component (0–255).
    pub const fn b(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Return the raw u32 value.
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl From<u32> for Colour {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<Colour> for u32 {
    fn from(c: Colour) -> Self {
        c.0
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:06X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_rgb_roundtrip() {
        let c = Colour::from_rgb(255, 87, 51);
        assert_eq!(c.r(), 255);
        assert_eq!(c.g(), 87);
        assert_eq!(c.b(), 51);
        assert_eq!(c.0, 0xFF5733);
    }

    #[test]
    fn test_named_constants() {
        assert_eq!(Colour::BLURPLE.0, 0x5865F2);
        assert_eq!(Colour::RED.r(), 0xED);
        assert_eq!(Colour::GREEN.g(), 0xF2);
    }

    #[test]
    fn test_display() {
        assert_eq!(Colour::BLURPLE.to_string(), "#5865F2");
    }

    #[test]
    fn test_u32_conversion() {
        let c: Colour = 0xABCDEFu32.into();
        assert_eq!(c.value(), 0xABCDEF);
        let v: u32 = c.into();
        assert_eq!(v, 0xABCDEF);
    }
}
