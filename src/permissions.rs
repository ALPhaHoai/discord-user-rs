//! Effective permission calculation.
//!
//! Computes the permissions a member actually has in a channel, accounting for:
//! 1. Guild owner always has all permissions.
//! 2. Guild-wide role permissions (everyone role + member roles, sorted by
//!    position).
//! 3. Channel-level permission overwrites (role overwrites, then member
//!    overwrite).
//!
//! # Example
//! ```
//! use discord_user::{
//!     permissions::compute_permissions,
//!     types::{Channel, Member, PermissionOverwrite, Permissions, Role},
//! };
//!
//! // Given a member, their guild roles, a channel, and the guild owner ID,
//! // compute effective permissions:
//! // let perms = compute_permissions(&member, &guild_roles, &channel, owner_id);
//! // if perms.contains(Permissions::SEND_MESSAGES) { ... }
//! ```

use crate::types::{Channel, Member, Permissions, Role};

/// Compute the effective [`Permissions`] a `member` has in a `channel`.
///
/// # Arguments
/// * `member`      – The guild member whose permissions to calculate.
/// * `guild_roles` – All roles defined in the guild (used to look up role
///   permissions).
/// * `channel`     – The channel (supplies permission overwrites).
/// * `owner_id`    – The guild owner's user ID (always gets `ADMINISTRATOR`).
pub fn compute_permissions(member: &Member, guild_roles: &[Role], channel: &Channel, owner_id: &str) -> Permissions {
    // --- Step 1: Guild owner bypasses everything ---
    if member.user.as_ref().map(|u| u.id.as_str()) == Some(owner_id) {
        return Permissions::all();
    }

    // --- Step 2: Base permissions from @everyone role ---
    let everyone_id = channel.guild_id.as_deref().unwrap_or("");
    let mut base = Permissions::empty();
    for role in guild_roles {
        if role.id == everyone_id {
            base |= parse_perms(&role.permissions);
        }
    }

    // --- Step 3: Apply member's other roles ---
    let member_role_ids: std::collections::HashSet<&str> = member.roles.iter().map(String::as_str).collect();
    for role in guild_roles {
        if member_role_ids.contains(role.id.as_str()) {
            base |= parse_perms(&role.permissions);
        }
    }

    // Administrator bypasses channel overwrites ---
    if base.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }

    // --- Step 4: Channel overwrites ---
    // 4a. @everyone overwrite
    for ow in &channel.permission_overwrites {
        if ow.overwrite_type == 0 && ow.id == everyone_id {
            base &= !parse_perms(&ow.deny);
            base |= parse_perms(&ow.allow);
        }
    }

    // 4b. Role overwrites (accumulated in one pass to respect deny/allow order)
    let mut role_allow = Permissions::empty();
    let mut role_deny = Permissions::empty();
    for ow in &channel.permission_overwrites {
        if ow.overwrite_type == 0 && member_role_ids.contains(ow.id.as_str()) {
            role_deny |= parse_perms(&ow.deny);
            role_allow |= parse_perms(&ow.allow);
        }
    }
    base &= !role_deny;
    base |= role_allow;

    // 4c. Member-specific overwrite
    let member_id = member.user.as_ref().map(|u| u.id.as_str()).unwrap_or("");
    for ow in &channel.permission_overwrites {
        if ow.overwrite_type == 1 && ow.id == member_id {
            base &= !parse_perms(&ow.deny);
            base |= parse_perms(&ow.allow);
        }
    }

    base
}

/// Parse a Discord permission bitfield string (decimal or empty) into
/// [`Permissions`].
pub fn parse_perms(s: &str) -> Permissions {
    if s.is_empty() {
        return Permissions::empty();
    }
    s.parse::<u64>().map(Permissions::from_bits_truncate).unwrap_or(Permissions::empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Channel, ChannelType, Member, PermissionOverwrite, Permissions, Role, User};

    fn make_user(id: &str) -> User {
        User {
            id: id.to_string(),
            username: "test".to_string(),
            discriminator: "0000".to_string(),
            global_name: None,
            avatar: None,
            avatar_decoration_data: None,
            banner: None,
            banner_color: None,
            accent_color: None,
            public_flags: Default::default(),
            flags: Default::default(),
            premium_type: 0,
            bot: false,
            bio: None,
            email: None,
            verified: false,
            mfa_enabled: false,
            phone: None,
            nsfw_allowed: None,
            mobile: false,
            desktop: false,
        }
    }

    fn make_member(user_id: &str, roles: Vec<String>) -> Member {
        Member {
            user: Some(make_user(user_id)),
            user_id: None,
            nick: None,
            avatar: None,
            roles,
            joined_at: None,
            premium_since: None,
            deaf: false,
            mute: false,
            pending: false,
            flags: 0,
            communication_disabled_until: None,
        }
    }

    fn make_role(id: &str, permissions: u64) -> Role {
        Role {
            id: id.to_string(),
            name: "test".to_string(),
            color: 0,
            hoist: false,
            icon: None,
            unicode_emoji: None,
            position: 0,
            permissions: permissions.to_string(),
            managed: false,
            mentionable: false,
            flags: 0,
            tags: None,
        }
    }

    fn make_channel(guild_id: &str, overwrites: Vec<PermissionOverwrite>) -> Channel {
        Channel {
            id: "chan1".to_string(),
            guild_id: Some(guild_id.to_string()),
            channel_type: ChannelType::GuildText,
            name: Some("general".to_string()),
            topic: None,
            nsfw: false,
            position: Some(0),
            parent_id: None,
            permission_overwrites: overwrites,
            user_limit: None,
            rate_limit_per_user: None,
            last_message_id: None,
            bitrate: None,
            recipients: vec![],
            recipient_ids: vec![],
            flags: 0,
        }
    }

    #[test]
    fn owner_gets_all_permissions() {
        let member = make_member("owner1", vec![]);
        let roles = vec![make_role("guild1", 0)]; // @everyone with no perms
        let channel = make_channel("guild1", vec![]);
        let perms = compute_permissions(&member, &roles, &channel, "owner1");
        assert_eq!(perms, Permissions::all());
    }

    #[test]
    fn base_permissions_from_everyone_role() {
        let member = make_member("user1", vec![]);
        let send_msgs = Permissions::SEND_MESSAGES.bits();
        let roles = vec![make_role("guild1", send_msgs)]; // @everyone id == guild_id
        let channel = make_channel("guild1", vec![]);
        let perms = compute_permissions(&member, &roles, &channel, "owner1");
        assert!(perms.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn channel_overwrite_denies_permission() {
        let member = make_member("user1", vec![]);
        let send_msgs = Permissions::SEND_MESSAGES.bits();
        let roles = vec![make_role("guild1", send_msgs)];
        let ow = PermissionOverwrite {
            id: "guild1".to_string(), // @everyone overwrite
            overwrite_type: 0,
            allow: "0".to_string(),
            deny: send_msgs.to_string(),
        };
        let channel = make_channel("guild1", vec![ow]);
        let perms = compute_permissions(&member, &roles, &channel, "owner1");
        assert!(!perms.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn member_overwrite_grants_permission() {
        let member = make_member("user1", vec![]);
        let roles = vec![make_role("guild1", 0)]; // no base perms
        let ow = PermissionOverwrite {
            id: "user1".to_string(),
            overwrite_type: 1, // member overwrite
            allow: Permissions::SEND_MESSAGES.bits().to_string(),
            deny: "0".to_string(),
        };
        let channel = make_channel("guild1", vec![ow]);
        let perms = compute_permissions(&member, &roles, &channel, "owner1");
        assert!(perms.contains(Permissions::SEND_MESSAGES));
    }
}
