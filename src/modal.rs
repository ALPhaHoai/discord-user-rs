//! Quick modal builder for Discord interactions.
//!
//! Discord modals are pop-up forms shown in response to slash commands or
//! button interactions.  A modal has a `custom_id`, a `title`, and up to 5
//! text-input components organised inside action rows.
//!
//! Use [`ModalBuilder`] to construct the JSON payload to pass as the
//! interaction response body.
//!
//! # Example
//! ```
//! use discord_user::modal::ModalBuilder;
//!
//! let modal = ModalBuilder::new("my_modal", "Tell us about yourself")
//!     .short_field("name_input", "Your name")
//!     .paragraph_field("bio_input", "Short bio", Some("Tell us about yourself…"))
//!     .build();
//!
//! assert_eq!(modal["custom_id"], "my_modal");
//! assert_eq!(modal["title"], "Tell us about yourself");
//! let components = modal["components"].as_array().unwrap();
//! assert_eq!(components.len(), 2);
//! ```

use serde_json::{json, Value};

/// Text-input style constants (Discord component API).
const TEXT_SHORT: u8 = 1;
const TEXT_PARAGRAPH: u8 = 2;

/// Builder for a Discord modal interaction response payload.
///
/// The final [`build`](ModalBuilder::build) call returns a `serde_json::Value`
/// ready to be sent as the `data` field of an `INTERACTION_CALLBACK` response
/// with type `9` (MODAL).
pub struct ModalBuilder {
    custom_id: String,
    title: String,
    components: Vec<Value>,
}

impl ModalBuilder {
    /// Create a new modal builder.
    ///
    /// * `custom_id` — developer-defined identifier (max 100 chars), echoed
    ///   back in the `INTERACTION_CREATE` event when the user submits.
    /// * `title` — text shown in the modal header (max 45 chars).
    pub fn new(custom_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self { custom_id: custom_id.into(), title: title.into(), components: Vec::new() }
    }

    /// Add a single-line text input.
    ///
    /// * `custom_id` — field identifier (max 100 chars).
    /// * `label` — label shown above the input (max 45 chars).
    pub fn short_field(mut self, custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        self.push_text_input(custom_id.into(), label.into(), TEXT_SHORT, None, false);
        self
    }

    /// Add a single-line text input that the user must fill in.
    pub fn required_short_field(mut self, custom_id: impl Into<String>, label: impl Into<String>) -> Self {
        self.push_text_input(custom_id.into(), label.into(), TEXT_SHORT, None, true);
        self
    }

    /// Add a multi-line paragraph text input.
    ///
    /// * `placeholder` — grey hint text shown inside the input when empty.
    pub fn paragraph_field(mut self, custom_id: impl Into<String>, label: impl Into<String>, placeholder: Option<&str>) -> Self {
        self.push_text_input(custom_id.into(), label.into(), TEXT_PARAGRAPH, placeholder.map(str::to_string), false);
        self
    }

    /// Add a required multi-line paragraph text input.
    pub fn required_paragraph_field(mut self, custom_id: impl Into<String>, label: impl Into<String>, placeholder: Option<&str>) -> Self {
        self.push_text_input(custom_id.into(), label.into(), TEXT_PARAGRAPH, placeholder.map(str::to_string), true);
        self
    }

    fn push_text_input(&mut self, custom_id: String, label: String, style: u8, placeholder: Option<String>, required: bool) {
        let mut input = json!({
            "type": 4,  // TEXT_INPUT component type
            "custom_id": custom_id,
            "label": label,
            "style": style,
            "required": required,
        });
        if let Some(ph) = placeholder {
            input["placeholder"] = json!(ph);
        }
        // Wrap in an action row (type 1)
        self.components.push(json!({
            "type": 1,
            "components": [input],
        }));
    }

    /// Consume the builder and produce the modal JSON payload.
    ///
    /// Returns the `data` object for an `INTERACTION_CALLBACK` response of
    /// type `9` (MODAL).
    pub fn build(self) -> Value {
        json!({
            "custom_id": self.custom_id,
            "title": self.title,
            "components": self.components,
        })
    }

    /// Wrap the modal payload in a full interaction callback response body.
    ///
    /// Returns `{ "type": 9, "data": { … } }`, ready to POST to
    /// `/interactions/{id}/{token}/callback`.
    pub fn into_response(self) -> Value {
        json!({
            "type": 9,  // MODAL interaction callback type
            "data": self.build(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_modal() {
        let m = ModalBuilder::new("id", "Title").build();
        assert_eq!(m["custom_id"], "id");
        assert_eq!(m["title"], "Title");
        assert_eq!(m["components"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn short_field_wraps_in_action_row() {
        let m = ModalBuilder::new("id", "Title").short_field("f1", "Name").build();
        let rows = m["components"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["type"], 1); // action row
        let inner = &rows[0]["components"][0];
        assert_eq!(inner["type"], 4); // text input
        assert_eq!(inner["style"], TEXT_SHORT as i64);
        assert_eq!(inner["custom_id"], "f1");
        assert_eq!(inner["label"], "Name");
    }

    #[test]
    fn paragraph_with_placeholder() {
        let m = ModalBuilder::new("id", "T").paragraph_field("bio", "Bio", Some("Enter bio…")).build();
        let inner = &m["components"][0]["components"][0];
        assert_eq!(inner["style"], TEXT_PARAGRAPH as i64);
        assert_eq!(inner["placeholder"], "Enter bio…");
    }

    #[test]
    fn two_fields() {
        let m = ModalBuilder::new("id", "T").short_field("f1", "First").paragraph_field("f2", "Second", None).build();
        assert_eq!(m["components"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn into_response_wraps_type_9() {
        let r = ModalBuilder::new("id", "T").into_response();
        assert_eq!(r["type"], 9);
        assert_eq!(r["data"]["custom_id"], "id");
    }
}
