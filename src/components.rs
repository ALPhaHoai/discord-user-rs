//! Message component builders — buttons, select menus, and action rows.
//!
//! Discord message components allow interactive UI elements inside messages.
//! They are arranged in [`CreateActionRow`] containers, each holding either
//! a row of [`CreateButton`]s or a single [`CreateSelectMenu`].
//!
//! Pass the assembled rows to [`MessageBuilder::components`].
//!
//! # Example
//! ```ignore
//! use discord_user::components::{CreateActionRow, CreateButton, ButtonStyle};
//!
//! let row = CreateActionRow::buttons(vec![
//!     CreateButton::new("confirm", ButtonStyle::Success).label("Confirm"),
//!     CreateButton::new("cancel",  ButtonStyle::Danger).label("Cancel"),
//! ]);
//! user.message()
//!     .channel(channel_id)
//!     .content("Are you sure?")
//!     .components(vec![row])
//!     .send()
//!     .await?;
//! ```

use serde::Serialize;
use serde_json::{json, Value};

// ── Button style ─────────────────────────────────────────────────────────────

/// Visual style of a [`CreateButton`].
///
/// Mirrors serenity's `ButtonStyle` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ButtonStyle {
    /// Blurple — a standard action button (requires `custom_id`).
    Primary = 1,
    /// Grey — a secondary action button (requires `custom_id`).
    Secondary = 2,
    /// Green — a positive/confirm button (requires `custom_id`).
    Success = 3,
    /// Red — a danger/destructive button (requires `custom_id`).
    Danger = 4,
    /// Grey link button that navigates to a URL (requires `url`, no
    /// `custom_id`).
    Link = 5,
}

// ── Button ───────────────────────────────────────────────────────────────────

/// A clickable button component.
///
/// Mirrors serenity's `CreateButton`.
///
/// # Example
/// ```ignore
/// use discord_user::components::{CreateButton, ButtonStyle};
/// let btn = CreateButton::new("my_btn", ButtonStyle::Primary)
///     .label("Click me")
///     .disabled(false);
/// ```
#[derive(Debug, Clone)]
pub struct CreateButton {
    style: ButtonStyle,
    custom_id: Option<String>,
    url: Option<String>,
    label: Option<String>,
    emoji: Option<Value>,
    disabled: bool,
}

impl CreateButton {
    /// Create a non-link button with the given `custom_id` and style.
    pub fn new(custom_id: impl Into<String>, style: ButtonStyle) -> Self {
        Self { style, custom_id: Some(custom_id.into()), url: None, label: None, emoji: None, disabled: false }
    }

    /// Create a link button that navigates to `url`.
    pub fn link(url: impl Into<String>) -> Self {
        Self { style: ButtonStyle::Link, custom_id: None, url: Some(url.into()), label: None, emoji: None, disabled: false }
    }

    /// Set the visible label text.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set a Unicode or partial custom emoji.
    ///
    /// `emoji` should be a JSON object matching Discord's emoji partial:
    /// `{"id": null, "name": "👍"}` or `{"id": "123", "name": "upvote",
    /// "animated": false}`.
    pub fn emoji(mut self, emoji: Value) -> Self {
        self.emoji = Some(emoji);
        self
    }

    /// Whether the button appears greyed-out and cannot be clicked.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Serialize to a Discord component JSON object.
    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "type": 2,
            "style": self.style as u8,
            "disabled": self.disabled,
        });
        if let Some(ref id) = self.custom_id {
            obj["custom_id"] = json!(id);
        }
        if let Some(ref url) = self.url {
            obj["url"] = json!(url);
        }
        if let Some(ref label) = self.label {
            obj["label"] = json!(label);
        }
        if let Some(ref emoji) = self.emoji {
            obj["emoji"] = emoji.clone();
        }
        obj
    }
}

// ── Select menu option
// ────────────────────────────────────────────────────────

/// A single option within a [`CreateSelectMenu`].
///
/// Mirrors serenity's `CreateSelectMenuOption`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSelectMenuOption {
    /// Displayed text.
    pub label: String,
    /// Value sent to the application when this option is chosen.
    pub value: String,
    /// Optional description shown below the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional partial emoji shown next to the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<Value>,
    /// If true, this option is pre-selected as the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

impl CreateSelectMenuOption {
    /// Create an option with the required label and value fields.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self { label: label.into(), value: value.into(), description: None, emoji: None, default: None }
    }

    /// Add a description line.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark this option as the default selection.
    pub fn default_selection(mut self, is_default: bool) -> Self {
        self.default = Some(is_default);
        self
    }
}

// ── Select menu
// ───────────────────────────────────────────────────────────────

/// A dropdown select menu component.
///
/// Mirrors serenity's `CreateSelectMenu`.
///
/// # Example
/// ```ignore
/// use discord_user::components::{CreateSelectMenu, CreateSelectMenuOption};
/// let menu = CreateSelectMenu::new("pick_color")
///     .placeholder("Choose a color")
///     .add_option(CreateSelectMenuOption::new("Red", "red"))
///     .add_option(CreateSelectMenuOption::new("Blue", "blue"));
/// ```
#[derive(Debug, Clone)]
pub struct CreateSelectMenu {
    custom_id: String,
    placeholder: Option<String>,
    min_values: Option<u8>,
    max_values: Option<u8>,
    disabled: bool,
    options: Vec<CreateSelectMenuOption>,
}

impl CreateSelectMenu {
    /// Create a new string-select menu with the given `custom_id`.
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: false,
            options: Vec::new(),
        }
    }

    /// Set the placeholder text shown when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Minimum number of values the user must select.
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Maximum number of values the user can select (up to 25).
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Whether the menu is disabled (greyed out, unclickable).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add an option to the menu.
    pub fn add_option(mut self, option: CreateSelectMenuOption) -> Self {
        self.options.push(option);
        self
    }

    /// Replace all options at once.
    pub fn options(mut self, options: Vec<CreateSelectMenuOption>) -> Self {
        self.options = options;
        self
    }

    /// Serialize to a Discord component JSON object.
    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "type": 3,            // STRING_SELECT
            "custom_id": self.custom_id,
            "options": self.options,
            "disabled": self.disabled,
        });
        if let Some(ref ph) = self.placeholder {
            obj["placeholder"] = json!(ph);
        }
        if let Some(min) = self.min_values {
            obj["min_values"] = json!(min);
        }
        if let Some(max) = self.max_values {
            obj["max_values"] = json!(max);
        }
        obj
    }
}

// ── Action row
// ────────────────────────────────────────────────────────────────

/// A container for up to 5 buttons or 1 select menu.
///
/// Mirrors serenity's `CreateActionRow`.
#[derive(Debug, Clone)]
pub enum CreateActionRow {
    /// A row containing 1–5 buttons.
    Buttons(Vec<CreateButton>),
    /// A row containing a single select menu.
    SelectMenu(CreateSelectMenu),
}

impl CreateActionRow {
    /// Convenience constructor for a button row.
    pub fn buttons(buttons: Vec<CreateButton>) -> Self {
        Self::Buttons(buttons)
    }

    /// Convenience constructor for a select-menu row.
    pub fn select_menu(menu: CreateSelectMenu) -> Self {
        Self::SelectMenu(menu)
    }

    /// Serialize to a Discord action-row component JSON object.
    pub fn to_json(&self) -> Value {
        match self {
            Self::Buttons(buttons) => json!({
                "type": 1,
                "components": buttons.iter().map(|b| b.to_json()).collect::<Vec<_>>(),
            }),
            Self::SelectMenu(menu) => json!({
                "type": 1,
                "components": [menu.to_json()],
            }),
        }
    }
}
