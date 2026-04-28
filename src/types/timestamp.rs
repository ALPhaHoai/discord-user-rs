//! Discord timestamp formatting
//!
//! Discord supports rendering timestamps client-side using the `<t:unix:style>`
//! markdown syntax.  Use [`FormattedTimestamp`] to produce these strings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Visual style for a [`FormattedTimestamp`].
///
/// Corresponds to the single-letter format specifiers in `<t:unix:X>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimestampStyle {
    /// `9:41 AM`
    ShortTime,
    /// `9:41:30 AM`
    LongTime,
    /// `01/20/2021`
    ShortDate,
    /// `January 20, 2021`
    LongDate,
    /// `January 20, 2021 9:41 AM` (default when no style is given)
    ShortDateTime,
    /// `Wednesday, January 20, 2021 9:41 AM`
    LongDateTime,
    /// `3 years ago` / `in 5 minutes`
    RelativeTime,
}

impl TimestampStyle {
    /// Single-letter Discord format specifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ShortTime => "t",
            Self::LongTime => "T",
            Self::ShortDate => "d",
            Self::LongDate => "D",
            Self::ShortDateTime => "f",
            Self::LongDateTime => "F",
            Self::RelativeTime => "R",
        }
    }
}

/// A Discord inline timestamp that the client renders according to the user's
/// locale and timezone.
///
/// # Example
/// ```
/// use discord_user::types::{FormattedTimestamp, TimestampStyle};
///
/// let ts = FormattedTimestamp::new(1_609_459_200, TimestampStyle::RelativeTime);
/// assert_eq!(ts.to_string(), "<t:1609459200:R>");
///
/// // From a chrono DateTime
/// use chrono::{TimeZone, Utc};
/// let dt = Utc.timestamp_opt(1_609_459_200, 0).unwrap();
/// let ts2 = FormattedTimestamp::from_datetime(dt, TimestampStyle::LongDate);
/// assert_eq!(ts2.to_string(), "<t:1609459200:D>");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormattedTimestamp {
    /// Unix timestamp in seconds.
    pub unix: i64,
    #[serde(skip)]
    pub style: Option<TimestampStyle>,
}

impl FormattedTimestamp {
    /// Create from a Unix second timestamp with an explicit style.
    pub fn new(unix: i64, style: TimestampStyle) -> Self {
        Self { unix, style: Some(style) }
    }

    /// Create from a Unix second timestamp using Discord's default style
    /// (`ShortDateTime`, rendered as `<t:unix>`).
    pub fn default_style(unix: i64) -> Self {
        Self { unix, style: None }
    }

    /// Create from a [`chrono::DateTime<Utc>`].
    pub fn from_datetime(dt: DateTime<Utc>, style: TimestampStyle) -> Self {
        Self::new(dt.timestamp(), style)
    }
}

impl FormattedTimestamp {
    /// Create a timestamp for the current moment with the given style.
    pub fn now(style: TimestampStyle) -> Self {
        Self::new(Utc::now().timestamp(), style)
    }

    /// Relative-time timestamp for the current moment (`<t:unix:R>`).
    pub fn relative_now() -> Self {
        Self::now(TimestampStyle::RelativeTime)
    }

    /// Format this timestamp with every style and return all 7 strings.
    pub fn all_styles(&self) -> [String; 7] {
        [
            FormattedTimestamp::new(self.unix, TimestampStyle::ShortTime).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::LongTime).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::ShortDate).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::LongDate).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::ShortDateTime).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::LongDateTime).to_string(),
            FormattedTimestamp::new(self.unix, TimestampStyle::RelativeTime).to_string(),
        ]
    }
}

impl std::fmt::Display for FormattedTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.style {
            Some(style) => write!(f, "<t:{}:{}>", self.unix, style.as_str()),
            None => write!(f, "<t:{}>", self.unix),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative() {
        let ts = FormattedTimestamp::new(1_609_459_200, TimestampStyle::RelativeTime);
        assert_eq!(ts.to_string(), "<t:1609459200:R>");
    }

    #[test]
    fn test_long_date() {
        let ts = FormattedTimestamp::new(1_609_459_200, TimestampStyle::LongDate);
        assert_eq!(ts.to_string(), "<t:1609459200:D>");
    }

    #[test]
    fn test_default_style() {
        let ts = FormattedTimestamp::default_style(1_609_459_200);
        assert_eq!(ts.to_string(), "<t:1609459200>");
    }

    #[test]
    fn test_from_datetime() {
        use chrono::TimeZone;
        let dt = Utc.timestamp_opt(1_609_459_200, 0).unwrap();
        let ts = FormattedTimestamp::from_datetime(dt, TimestampStyle::ShortTime);
        assert_eq!(ts.to_string(), "<t:1609459200:t>");
    }
}
