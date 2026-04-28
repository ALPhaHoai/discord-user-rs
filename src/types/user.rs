//! Discord user types

use serde::{Deserialize, Serialize};

use super::{ImageHash, ImageSize, RoleId, UserId, UserPublicFlags};

/// Discord user data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub discriminator: String,
    #[serde(default)]
    pub global_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<ImageHash>,
    #[serde(default)]
    pub avatar_decoration_data: Option<serde_json::Value>,
    #[serde(default)]
    pub banner: Option<String>,
    #[serde(default)]
    pub banner_color: Option<String>,
    #[serde(default)]
    pub accent_color: Option<u32>,
    #[serde(default)]
    pub public_flags: UserPublicFlags,
    #[serde(default)]
    pub flags: UserPublicFlags,
    #[serde(default)]
    pub premium_type: u8,
    #[serde(default)]
    pub bot: bool,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub mfa_enabled: bool,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub nsfw_allowed: Option<bool>,
    #[serde(default)]
    pub mobile: bool,
    #[serde(default)]
    pub desktop: bool,
}

impl User {
    /// Get display name (global_name or username)
    pub fn display_name(&self) -> &str {
        self.global_name.as_deref().unwrap_or(&self.username)
    }

    /// Get avatar URL (WebP for static, GIF for animated) at a given size.
    pub fn avatar_url(&self, size: ImageSize) -> Option<String> {
        self.avatar.as_ref().and_then(|hash| self.id.parse::<u64>().ok().map(|id| hash.user_avatar_url(id, Some(size))))
    }

    /// Return the Discord "tag" — `username#discriminator` for legacy accounts
    /// or just `username` for new pomelo accounts (discriminator is "0").
    ///
    /// Mirrors serenity's `User::tag()`.
    pub fn tag(&self) -> String {
        if self.discriminator.is_empty() || self.discriminator == "0" || self.discriminator == "0000" {
            self.username.clone()
        } else {
            format!("{}#{}", self.username, self.discriminator)
        }
    }

    /// Format the user as a mention string (`<@user_id>`).
    ///
    /// Mirrors serenity's `User::mention()` / `Mentionable` impl.
    pub fn mention(&self) -> String {
        format!("<@{}>", self.id)
    }

    /// Return the default avatar URL for users without a custom avatar.
    ///
    /// For legacy accounts (discriminator != "0") the index is `discriminator %
    /// 5`. For pomelo accounts the index is `(user_id >> 22) % 6`.
    ///
    /// Mirrors serenity's `User::default_avatar_url()`.
    pub fn default_avatar_url(&self) -> String {
        let index = if self.discriminator.is_empty() || self.discriminator == "0" || self.discriminator == "0000" {
            // Pomelo: use snowflake
            self.id.parse::<u64>().map(|id| (id >> 22) % 6).unwrap_or(0)
        } else {
            // Legacy: use discriminator % 5
            self.discriminator.parse::<u64>().unwrap_or(0) % 5
        };
        format!("https://cdn.discordapp.com/embed/avatars/{}.png", index)
    }

    /// Return the user's effective face URL — custom avatar if set, otherwise
    /// the default avatar.
    ///
    /// Mirrors serenity's `User::face()`.
    pub fn face(&self) -> String {
        self.avatar_url(ImageSize::Size128).unwrap_or_else(|| self.default_avatar_url())
    }

    /// Return the user's banner URL, if they have one.
    ///
    /// The banner hash is present on the full user object returned by the
    /// profile endpoint; it may be `None` on partial user objects.
    ///
    /// Mirrors serenity's `User::banner_url()`.
    pub fn banner_url(&self) -> Option<String> {
        let banner = self.banner.as_ref()?;
        let user_id = self.id.parse::<u64>().ok()?;
        let ext = if banner.starts_with("a_") { "gif" } else { "webp" };
        Some(format!("https://cdn.discordapp.com/banners/{}/{}.{}?size=512", user_id, banner, ext))
    }
}

/// Guild member data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Member {
    #[serde(default)]
    pub user: Option<User>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub nick: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub joined_at: Option<String>,
    #[serde(default)]
    pub premium_since: Option<String>,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub flags: u64,
    #[serde(default)]
    pub communication_disabled_until: Option<String>,
}

impl Member {
    /// Get display name (nick > global_name > username)
    pub fn display_name(&self) -> Option<&str> {
        self.nick.as_deref().or_else(|| self.user.as_ref().map(|u| u.display_name()))
    }

    /// Format this member as a mention string (`<@user_id>`).
    ///
    /// Uses the inner user ID if the `user` field is populated, or falls back
    /// to the `user_id` field.  Returns `None` if neither is present.
    ///
    /// Mirrors serenity's `Member::mention()`.
    pub fn mention(&self) -> Option<String> {
        let id = self.user.as_ref().map(|u| u.id.as_str()).or(self.user_id.as_deref())?;
        Some(format!("<@{}>", id))
    }

    /// Whether this member is currently timed out (communication disabled).
    ///
    /// A member is timed-out when `communication_disabled_until` is set to a
    /// future ISO 8601 timestamp.
    ///
    /// Mirrors serenity's `Member::is_timed_out()`.
    pub fn is_timed_out(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        self.communication_disabled_until
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
                dt.timestamp() > now
            })
            .unwrap_or(false)
    }

    /// Whether this member has a guild-specific avatar set.
    ///
    /// Mirrors serenity's `Member::avatar`.
    pub fn has_guild_avatar(&self) -> bool {
        self.avatar.is_some()
    }

    /// Typed accessor for the member's user ID.
    ///
    /// Returns the user ID from the nested `user` object, or falls back to the
    /// top-level `user_id` field.  Returns `None` if neither is present.
    pub fn user_id_typed(&self) -> Option<UserId> {
        self.user.as_ref().and_then(|u| u.id.parse().ok().map(UserId::new)).or_else(|| self.user_id.as_deref()?.parse().ok().map(UserId::new))
    }

    /// Typed accessor for the member's roles as [`RoleId`] values.
    pub fn role_ids(&self) -> Vec<RoleId> {
        self.roles.iter().filter_map(|s| s.parse().ok().map(RoleId::new)).collect()
    }
}

/// Session info from READY event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    pub status: String,
    pub session_id: String,
    #[serde(default)]
    pub client_info: ClientInfo,
    #[serde(default)]
    pub activities: Vec<serde_json::Value>,
}

/// Client info within a session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientInfo {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub client: String,
}

/// User profile data from the profile endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// The user data
    pub user: User,
    /// Connected accounts (Spotify, YouTube, etc.)
    #[serde(default)]
    pub connected_accounts: Vec<ConnectedAccount>,
    /// Premium (Nitro) since timestamp
    #[serde(default)]
    pub premium_since: Option<String>,
    /// Premium type (0 = None, 1 = Nitro Classic, 2 = Nitro, 3 = Nitro Basic)
    #[serde(default)]
    pub premium_type: Option<u8>,
    /// Premium guild since timestamp (for boosting)
    #[serde(default)]
    pub premium_guild_since: Option<String>,
    /// User bio
    #[serde(default)]
    pub user_profile: Option<UserProfileData>,
    /// Mutual guilds with the requesting user
    #[serde(default)]
    pub mutual_guilds: Vec<MutualGuild>,
    /// Mutual friends with the requesting user
    #[serde(default)]
    pub mutual_friends: Vec<User>,
    /// Guild member data (if guild_id was provided)
    #[serde(default)]
    pub guild_member: Option<Member>,
}

/// Detailed profile data (bio, banner, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfileData {
    /// User bio
    #[serde(default)]
    pub bio: Option<String>,
    /// Profile banner hash
    #[serde(default)]
    pub banner: Option<String>,
    /// Accent color
    #[serde(default)]
    pub accent_color: Option<u32>,
    /// Theme colors
    #[serde(default)]
    pub theme_colors: Option<Vec<u32>>,
    /// Pronouns
    #[serde(default)]
    pub pronouns: Option<String>,
}

/// Connected account data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedAccount {
    /// Account ID
    pub id: String,
    /// Account name
    pub name: String,
    /// Account type (spotify, youtube, twitter, etc.)
    #[serde(rename = "type")]
    pub account_type: String,
    /// Whether the account is verified
    #[serde(default)]
    pub verified: bool,
    /// Metadata (varies by account type)
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Mutual guild data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualGuild {
    /// Guild ID
    pub id: String,
    /// User's nickname in this guild
    #[serde(default)]
    pub nick: Option<String>,
}
