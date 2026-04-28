//! Protobuf encoding for Discord user settings (PreloadedUserSettings proto)

use base64::{engine::general_purpose::STANDARD, Engine};

fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

fn encode_length_delimited(buf: &mut Vec<u8>, field_number: u32, data: &[u8]) {
    encode_varint(buf, ((field_number as u64) << 3) | 2);
    encode_varint(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

/// Wire type 1 (64-bit fixed) — required for timestamp fields 4 and 5
fn encode_fixed64(buf: &mut Vec<u8>, field_number: u32, value: u64) {
    encode_varint(buf, ((field_number as u64) << 3) | 1);
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Custom status settings for Discord
#[derive(Debug, Clone)]
pub struct CustomStatus {
    pub text: String,
    pub emoji_id: Option<u64>,
    pub emoji_name: Option<String>,
    /// Unix timestamp in milliseconds; field 4, fixed64
    pub expires_at_ms: Option<u64>,
    /// Unix timestamp in milliseconds; field 5, fixed64
    pub created_at_ms: Option<u64>,
}

impl CustomStatus {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), emoji_id: None, emoji_name: None, expires_at_ms: None, created_at_ms: None }
    }

    pub fn with_expiry(mut self, expires_at_ms: u64) -> Self {
        self.expires_at_ms = Some(expires_at_ms);
        self
    }

    pub fn with_created_at(mut self, created_at_ms: u64) -> Self {
        self.created_at_ms = Some(created_at_ms);
        self
    }

    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        if !self.text.is_empty() {
            encode_length_delimited(&mut buf, 1, self.text.as_bytes());
        }

        if let Some(emoji_id) = self.emoji_id {
            if emoji_id != 0 {
                encode_varint(&mut buf, 2u64 << 3);
                encode_varint(&mut buf, emoji_id);
            }
        }

        if let Some(ref emoji_name) = self.emoji_name {
            if !emoji_name.is_empty() {
                encode_length_delimited(&mut buf, 3, emoji_name.as_bytes());
            }
        }

        // Fields 4 and 5 must be fixed64 (wire type 1), not varint
        if let Some(expires) = self.expires_at_ms {
            encode_fixed64(&mut buf, 4, expires);
        }
        if let Some(created) = self.created_at_ms {
            encode_fixed64(&mut buf, 5, created);
        }

        buf
    }
}

/// User status (online, idle, dnd, invisible)
#[derive(Debug, Clone)]
pub struct StatusSettings {
    pub status: String,
    pub custom_status: Option<CustomStatus>,
}

impl StatusSettings {
    pub fn new(status: impl Into<String>) -> Self {
        Self { status: status.into(), custom_status: None }
    }

    pub fn with_custom_status(mut self, custom: CustomStatus) -> Self {
        self.custom_status = Some(custom);
        self
    }

    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        if !self.status.is_empty() {
            let mut status_wrapper = Vec::new();
            encode_length_delimited(&mut status_wrapper, 1, self.status.as_bytes());
            encode_length_delimited(&mut buf, 1, &status_wrapper);
        }

        if let Some(ref custom) = self.custom_status {
            let custom_bytes = custom.encode();
            if !custom_bytes.is_empty() {
                encode_length_delimited(&mut buf, 2, &custom_bytes);
            }
        }

        buf
    }
}

/// PreloadedUserSettings protobuf encoder — status lives at field 11
pub struct PreloadedUserSettings {
    pub status: Option<StatusSettings>,
}

impl PreloadedUserSettings {
    pub fn with_status(status: StatusSettings) -> Self {
        Self { status: Some(status) }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        if let Some(ref status) = self.status {
            let status_bytes = status.encode();
            if !status_bytes.is_empty() {
                encode_length_delimited(&mut buf, 11, &status_bytes);
            }
        }

        buf
    }

    /// Encode to base64 string (for API request)
    pub fn to_base64(&self) -> String {
        STANDARD.encode(self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verified against a captured Discord PATCH /users/@me/settings-proto/1 request.
    // Layout (no emoji, no timestamps):
    //   [0]     5A        field 11, wire-type 2 (outer PreloadedUserSettings)
    //   [1]     17        length 23
    //   [2]     0A        field 1, wire-type 2 (StatusSettings.status wrapper)
    //   [3]     08        length 8
    //   [4]     0A        field 1, wire-type 2 (StringValue.value)
    //   [5]     06        length 6
    //   [6..12] "online"
    //   [12]    12        field 2, wire-type 2 (CustomStatus embedded message)
    //   [13]    0B        length 11
    //   [14]    0A        field 1, wire-type 2 (CustomStatus.text)
    //   [15]    09        length 9
    //   [16..25] "studyphim"
    #[test]
    fn test_encode_matches_wire_format() {
        let settings = PreloadedUserSettings::with_status(
            StatusSettings::new("online").with_custom_status(CustomStatus::new("studyphim")),
        );
        let bytes = settings.encode();

        assert_eq!(bytes[0], 0x5A, "outer field tag must be field 11 (0x5A)");
        assert_eq!(bytes[2], 0x0A, "StatusSettings field 1 tag");
        assert_eq!(&bytes[6..12], b"online");
        assert_eq!(bytes[12], 0x12, "CustomStatus field 2 tag");
        assert_eq!(&bytes[16..25], b"studyphim");
    }
}
