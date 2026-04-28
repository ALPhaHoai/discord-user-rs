//! Type-safe ID wrappers for Discord resources
//!
//! These newtype wrappers provide compile-time safety to prevent
//! accidentally passing the wrong ID type to API methods.
//!
//! Validation is performed when constructing IDs via `new()`.

use std::{error::Error, fmt, num::NonZeroU64};

use chrono::{DateTime, TimeZone, Utc};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Discord's snowflake epoch: 2015-01-01T00:00:00Z in milliseconds
const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;

/// Error returned when parsing an invalid Snowflake ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSnowflakeError(pub String);

impl fmt::Display for InvalidSnowflakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid snowflake ID '{}': must be a numeric non-zero value", self.0)
    }
}

impl Error for InvalidSnowflakeError {}

macro_rules! impl_id_type {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub NonZeroU64);

        impl $name {
            /// Create a new validated ID from a u64
            /// Panics if the value is 0
            pub fn new(id: u64) -> Self {
                Self(NonZeroU64::new(id).expect(concat!("Invalid ", stringify!($name), ": 0")))
            }

            /// Create a new ID without validation
            pub fn new_unchecked(id: u64) -> Self {
                Self(NonZeroU64::new(id).unwrap_or(NonZeroU64::new(1).unwrap()))
            }

            /// Get the ID as a u64
            pub fn get(&self) -> u64 {
                self.0.get()
            }

            /// Extract the creation timestamp from this snowflake ID.
            ///
            /// Discord encodes the creation time in the top 42 bits of every snowflake
            /// (milliseconds since 2015-01-01T00:00:00Z).
            pub fn created_at(&self) -> DateTime<Utc> {
                let ms = (self.0.get() >> 22) + DISCORD_EPOCH_MS;
                Utc.timestamp_millis_opt(ms as i64).single().expect("snowflake timestamp out of range")
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.get())
            }
        }

        impl From<u64> for $name {
            fn from(id: u64) -> Self {
                Self::new(id)
            }
        }

        impl std::str::FromStr for $name {
            type Err = InvalidSnowflakeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let val = s.parse::<u64>().map_err(|_| InvalidSnowflakeError(s.to_string()))?;
                NonZeroU64::new(val).map(Self).ok_or_else(|| InvalidSnowflakeError(s.to_string()))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.get().to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct IdVisitor;

                impl<'de> Visitor<'de> for IdVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a string containing a 64-bit unsigned integer or a 64-bit unsigned integer")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let val = value.parse::<u64>().map_err(de::Error::custom)?;
                        NonZeroU64::new(val).map($name).ok_or_else(|| de::Error::custom("ID cannot be 0"))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        NonZeroU64::new(value).map($name).ok_or_else(|| de::Error::custom("ID cannot be 0"))
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        if value <= 0 {
                            return Err(de::Error::custom("ID must be positive and non-zero"));
                        }
                        NonZeroU64::new(value as u64).map($name).ok_or_else(|| de::Error::custom("ID cannot be 0"))
                    }
                }

                deserializer.deserialize_any(IdVisitor)
            }
        }
    };
}

impl_id_type!(ChannelId, "A channel ID wrapper for type safety");
impl_id_type!(UserId, "A user ID wrapper for type safety");
impl_id_type!(MessageId, "A message ID wrapper for type safety");
impl_id_type!(GuildId, "A guild (server) ID wrapper for type safety");
impl_id_type!(RoleId, "A role ID wrapper for type safety");
impl_id_type!(EmojiId, "An emoji ID wrapper for type safety");
impl_id_type!(WebhookId, "A webhook ID wrapper for type safety");
impl_id_type!(ApplicationId, "An application ID wrapper for type safety");
impl_id_type!(InteractionId, "An interaction ID wrapper for type safety");
impl_id_type!(StickerId, "A sticker ID wrapper for type safety");
impl_id_type!(ScheduledEventId, "A guild scheduled event ID wrapper for type safety");
impl_id_type!(AutoModerationRuleId, "An auto-moderation rule ID wrapper for type safety");
impl_id_type!(SoundboardSoundId, "A soundboard sound ID wrapper for type safety");
impl_id_type!(CommandId, "An application command ID wrapper for type safety");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_snowflake() {
        let id_val = 12345678901234567;
        let id_str = "12345678901234567";
        let id = ChannelId::new(id_val);
        assert_eq!(id.get(), id_val);
        assert_eq!(id.to_string(), id_str);
    }

    #[test]
    fn test_role_id() {
        let id_val = 98765432109876543;
        let id = RoleId::new(id_val);
        assert_eq!(id.get(), id_val);
    }

    #[test]
    #[should_panic(expected = "Invalid ChannelId: 0")]
    fn test_invalid_snowflake_zero() {
        ChannelId::new(0);
    }

    #[test]
    fn test_deserialize_str() {
        let json = r#""12345678901234567""#;
        let id: ChannelId = serde_json::from_str(json).unwrap();
        assert_eq!(id.get(), 12345678901234567);
    }

    #[test]
    fn test_deserialize_num() {
        let json = r#"12345678901234567"#;
        let id: ChannelId = serde_json::from_str(json).unwrap();
        assert_eq!(id.get(), 12345678901234567);
    }

    #[test]
    fn test_serialize() {
        let id = ChannelId::new(12345678901234567);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""12345678901234567""#);
    }

    #[test]
    fn test_created_at() {
        // Snowflake 175928847299117063 was created at 2016-04-30T11:18:25.796Z
        // (known reference value from Discord docs)
        let id = MessageId::new(175_928_847_299_117_063);
        let ts = id.created_at();
        assert_eq!(ts.format("%Y-%m-%d").to_string(), "2016-04-30");
    }
}
