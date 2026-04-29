//! Protobuf encoding for Discord user settings (PreloadedUserSettings proto)

use base64::{engine::general_purpose::STANDARD, Engine};

/// Errors that can occur while decoding a Discord settings-proto response.
#[derive(Debug, thiserror::Error)]
pub enum ProtoDecodeError {
    #[error("invalid base64 in settings field")]
    InvalidBase64,
    #[error("buffer ended mid-field")]
    Truncated,
    #[error("varint overflowed 64 bits")]
    VarintOverflow,
    #[error("unsupported wire type: {0}")]
    UnknownWireType(u8),
    #[error("invalid utf-8 in string field")]
    InvalidUtf8,
}

fn read_varint(bytes: &[u8], pos: &mut usize) -> Result<u64, ProtoDecodeError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if *pos >= bytes.len() {
            return Err(ProtoDecodeError::Truncated);
        }
        let b = bytes[*pos];
        *pos += 1;
        value |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 64 {
            return Err(ProtoDecodeError::VarintOverflow);
        }
    }
}

fn read_fixed64(bytes: &[u8], pos: &mut usize) -> Result<u64, ProtoDecodeError> {
    if *pos + 8 > bytes.len() {
        return Err(ProtoDecodeError::Truncated);
    }
    let v = u64::from_le_bytes(bytes[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    Ok(v)
}

fn read_length_delimited<'a>(
    bytes: &'a [u8],
    pos: &mut usize,
) -> Result<&'a [u8], ProtoDecodeError> {
    let len = read_varint(bytes, pos)? as usize;
    if pos.checked_add(len).map_or(true, |end| end > bytes.len()) {
        return Err(ProtoDecodeError::Truncated);
    }
    let slice = &bytes[*pos..*pos + len];
    *pos += len;
    Ok(slice)
}

fn skip_field(bytes: &[u8], pos: &mut usize, wire_type: u8) -> Result<(), ProtoDecodeError> {
    match wire_type {
        0 => {
            read_varint(bytes, pos)?;
        }
        1 => {
            if *pos + 8 > bytes.len() {
                return Err(ProtoDecodeError::Truncated);
            }
            *pos += 8;
        }
        2 => {
            read_length_delimited(bytes, pos)?;
        }
        5 => {
            if *pos + 4 > bytes.len() {
                return Err(ProtoDecodeError::Truncated);
            }
            *pos += 4;
        }
        other => return Err(ProtoDecodeError::UnknownWireType(other)),
    }
    Ok(())
}

/// Decode a `google.protobuf.StringValue` wrapper (field 1 = string).
fn decode_string_value(bytes: &[u8]) -> Result<String, ProtoDecodeError> {
    let mut pos = 0;
    let mut out = String::new();
    while pos < bytes.len() {
        let tag = read_varint(bytes, &mut pos)?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x7) as u8;
        if field == 1 && wire == 2 {
            let raw = read_length_delimited(bytes, &mut pos)?;
            out = std::str::from_utf8(raw).map_err(|_| ProtoDecodeError::InvalidUtf8)?.to_string();
        } else {
            skip_field(bytes, &mut pos, wire)?;
        }
    }
    Ok(out)
}

/// Decode a `google.protobuf.BoolValue` wrapper. Empty body ⇒ `false`.
fn decode_bool_value(bytes: &[u8]) -> Result<bool, ProtoDecodeError> {
    let mut pos = 0;
    let mut out = false;
    while pos < bytes.len() {
        let tag = read_varint(bytes, &mut pos)?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x7) as u8;
        if field == 1 && wire == 0 {
            out = read_varint(bytes, &mut pos)? != 0;
        } else {
            skip_field(bytes, &mut pos, wire)?;
        }
    }
    Ok(out)
}

/// Decode a `google.protobuf.Int64Value` wrapper (field 1 varint).
fn decode_int64_value(bytes: &[u8]) -> Result<u64, ProtoDecodeError> {
    let mut pos = 0;
    let mut out = 0u64;
    while pos < bytes.len() {
        let tag = read_varint(bytes, &mut pos)?;
        let field = (tag >> 3) as u32;
        let wire = (tag & 0x7) as u8;
        if field == 1 && wire == 0 {
            out = read_varint(bytes, &mut pos)?;
        } else {
            skip_field(bytes, &mut pos, wire)?;
        }
    }
    Ok(out)
}

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

        // Fields 4 and 5 must be fixed64 (wire type 1), not varint.
        if let Some(expires) = self.expires_at_ms {
            encode_fixed64(&mut buf, 4, expires);
        }
        if let Some(created) = self.created_at_ms {
            encode_fixed64(&mut buf, 5, created);
        }

        buf
    }

    /// Decode a CustomStatus protobuf message body (the bytes inside the field-2
    /// length-delimited wrapper of StatusSettings).
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtoDecodeError> {
        let mut out = Self {
            text: String::new(),
            emoji_id: None,
            emoji_name: None,
            expires_at_ms: None,
            created_at_ms: None,
        };
        let mut pos = 0;
        while pos < bytes.len() {
            let tag = read_varint(bytes, &mut pos)?;
            let field = (tag >> 3) as u32;
            let wire = (tag & 0x7) as u8;
            match (field, wire) {
                (1, 2) => {
                    let raw = read_length_delimited(bytes, &mut pos)?;
                    out.text = std::str::from_utf8(raw)
                        .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                        .to_string();
                }
                (2, 0) => out.emoji_id = Some(read_varint(bytes, &mut pos)?),
                (3, 2) => {
                    let raw = read_length_delimited(bytes, &mut pos)?;
                    out.emoji_name = Some(
                        std::str::from_utf8(raw)
                            .map_err(|_| ProtoDecodeError::InvalidUtf8)?
                            .to_string(),
                    );
                }
                (4, 1) => out.expires_at_ms = Some(read_fixed64(bytes, &mut pos)?),
                (5, 1) => out.created_at_ms = Some(read_fixed64(bytes, &mut pos)?),
                (_, w) => skip_field(bytes, &mut pos, w)?,
            }
        }
        Ok(out)
    }
}

/// User status (online, idle, dnd, invisible)
#[derive(Debug, Clone)]
pub struct StatusSettings {
    pub status: String,
    pub custom_status: Option<CustomStatus>,
    /// Field 3 (BoolValue wrapper): `show_current_game`. When `Some(false)` the
    /// wrapper is emitted as an empty sub-message (0x1A 0x00) — the form
    /// Discord's web client serializes when this setting is left at default.
    pub show_current_game: Option<bool>,
    /// Field 5 (Int64Value wrapper): when the status itself should auto-clear,
    /// in milliseconds since the Unix epoch.
    pub status_expires_at_ms: Option<u64>,
}

impl StatusSettings {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            custom_status: None,
            show_current_game: None,
            status_expires_at_ms: None,
        }
    }

    pub fn with_custom_status(mut self, custom: CustomStatus) -> Self {
        self.custom_status = Some(custom);
        self
    }

    pub fn with_show_current_game(mut self, value: bool) -> Self {
        self.show_current_game = Some(value);
        self
    }

    pub fn with_status_expires_at(mut self, ms: u64) -> Self {
        self.status_expires_at_ms = Some(ms);
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

        // Field 3: show_current_game as a wrapped BoolValue. `false` emits an
        // empty wrapper (matches Discord's wire output); `true` emits field 1
        // varint = 1.
        if let Some(value) = self.show_current_game {
            let mut wrap = Vec::new();
            if value {
                encode_varint(&mut wrap, 1u64 << 3); // field 1, varint
                encode_varint(&mut wrap, 1);
            }
            encode_length_delimited(&mut buf, 3, &wrap);
        }

        // Field 5: status_expires_at as a wrapped Int64Value (sub-message with
        // field 1 varint = milliseconds since epoch).
        if let Some(ms) = self.status_expires_at_ms {
            let mut wrap = Vec::new();
            encode_varint(&mut wrap, 1u64 << 3); // field 1, varint
            encode_varint(&mut wrap, ms);
            encode_length_delimited(&mut buf, 5, &wrap);
        }

        buf
    }

    /// Decode a StatusSettings protobuf message body (the bytes inside the
    /// field-11 length-delimited wrapper of PreloadedUserSettings).
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtoDecodeError> {
        let mut out = Self::new("");
        let mut pos = 0;
        while pos < bytes.len() {
            let tag = read_varint(bytes, &mut pos)?;
            let field = (tag >> 3) as u32;
            let wire = (tag & 0x7) as u8;
            match (field, wire) {
                // status — wrapped StringValue
                (1, 2) => {
                    let inner = read_length_delimited(bytes, &mut pos)?;
                    out.status = decode_string_value(inner)?;
                }
                // custom_status — embedded message
                (2, 2) => {
                    let inner = read_length_delimited(bytes, &mut pos)?;
                    out.custom_status = Some(CustomStatus::decode(inner)?);
                }
                // show_current_game — wrapped BoolValue
                (3, 2) => {
                    let inner = read_length_delimited(bytes, &mut pos)?;
                    out.show_current_game = Some(decode_bool_value(inner)?);
                }
                // status_expires_at_ms — wrapped Int64Value
                (5, 2) => {
                    let inner = read_length_delimited(bytes, &mut pos)?;
                    out.status_expires_at_ms = Some(decode_int64_value(inner)?);
                }
                (_, w) => skip_field(bytes, &mut pos, w)?,
            }
        }
        Ok(out)
    }
}

/// PreloadedUserSettings protobuf encoder — status lives at field 11
#[derive(Debug, Clone)]
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

    /// Decode the raw protobuf bytes returned by Discord. Only the `status`
    /// field (PreloadedUserSettings field 11) is materialized — every other
    /// field is skipped without allocation. Unknown wire types beyond the
    /// standard four (varint, fixed64, length-delimited, fixed32) abort the
    /// decode with [`ProtoDecodeError::UnknownWireType`].
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtoDecodeError> {
        let mut out = Self { status: None };
        let mut pos = 0;
        while pos < bytes.len() {
            let tag = read_varint(bytes, &mut pos)?;
            let field = (tag >> 3) as u32;
            let wire = (tag & 0x7) as u8;
            match (field, wire) {
                (11, 2) => {
                    let inner = read_length_delimited(bytes, &mut pos)?;
                    out.status = Some(StatusSettings::decode(inner)?);
                }
                (_, w) => skip_field(bytes, &mut pos, w)?,
            }
        }
        Ok(out)
    }

    /// Convenience wrapper that base64-decodes a `settings` JSON value and
    /// then decodes the resulting protobuf bytes.
    pub fn from_base64(b64: &str) -> Result<Self, ProtoDecodeError> {
        let bytes = STANDARD.decode(b64).map_err(|_| ProtoDecodeError::InvalidBase64)?;
        Self::decode(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal "online + studyphim" wire layout (no timestamps, no extras).
    //   [0]     5A        field 11, wire-type 2 (outer PreloadedUserSettings)
    //   [1]     17        length 23
    //   [2..12] StatusSettings.status wrapped "online"
    //   [12]    12        field 2 (CustomStatus embedded message)
    //   [13]    0B        length 11
    //   [14..16] CustomStatus.text header (0A 09)
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

    // ---------------------------------------------------------------------
    // Captured curl #1 — custom status "studyphim" with a +1h expiry.
    //
    // PATCH /users/@me/settings-proto/1
    // body: {"settings":"WjQKCAoGb25saW5lEh0KCXN0dWR5cGhpbSFPGaPYnQEAAClPX8fXnQEAABoAKgcI79utg5Ez"}
    //
    // Decoded layout (52 inner bytes, 54 with outer tag+len):
    //   5A 34                              field 11 (PreloadedUserSettings), len 52
    //     0A 08 0A 06 "online"              StatusSettings.status
    //     12 1D                             CustomStatus, len 29
    //       0A 09 "studyphim"                 text
    //       21 4F 19 A3 D8 9D 01 00 00       expires_at_ms (field 4, fixed64)
    //       29 4F 5F C7 D7 9D 01 00 00       created_at_ms (field 5, fixed64)
    //     1A 00                             show_current_game (field 3, empty BoolValue)
    //     2A 07 08 EF DB AD 83 91 33        status_expires_at_ms (field 5, Int64Value)
    // ---------------------------------------------------------------------
    const STUDYPHIM_CURL_B64: &str = "WjQKCAoGb25saW5lEh0KCXN0dWR5cGhpbSFPGaPYnQEAAClPX8fXnQEAABoAKgcI79utg5Ez";
    const STALE_STATUS_EXPIRES_AT_MS: u64 = 1_756_917_100_015;

    fn studyphim_settings() -> PreloadedUserSettings {
        let custom = CustomStatus::new("studyphim")
            .with_expiry(0x0000_019D_D8A3_194F)
            .with_created_at(0x0000_019D_D7C7_5F4F);
        let status = StatusSettings::new("online")
            .with_custom_status(custom)
            .with_show_current_game(false)
            .with_status_expires_at(STALE_STATUS_EXPIRES_AT_MS);
        PreloadedUserSettings::with_status(status)
    }

    #[test]
    fn test_encode_matches_studyphim_curl_byte_for_byte() {
        assert_eq!(studyphim_settings().to_base64(), STUDYPHIM_CURL_B64);
        let expected = STANDARD.decode(STUDYPHIM_CURL_B64).expect("captured base64 decodes");
        assert_eq!(studyphim_settings().encode(), expected);
    }

    #[test]
    fn test_encode_studyphim_structure() {
        let bytes = studyphim_settings().encode();

        // Outer PreloadedUserSettings: field 11, wire-type 2, length 52.
        assert_eq!(bytes[0], 0x5A);
        assert_eq!(bytes[1], 0x34);

        // StatusSettings.status = "online".
        assert_eq!(&bytes[2..4], &[0x0A, 0x08]);
        assert_eq!(&bytes[4..6], &[0x0A, 0x06]);
        assert_eq!(&bytes[6..12], b"online");

        // CustomStatus header: field 2, length 29.
        assert_eq!(bytes[12], 0x12);
        assert_eq!(bytes[13], 0x1D);

        // CustomStatus.text = "studyphim".
        assert_eq!(&bytes[14..16], &[0x0A, 0x09]);
        assert_eq!(&bytes[16..25], b"studyphim");

        // CustomStatus.expires_at_ms (field 4, fixed64).
        assert_eq!(bytes[25], 0x21);
        let expires = u64::from_le_bytes(bytes[26..34].try_into().unwrap());
        assert_eq!(expires, 0x0000_019D_D8A3_194F);

        // CustomStatus.created_at_ms (field 5, fixed64).
        assert_eq!(bytes[34], 0x29);
        let created = u64::from_le_bytes(bytes[35..43].try_into().unwrap());
        assert_eq!(created, 0x0000_019D_D7C7_5F4F);

        // StatusSettings.show_current_game (field 3, empty BoolValue).
        assert_eq!(bytes[43], 0x1A);
        assert_eq!(bytes[44], 0x00);

        // StatusSettings.status_expires_at_ms (field 5, Int64Value).
        assert_eq!(bytes[45], 0x2A);
        assert_eq!(bytes[46], 0x07);
        assert_eq!(bytes[47], 0x08);
        assert_eq!(&bytes[48..54], &[0xEF, 0xDB, 0xAD, 0x83, 0x91, 0x33]);
        assert_eq!(bytes.len(), 54);
    }

    // ---------------------------------------------------------------------
    // Captured curl #2 — custom status "status test 1" with a +1h expiry.
    //
    // PATCH /users/@me/settings-proto/1
    // body: {"settings":"WjgKCAoGb25saW5lEiEKDXN0YXR1cyB0ZXN0IDEhfEwE2J0BAAAp/F3N150BAAAaACoHCO/brYORMw=="}
    //
    // Decoded layout (56 inner bytes, 58 with outer tag+len):
    //   5A 38                              field 11 (PreloadedUserSettings), len 56
    //     0A 08 0A 06 "online"              StatusSettings.status
    //     12 21                             CustomStatus, len 33
    //       0A 0D "status test 1"             text
    //       21 7C 4C 04 D8 9D 01 00 00       expires_at_ms (field 4, fixed64)
    //       29 FC 5D CD D7 9D 01 00 00       created_at_ms (field 5, fixed64)
    //     1A 00                             show_current_game (field 3, empty BoolValue)
    //     2A 07 08 EF DB AD 83 91 33        status_expires_at_ms (field 5, Int64Value)
    // ---------------------------------------------------------------------
    const STATUS_TEST_1_CURL_B64: &str =
        "WjgKCAoGb25saW5lEiEKDXN0YXR1cyB0ZXN0IDEhfEwE2J0BAAAp/F3N150BAAAaACoHCO/brYORMw==";
    const STATUS_TEST_1_EXPIRES_MS: u64 = 0x0000_019D_D804_4C7C;
    const STATUS_TEST_1_CREATED_MS: u64 = 0x0000_019D_D7CD_5DFC;

    fn status_test_1_settings() -> PreloadedUserSettings {
        let custom = CustomStatus::new("status test 1")
            .with_expiry(STATUS_TEST_1_EXPIRES_MS)
            .with_created_at(STATUS_TEST_1_CREATED_MS);
        let status = StatusSettings::new("online")
            .with_custom_status(custom)
            .with_show_current_game(false)
            .with_status_expires_at(STALE_STATUS_EXPIRES_AT_MS);
        PreloadedUserSettings::with_status(status)
    }

    #[test]
    fn test_encode_matches_status_test_1_curl_byte_for_byte() {
        assert_eq!(status_test_1_settings().to_base64(), STATUS_TEST_1_CURL_B64);
        let expected = STANDARD.decode(STATUS_TEST_1_CURL_B64).expect("captured base64 decodes");
        assert_eq!(status_test_1_settings().encode(), expected);
    }

    #[test]
    fn test_encode_status_test_1_structure() {
        let bytes = status_test_1_settings().encode();

        // Outer PreloadedUserSettings: field 11, wire-type 2, length 56.
        assert_eq!(bytes[0], 0x5A);
        assert_eq!(bytes[1], 0x38);

        // StatusSettings.status = "online".
        assert_eq!(&bytes[2..4], &[0x0A, 0x08]);
        assert_eq!(&bytes[4..6], &[0x0A, 0x06]);
        assert_eq!(&bytes[6..12], b"online");

        // CustomStatus header: field 2, length 33.
        assert_eq!(bytes[12], 0x12);
        assert_eq!(bytes[13], 0x21);

        // CustomStatus.text = "status test 1".
        assert_eq!(&bytes[14..16], &[0x0A, 0x0D]);
        assert_eq!(&bytes[16..29], b"status test 1");

        // CustomStatus.expires_at_ms (field 4, fixed64).
        assert_eq!(bytes[29], 0x21);
        let expires = u64::from_le_bytes(bytes[30..38].try_into().unwrap());
        assert_eq!(expires, STATUS_TEST_1_EXPIRES_MS);

        // CustomStatus.created_at_ms (field 5, fixed64).
        assert_eq!(bytes[38], 0x29);
        let created = u64::from_le_bytes(bytes[39..47].try_into().unwrap());
        assert_eq!(created, STATUS_TEST_1_CREATED_MS);

        // The screenshot reads "Clear at 13:54" — expiry is exactly +1 hour.
        assert_eq!(expires - created, 3_600_000);

        // StatusSettings.show_current_game (field 3, empty BoolValue).
        assert_eq!(bytes[47], 0x1A);
        assert_eq!(bytes[48], 0x00);

        // StatusSettings.status_expires_at_ms (field 5, Int64Value).
        assert_eq!(bytes[49], 0x2A);
        assert_eq!(bytes[50], 0x07);
        assert_eq!(bytes[51], 0x08);
        assert_eq!(&bytes[52..58], &[0xEF, 0xDB, 0xAD, 0x83, 0x91, 0x33]);
        assert_eq!(bytes.len(), 58);
    }

    // The Int64Value varint payload `08 EF DB AD 83 91 33` decodes to a single
    // millisecond timestamp; locking the value down keeps regressions in the
    // varint encoder honest.
    #[test]
    fn test_status_expires_at_varint_value() {
        let mut buf = Vec::new();
        encode_varint(&mut buf, STALE_STATUS_EXPIRES_AT_MS);
        assert_eq!(buf, vec![0xEF, 0xDB, 0xAD, 0x83, 0x91, 0x33]);
    }

    // show_current_game = true should emit `1A 02 08 01` (BoolValue with field 1
    // varint = 1), not the empty `1A 00` form. Guards the bool branch.
    #[test]
    fn test_show_current_game_true_emits_value() {
        let status = StatusSettings::new("online").with_show_current_game(true);
        let bytes = status.encode();
        // Last 4 bytes should be the BoolValue wrapper with field 1 varint = 1.
        assert_eq!(&bytes[bytes.len() - 4..], &[0x1A, 0x02, 0x08, 0x01]);
    }

    // ---------------------------------------------------------------------
    // Decoder tests — round-trip and live response from Discord.
    // ---------------------------------------------------------------------

    #[test]
    fn test_decode_round_trips_status_test_1() {
        let original = status_test_1_settings();
        let decoded =
            PreloadedUserSettings::from_base64(STATUS_TEST_1_CURL_B64).expect("captured base64 decodes");
        let status = decoded.status.as_ref().expect("status sub-message present");
        assert_eq!(status.status, "online");
        assert_eq!(status.show_current_game, Some(false));
        assert_eq!(status.status_expires_at_ms, Some(STALE_STATUS_EXPIRES_AT_MS));

        let custom = status.custom_status.as_ref().expect("custom_status present");
        assert_eq!(custom.text, "status test 1");
        assert_eq!(custom.expires_at_ms, Some(STATUS_TEST_1_EXPIRES_MS));
        assert_eq!(custom.created_at_ms, Some(STATUS_TEST_1_CREATED_MS));
        // emoji_name absent in the captured curl (the trailing `1A 00` belongs
        // to StatusSettings.show_current_game, not CustomStatus.emoji_name).
        assert!(custom.emoji_name.is_none());

        // Re-encode and verify byte equivalence with the original capture.
        let original_encoded = original.encode();
        let recoded = PreloadedUserSettings::with_status(status.clone()).encode();
        assert_eq!(recoded, original_encoded, "decode → re-encode round-trips bit-perfectly");
    }

    #[test]
    fn test_decode_studyphim_round_trips() {
        let decoded =
            PreloadedUserSettings::from_base64(STUDYPHIM_CURL_B64).expect("captured base64 decodes");
        let status = decoded.status.as_ref().expect("status sub-message present");
        assert_eq!(status.status, "online");
        assert_eq!(status.show_current_game, Some(false));
        assert_eq!(status.status_expires_at_ms, Some(STALE_STATUS_EXPIRES_AT_MS));

        let custom = status.custom_status.as_ref().expect("custom_status present");
        assert_eq!(custom.text, "studyphim");
        assert_eq!(custom.expires_at_ms, Some(0x0000_019D_D8A3_194F));
        assert_eq!(custom.created_at_ms, Some(0x0000_019D_D7C7_5F4F));
    }

    // Live captured PATCH response from Discord. The body is the full
    // PreloadedUserSettings proto echoed back by the server — every top-level
    // field except `status` should be skipped silently. Verifies the decoder
    // copes with unknown fields and arbitrary wire types.
    const LIVE_RESPONSE_B64: &str = include_str!("../tests/fixtures/settings_proto_response.b64");

    #[test]
    fn test_decode_live_response_extracts_status() {
        let trimmed = LIVE_RESPONSE_B64.trim();
        let decoded = PreloadedUserSettings::from_base64(trimmed).expect("live response decodes");
        let status = decoded.status.as_ref().expect("status sub-message present");

        // Same trio of values as the curl capture (the response echoes the
        // PATCH body back).
        assert_eq!(status.status, "online");
        assert_eq!(status.show_current_game, Some(false));
        assert_eq!(status.status_expires_at_ms, Some(STALE_STATUS_EXPIRES_AT_MS));

        let custom = status.custom_status.as_ref().expect("custom_status echoed back");
        assert_eq!(custom.text, "status test 1");
        assert_eq!(custom.expires_at_ms, Some(STATUS_TEST_1_EXPIRES_MS));
        assert_eq!(custom.created_at_ms, Some(STATUS_TEST_1_CREATED_MS));
    }

    // Encoder/decoder round-trip with every CustomStatus field populated.
    #[test]
    fn test_custom_status_full_round_trip() {
        let custom = CustomStatus {
            text: "✨ working".to_string(),
            emoji_id: Some(123_456_789),
            emoji_name: Some("sparkles".to_string()),
            expires_at_ms: Some(1_777_445_453_628),
            created_at_ms: Some(1_777_441_853_628),
        };
        let bytes = custom.encode();
        let decoded = CustomStatus::decode(&bytes).expect("round-trip decode");
        assert_eq!(decoded.text, custom.text);
        assert_eq!(decoded.emoji_id, custom.emoji_id);
        assert_eq!(decoded.emoji_name, custom.emoji_name);
        assert_eq!(decoded.expires_at_ms, custom.expires_at_ms);
        assert_eq!(decoded.created_at_ms, custom.created_at_ms);
    }

    // The decoder must reject truncated input rather than panic.
    #[test]
    fn test_decode_truncated_input_is_error() {
        let bytes = STANDARD.decode(STATUS_TEST_1_CURL_B64).unwrap();
        let truncated = &bytes[..bytes.len() - 5];
        assert!(matches!(
            PreloadedUserSettings::decode(truncated),
            Err(ProtoDecodeError::Truncated)
        ));
    }

    // Invalid base64 should surface a typed error.
    #[test]
    fn test_decode_invalid_base64() {
        assert!(matches!(
            PreloadedUserSettings::from_base64("not-valid-base64-!!!!"),
            Err(ProtoDecodeError::InvalidBase64)
        ));
    }
}
