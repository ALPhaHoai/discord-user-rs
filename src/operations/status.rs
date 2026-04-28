//! Status operations for DiscordUser

use serde_json::{json, Value};

use crate::{
    context::DiscordContext,
    error::{DiscordError, Result},
    route::Route,
    types::*,
};

/// Rich presence activity sent via the gateway Presence Update opcode (op 3).
///
/// Maps to the Discord activity `type` integers:
/// 0 = Playing, 1 = Streaming, 2 = Listening, 3 = Watching, 5 = Competing.
///
/// # Example
/// ```ignore
/// user.set_activity(ActivityData::playing("Chess"), UserStatus::Online).await?;
/// user.set_activity(ActivityData::streaming("Coding", "https://twitch.tv/me"), UserStatus::Online).await?;
/// ```
#[derive(Debug, Clone)]
pub enum ActivityData {
    /// "Playing {name}" — type 0
    Playing { name: String },
    /// "Streaming {name}" — type 1; requires a Twitch/YouTube URL
    Streaming { name: String, url: String },
    /// "Listening to {name}" — type 2
    Listening { name: String },
    /// "Watching {name}" — type 3
    Watching { name: String },
    /// "Competing in {name}" — type 5
    Competing { name: String },
}

impl ActivityData {
    pub fn playing(name: impl Into<String>) -> Self {
        Self::Playing { name: name.into() }
    }
    pub fn streaming(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Streaming { name: name.into(), url: url.into() }
    }
    pub fn listening(name: impl Into<String>) -> Self {
        Self::Listening { name: name.into() }
    }
    pub fn watching(name: impl Into<String>) -> Self {
        Self::Watching { name: name.into() }
    }
    pub fn competing(name: impl Into<String>) -> Self {
        Self::Competing { name: name.into() }
    }

    fn to_json(&self) -> Value {
        match self {
            Self::Playing { name } => json!({ "name": name, "type": 0 }),
            Self::Streaming { name, url } => json!({ "name": name, "type": 1, "url": url }),
            Self::Listening { name } => json!({ "name": name, "type": 2 }),
            Self::Watching { name } => json!({ "name": name, "type": 3 }),
            Self::Competing { name } => json!({ "name": name, "type": 5 }),
        }
    }
}

impl<T: DiscordContext + Send + Sync> StatusOps for T {}

/// Extension trait providing status operations
#[allow(async_fn_in_trait)]
pub trait StatusOps: DiscordContext {
    /// Set the user's online status (online, idle, dnd, or invisible).
    ///
    /// Sends a gateway Presence Update (op 3) with no activity.
    ///
    /// # Errors
    /// Returns [`DiscordError::NotInitialized`] if the gateway is not
    /// connected. Returns [`DiscordError::WebSocket`] on send failure.
    async fn set_status(&self, status: UserStatus) -> Result<()> {
        if let Some(gateway) = self.gateway() {
            gateway.send_presence(status).await
        } else {
            Err(DiscordError::NotInitialized)
        }
    }

    /// Set custom status with text (and optionally persist via protobuf API)
    ///
    /// # Arguments
    /// * `status` - The user status (online, idle, dnd, invisible)
    /// * `custom_status_text` - Optional custom status text (e.g., "Playing
    ///   games")
    /// * `expires_at_ms` - Optional expiration timestamp in milliseconds
    ///
    /// # Example
    /// ```ignore
    /// user.set_custom_status(UserStatus::Online, Some("Working"), None).await?;
    /// ```
    async fn set_custom_status(&self, status: UserStatus, custom_status_text: Option<&str>, expires_at_ms: Option<u64>) -> Result<()> {
        use crate::proto::{CustomStatus, PreloadedUserSettings, StatusSettings};

        // Build WebSocket presence payload
        let mut activities = Vec::new();
        if let Some(text) = custom_status_text {
            if !text.is_empty() {
                let mut activity = json!({
                    "name": "Custom Status",
                    "type": 4,
                    "state": text,
                    "emoji": null
                });
                if let Some(expires) = expires_at_ms {
                    activity["timestamps"] = json!({ "end": expires });
                }
                activities.push(activity);
            }
        }

        let ws_payload = json!({
            "op": 3,
            "d": {
                "status": status.as_str(),
                "since": 0,
                "activities": activities,
                "afk": false
            }
        });

        // Send via WebSocket
        if let Some(gateway) = self.gateway() {
            gateway.send_raw(ws_payload).await?;
        }

        // Build protobuf settings for persistence
        let mut status_settings = StatusSettings::new(status.as_str());

        if let Some(text) = custom_status_text {
            if !text.is_empty() {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let mut custom = CustomStatus::new(text).with_created_at(now_ms);
                if let Some(expires) = expires_at_ms {
                    custom = custom.with_expiry(expires);
                }
                status_settings = status_settings.with_custom_status(custom);
            }
        }

        let settings = PreloadedUserSettings::with_status(status_settings);
        let encoded = settings.to_base64();

        // Persist via HTTP API
        let _result: Value = self.http().patch(crate::route::Route::SettingsProto { version: 1 }, json!({ "settings": encoded })).await?;

        Ok(())
    }

    /// Clear the current custom status, reverting to the default Online state.
    ///
    /// # Errors
    /// Returns [`DiscordError::NotInitialized`] if the gateway is not
    /// connected.
    async fn clear_custom_status(&self) -> Result<()> {
        self.set_custom_status(UserStatus::Online, None, None).await
    }

    /// Set a rich presence activity (Playing, Streaming, Listening, Watching,
    /// Competing).
    ///
    /// Sends a gateway Presence Update (op 3) with the given activity and
    /// status. The activity appears in the user's profile visible to
    /// friends and guild members.
    ///
    /// # Example
    /// ```ignore
    /// user.set_activity(ActivityData::playing("Chess"), UserStatus::Online).await?;
    /// ```
    async fn set_activity(&self, activity: ActivityData, status: UserStatus) -> Result<()> {
        let ws_payload = json!({
            "op": 3,
            "d": {
                "status": status.as_str(),
                "since": 0,
                "activities": [activity.to_json()],
                "afk": false
            }
        });

        if let Some(gateway) = self.gateway() {
            gateway.send_raw(ws_payload).await?;
        } else {
            return Err(DiscordError::NotInitialized);
        }
        Ok(())
    }

    /// Clear the current activity (removes the "Playing …" badge), keeping the
    /// status.
    ///
    /// # Errors
    /// Returns [`DiscordError::NotInitialized`] if the gateway is not
    /// connected.
    async fn clear_activity(&self, status: UserStatus) -> Result<()> {
        let ws_payload = json!({
            "op": 3,
            "d": {
                "status": status.as_str(),
                "since": 0,
                "activities": [],
                "afk": false
            }
        });

        if let Some(gateway) = self.gateway() {
            gateway.send_raw(ws_payload).await?;
        } else {
            return Err(DiscordError::NotInitialized);
        }
        Ok(())
    }

    /// Edit the current user's profile.
    ///
    /// Only fields set to `Some(...)` on `req` are sent; unset fields are left
    /// unchanged.  Returns the updated [`User`] object.
    async fn edit_profile(&self, req: EditProfileRequest) -> Result<User> {
        self.http().patch(Route::UpdateMe, req).await
    }
}
