use std::borrow::Cow;

use urlencoding;

/// A declarative route registry for all Discord API endpoints.
/// Generates the path strings and parameters for HTTP requests.
#[derive(Debug, Clone)]
pub enum Route<'a> {
    // Users
    /// Get current user
    GetMe,
    /// Update current user profile
    UpdateMe,
    /// Settings Proto
    SettingsProto { version: u8 },
    /// Get user profile
    GetUserProfile { user_id: u64, guild_id: Option<u64> },

    // Relationships
    /// Get relationships
    GetRelationships,
    /// Add relationship
    AddRelationship { user_id: u64 },
    /// Remove relationship
    RemoveRelationship { user_id: u64 },

    // Channels
    /// Create DM / Get Channels
    CreateDm,
    /// Get Channel
    GetChannel { channel_id: u64 },
    /// Edit Channel (PATCH /channels/{id})
    EditChannel { channel_id: u64 },
    /// Delete / Close Channel (DELETE /channels/{id})
    DeleteChannel { channel_id: u64 },
    /// Create Guild Channel (POST /guilds/{id}/channels)
    CreateGuildChannel { guild_id: u64 },
    /// Update Voice Status
    UpdateVoiceStatus { channel_id: u64 },

    /// Trigger Typing Indicator
    TriggerTyping { channel_id: u64 },

    // Messages
    /// Get Messages
    GetMessages { channel_id: u64, limit: Option<u32>, before: Option<u64>, after: Option<u64> },
    /// Get Pinned Messages
    GetPins { channel_id: u64 },
    /// Pin Message
    PinMessage { channel_id: u64, message_id: u64 },
    /// Unpin Message
    UnpinMessage { channel_id: u64, message_id: u64 },
    /// Crosspost Message (publish to followers)
    CrosspostMessage { channel_id: u64, message_id: u64 },
    /// Create Message
    CreateMessage { channel_id: u64 },
    /// Get Message
    GetMessage { channel_id: u64, message_id: u64 },
    /// Edit Message
    EditMessage { channel_id: u64, message_id: u64 },
    /// Delete Message
    DeleteMessage { channel_id: u64, message_id: u64 },
    /// Bulk Delete Messages
    BulkDeleteMessages { channel_id: u64 },

    // Reactions
    /// Add Reaction
    AddReaction { channel_id: u64, message_id: u64, emoji: &'a str },
    /// Remove Own Reaction (@me)
    RemoveOwnReaction { channel_id: u64, message_id: u64, emoji: &'a str },
    /// Remove User Reaction
    RemoveUserReaction { channel_id: u64, message_id: u64, emoji: &'a str, user_id: u64 },

    // Guilds
    /// Get Guild
    GetGuild { guild_id: u64, with_counts: bool },
    /// Get Guild Roles
    GetGuildRoles { guild_id: u64 },
    /// Create Guild Role
    CreateGuildRole { guild_id: u64 },
    /// Edit Guild Role
    EditGuildRole { guild_id: u64, role_id: u64 },
    /// Delete Guild Role
    DeleteGuildRole { guild_id: u64, role_id: u64 },
    /// Edit Guild Member
    EditGuildMember { guild_id: u64, member_id: u64 },
    /// Create Channel Invite
    CreateChannelInvite { channel_id: u64 },
    /// Get Guild Invites
    GetGuildInvites { guild_id: u64 },
    /// Join Guild
    JoinGuild { code: &'a str },
    /// Delete Invite
    DeleteInvite { code: &'a str },
    /// Get Guild Stickers
    GetGuildStickers { guild_id: u64 },
    /// Get a single guild sticker
    GetGuildSticker { guild_id: u64, sticker_id: u64 },
    /// Create a guild sticker (multipart)
    CreateGuildSticker { guild_id: u64 },
    /// Edit a guild sticker
    EditGuildSticker { guild_id: u64, sticker_id: u64 },
    /// Delete a guild sticker
    DeleteGuildSticker { guild_id: u64, sticker_id: u64 },
    /// Get Guild Audit Logs
    GetGuildAuditLogs { guild_id: u64, user_id: Option<u64>, action_type: Option<u32>, before: Option<u64>, after: Option<u64>, limit: Option<u8> },
    /// Search Guild Members (paginated POST) — POST /guilds/{id}/members-search
    SearchGuildMembers { guild_id: u64 },
    /// Search Guild Members by username prefix — GET
    /// /guilds/{id}/members/search?query=...&limit=...
    GetGuildMembersByQuery { guild_id: u64, query: String, limit: u32 },
    /// Kick a guild member (DELETE /guilds/{guild_id}/members/{user_id})
    KickMember { guild_id: u64, user_id: u64 },
    /// Get all guild bans
    GetGuildBans { guild_id: u64 },
    /// Get a single guild ban
    GetGuildBan { guild_id: u64, user_id: u64 },
    /// Create guild ban (PUT /guilds/{guild_id}/bans/{user_id})
    CreateGuildBan { guild_id: u64, user_id: u64 },
    /// Remove guild ban (DELETE /guilds/{guild_id}/bans/{user_id})
    RemoveGuildBan { guild_id: u64, user_id: u64 },
    /// Edit guild settings (PATCH /guilds/{guild_id})
    EditGuild { guild_id: u64 },

    // Threads
    /// Create a thread (not from a message): POST /channels/{id}/threads
    CreateThread { channel_id: u64 },
    /// Create a thread from a message: POST
    /// /channels/{id}/messages/{msg}/threads
    CreateThreadFromMessage { channel_id: u64, message_id: u64 },
    /// Join a thread: PUT /channels/{id}/thread-members/@me
    JoinThread { channel_id: u64 },
    /// Leave a thread: DELETE /channels/{id}/thread-members/@me
    LeaveThread { channel_id: u64 },
    /// Add a member to a thread: PUT /channels/{id}/thread-members/{user_id}
    AddThreadMember { channel_id: u64, user_id: u64 },
    /// Remove a member from a thread: DELETE
    /// /channels/{id}/thread-members/{user_id}
    RemoveThreadMember { channel_id: u64, user_id: u64 },
    /// Get thread members: GET /channels/{id}/thread-members
    GetThreadMembers { channel_id: u64 },
    /// Get active threads in a guild: GET /guilds/{id}/threads/active
    GetActiveThreads { guild_id: u64 },
    /// Create a new guild: POST /guilds
    CreateGuild,
    /// Delete a guild: DELETE /guilds/{guild_id}
    DeleteGuild { guild_id: u64 },
    /// Leave a guild (current user): DELETE /users/@me/guilds/{guild_id}
    LeaveGuild { guild_id: u64 },

    // Emojis
    /// List guild emojis: GET /guilds/{id}/emojis
    GetGuildEmojis { guild_id: u64 },
    /// Get a single emoji: GET /guilds/{id}/emojis/{emoji_id}
    GetGuildEmoji { guild_id: u64, emoji_id: u64 },
    /// Create emoji: POST /guilds/{id}/emojis
    CreateGuildEmoji { guild_id: u64 },
    /// Edit emoji: PATCH /guilds/{id}/emojis/{emoji_id}
    EditGuildEmoji { guild_id: u64, emoji_id: u64 },
    /// Delete emoji: DELETE /guilds/{id}/emojis/{emoji_id}
    DeleteGuildEmoji { guild_id: u64, emoji_id: u64 },

    // Webhooks
    /// Get channel webhooks: GET /channels/{id}/webhooks
    GetChannelWebhooks { channel_id: u64 },
    /// Get guild webhooks: GET /guilds/{id}/webhooks
    GetGuildWebhooks { guild_id: u64 },
    /// Create webhook: POST /channels/{id}/webhooks
    CreateWebhook { channel_id: u64 },
    /// Get webhook: GET /webhooks/{id}
    GetWebhook { webhook_id: u64 },
    /// Get webhook with token: GET /webhooks/{id}/{token}
    GetWebhookWithToken { webhook_id: u64, token: &'a str },
    /// Edit webhook: PATCH /webhooks/{id}
    EditWebhook { webhook_id: u64 },
    /// Edit webhook with token: PATCH /webhooks/{id}/{token}
    EditWebhookWithToken { webhook_id: u64, token: &'a str },
    /// Delete webhook: DELETE /webhooks/{id}
    DeleteWebhook { webhook_id: u64 },
    /// Delete webhook with token: DELETE /webhooks/{id}/{token}
    DeleteWebhookWithToken { webhook_id: u64, token: &'a str },
    /// Execute webhook: POST /webhooks/{id}/{token}
    ExecuteWebhook { webhook_id: u64, token: &'a str },

    // Application commands
    /// List global application commands
    GetGlobalCommands { application_id: u64 },
    /// Create a global application command
    CreateGlobalCommand { application_id: u64 },
    /// Get a global application command
    GetGlobalCommand { application_id: u64, command_id: u64 },
    /// Edit a global application command
    EditGlobalCommand { application_id: u64, command_id: u64 },
    /// Delete a global application command
    DeleteGlobalCommand { application_id: u64, command_id: u64 },
    /// Bulk overwrite global application commands
    BulkOverwriteGlobalCommands { application_id: u64 },
    /// List guild application commands
    GetGuildCommands { application_id: u64, guild_id: u64 },
    /// Create a guild application command
    CreateGuildCommand { application_id: u64, guild_id: u64 },
    /// Get a guild application command
    GetGuildCommand { application_id: u64, guild_id: u64, command_id: u64 },
    /// Edit a guild application command
    EditGuildCommand { application_id: u64, guild_id: u64, command_id: u64 },
    /// Delete a guild application command
    DeleteGuildCommand { application_id: u64, guild_id: u64, command_id: u64 },
    /// Bulk overwrite guild application commands
    BulkOverwriteGuildCommands { application_id: u64, guild_id: u64 },

    // Interaction callbacks
    /// Respond to an interaction
    CreateInteractionResponse { interaction_id: u64, interaction_token: &'a str },
    /// Get the original interaction response
    GetOriginalInteractionResponse { application_id: u64, interaction_token: &'a str },
    /// Edit the original interaction response
    EditOriginalInteractionResponse { application_id: u64, interaction_token: &'a str },
    /// Delete the original interaction response
    DeleteOriginalInteractionResponse { application_id: u64, interaction_token: &'a str },
    /// Create a followup message
    CreateFollowupMessage { application_id: u64, interaction_token: &'a str },
    /// Edit a followup message
    EditFollowupMessage { application_id: u64, interaction_token: &'a str, message_id: u64 },
    /// Delete a followup message
    DeleteFollowupMessage { application_id: u64, interaction_token: &'a str, message_id: u64 },

    // Soundboard
    /// List default soundboard sounds: GET /soundboard-default-sounds
    ListDefaultSoundboardSounds,
    /// List guild soundboard sounds: GET /guilds/{id}/soundboard-sounds
    GetGuildSoundboardSounds { guild_id: u64 },
    /// Get a guild soundboard sound: GET
    /// /guilds/{id}/soundboard-sounds/{sound_id}
    GetGuildSoundboardSound { guild_id: u64, sound_id: u64 },
    /// Create a guild soundboard sound: POST /guilds/{id}/soundboard-sounds
    CreateGuildSoundboardSound { guild_id: u64 },
    /// Edit a guild soundboard sound: PATCH
    /// /guilds/{id}/soundboard-sounds/{sound_id}
    EditGuildSoundboardSound { guild_id: u64, sound_id: u64 },
    /// Delete a guild soundboard sound: DELETE
    /// /guilds/{id}/soundboard-sounds/{sound_id}
    DeleteGuildSoundboardSound { guild_id: u64, sound_id: u64 },
    /// Send a soundboard sound in a voice channel: POST
    /// /channels/{id}/send-soundboard-sound
    SendSoundboardSound { channel_id: u64 },

    // Polls
    /// Get voters for a poll answer: GET
    /// /channels/{id}/polls/{message_id}/answers/{answer_id}
    GetPollAnswerVoters { channel_id: u64, message_id: u64, answer_id: u64 },
    /// End a poll early: POST /channels/{id}/polls/{message_id}/expire
    EndPoll { channel_id: u64, message_id: u64 },

    // Auto-moderation
    /// List auto-mod rules: GET /guilds/{id}/auto-moderation/rules
    GetAutoModerationRules { guild_id: u64 },
    /// Get an auto-mod rule: GET /guilds/{id}/auto-moderation/rules/{rule_id}
    GetAutoModerationRule { guild_id: u64, rule_id: u64 },
    /// Create an auto-mod rule: POST /guilds/{id}/auto-moderation/rules
    CreateAutoModerationRule { guild_id: u64 },
    /// Edit an auto-mod rule: PATCH
    /// /guilds/{id}/auto-moderation/rules/{rule_id}
    EditAutoModerationRule { guild_id: u64, rule_id: u64 },
    /// Delete an auto-mod rule: DELETE
    /// /guilds/{id}/auto-moderation/rules/{rule_id}
    DeleteAutoModerationRule { guild_id: u64, rule_id: u64 },

    // Scheduled events
    /// List guild scheduled events: GET /guilds/{id}/scheduled-events
    GetGuildScheduledEvents { guild_id: u64 },
    /// Get a scheduled event: GET /guilds/{id}/scheduled-events/{event_id}
    GetGuildScheduledEvent { guild_id: u64, event_id: u64 },
    /// Create a scheduled event: POST /guilds/{id}/scheduled-events
    CreateGuildScheduledEvent { guild_id: u64 },
    /// Edit a scheduled event: PATCH /guilds/{id}/scheduled-events/{event_id}
    EditGuildScheduledEvent { guild_id: u64, event_id: u64 },
    /// Delete a scheduled event: DELETE
    /// /guilds/{id}/scheduled-events/{event_id}
    DeleteGuildScheduledEvent { guild_id: u64, event_id: u64 },
    /// Get users subscribed to a scheduled event
    GetGuildScheduledEventUsers { guild_id: u64, event_id: u64 },

    // Stage instances
    /// Get stage instance: GET /stage-instances/{channel_id}
    GetStageInstance { channel_id: u64 },
    /// Create stage instance: POST /stage-instances
    CreateStageInstance,
    /// Edit stage instance: PATCH /stage-instances/{channel_id}
    EditStageInstance { channel_id: u64 },
    /// Delete stage instance: DELETE /stage-instances/{channel_id}
    DeleteStageInstance { channel_id: u64 },

    // Voice
    /// List voice regions: GET /voice/regions
    GetVoiceRegions,
    /// List guild voice regions: GET /guilds/{id}/regions
    GetGuildVoiceRegions { guild_id: u64 },
    /// Edit own voice state in a guild: PATCH /guilds/{id}/voice-states/@me
    EditMyVoiceState { guild_id: u64 },
    /// Edit another user's voice state: PATCH
    /// /guilds/{id}/voice-states/{user_id}
    EditVoiceState { guild_id: u64, user_id: u64 },
}

impl<'a> Route<'a> {
    /// Returns the compiled path as a Cow<str>.
    pub fn path(&self) -> Cow<'_, str> {
        match self {
            Route::GetMe | Route::UpdateMe => Cow::Borrowed("users/@me"),
            Route::SettingsProto { version } => Cow::Owned(format!("users/@me/settings-proto/{}", version)),
            Route::GetUserProfile { user_id, guild_id } => {
                let mut url = format!("users/{}/profile?with_mutual_guilds=true&with_mutual_friends=true", user_id);
                if let Some(gid) = guild_id {
                    url.push_str(&format!("&guild_id={}", gid));
                }
                Cow::Owned(url)
            }

            Route::GetRelationships => Cow::Borrowed("users/@me/relationships"),
            Route::AddRelationship { user_id } | Route::RemoveRelationship { user_id } => Cow::Owned(format!("users/@me/relationships/{}", user_id)),

            Route::CreateDm => Cow::Borrowed("users/@me/channels"),
            Route::GetChannel { channel_id } | Route::EditChannel { channel_id } | Route::DeleteChannel { channel_id } => Cow::Owned(format!("channels/{}", channel_id)),
            Route::CreateGuildChannel { guild_id } => Cow::Owned(format!("guilds/{}/channels", guild_id)),
            Route::UpdateVoiceStatus { channel_id } => Cow::Owned(format!("channels/{}/voice-status", channel_id)),
            Route::TriggerTyping { channel_id } => Cow::Owned(format!("channels/{}/typing", channel_id)),

            Route::GetMessages { channel_id, limit, before, after } => {
                let mut url = format!("channels/{}/messages", channel_id);
                let mut params = Vec::new();
                if let Some(l) = limit {
                    params.push(format!("limit={}", l));
                }
                if let Some(b) = before {
                    params.push(format!("before={}", b));
                }
                if let Some(a) = after {
                    params.push(format!("after={}", a));
                }
                if !params.is_empty() {
                    url.push('?');
                    url.push_str(&params.join("&"));
                }
                Cow::Owned(url)
            }
            Route::CreateMessage { channel_id } | Route::BulkDeleteMessages { channel_id } => {
                let suffix = if matches!(self, Route::BulkDeleteMessages { .. }) { "/bulk-delete" } else { "" };
                Cow::Owned(format!("channels/{}/messages{}", channel_id, suffix))
            }
            Route::GetMessage { channel_id, message_id } | Route::EditMessage { channel_id, message_id } | Route::DeleteMessage { channel_id, message_id } => Cow::Owned(format!("channels/{}/messages/{}", channel_id, message_id)),
            Route::GetPins { channel_id } => Cow::Owned(format!("channels/{}/pins", channel_id)),
            Route::PinMessage { channel_id, message_id } | Route::UnpinMessage { channel_id, message_id } => Cow::Owned(format!("channels/{}/pins/{}", channel_id, message_id)),
            Route::CrosspostMessage { channel_id, message_id } => Cow::Owned(format!("channels/{}/messages/{}/crosspost", channel_id, message_id)),

            Route::AddReaction { channel_id, message_id, emoji } => Cow::Owned(format!("channels/{}/messages/{}/reactions/{}/@me?location=Message&type=0", channel_id, message_id, emoji)),
            Route::RemoveOwnReaction { channel_id, message_id, emoji } => Cow::Owned(format!("channels/{}/messages/{}/reactions/{}/@me", channel_id, message_id, emoji)),
            Route::RemoveUserReaction { channel_id, message_id, emoji, user_id } => Cow::Owned(format!("channels/{}/messages/{}/reactions/{}/{}", channel_id, message_id, emoji, user_id)),

            Route::GetGuild { guild_id, with_counts } => {
                if *with_counts {
                    Cow::Owned(format!("guilds/{}?with_counts=true", guild_id))
                } else {
                    Cow::Owned(format!("guilds/{}", guild_id))
                }
            }
            Route::GetGuildRoles { guild_id } | Route::CreateGuildRole { guild_id } => Cow::Owned(format!("guilds/{}/roles", guild_id)),
            Route::EditGuildRole { guild_id, role_id } | Route::DeleteGuildRole { guild_id, role_id } => Cow::Owned(format!("guilds/{}/roles/{}", guild_id, role_id)),
            Route::EditGuildMember { guild_id, member_id } => Cow::Owned(format!("guilds/{}/members/{}", guild_id, member_id)),
            Route::CreateChannelInvite { channel_id } => Cow::Owned(format!("channels/{}/invites", channel_id)),
            Route::GetGuildInvites { guild_id } => Cow::Owned(format!("guilds/{}/invites", guild_id)),
            Route::JoinGuild { code } | Route::DeleteInvite { code } => Cow::Owned(format!("invites/{}", code)),
            Route::GetGuildStickers { guild_id } | Route::CreateGuildSticker { guild_id } => Cow::Owned(format!("guilds/{}/stickers", guild_id)),
            Route::GetGuildSticker { guild_id, sticker_id } | Route::EditGuildSticker { guild_id, sticker_id } | Route::DeleteGuildSticker { guild_id, sticker_id } => Cow::Owned(format!("guilds/{}/stickers/{}", guild_id, sticker_id)),
            Route::GetGuildAuditLogs { guild_id, user_id, action_type, before, after, limit } => {
                let mut url = format!("guilds/{}/audit-logs", guild_id);
                let mut params: Vec<String> = Vec::new();
                if let Some(u) = user_id {
                    params.push(format!("user_id={}", u));
                }
                if let Some(a) = action_type {
                    params.push(format!("action_type={}", a));
                }
                if let Some(b) = before {
                    params.push(format!("before={}", b));
                }
                if let Some(a) = after {
                    params.push(format!("after={}", a));
                }
                if let Some(l) = limit {
                    params.push(format!("limit={}", (*l).min(100u8)));
                }
                if !params.is_empty() {
                    url.push('?');
                    url.push_str(&params.join("&"));
                }
                Cow::Owned(url)
            }
            Route::SearchGuildMembers { guild_id } => Cow::Owned(format!("guilds/{}/members-search", guild_id)),
            Route::GetGuildMembersByQuery { guild_id, query, limit } => Cow::Owned(format!("guilds/{}/members/search?query={}&limit={}", guild_id, urlencoding::encode(query), limit)),
            Route::EditGuild { guild_id } => Cow::Owned(format!("guilds/{}", guild_id)),
            Route::KickMember { guild_id, user_id } => Cow::Owned(format!("guilds/{}/members/{}", guild_id, user_id)),
            Route::GetGuildBans { guild_id } => Cow::Owned(format!("guilds/{}/bans", guild_id)),
            Route::GetGuildBan { guild_id, user_id } | Route::CreateGuildBan { guild_id, user_id } | Route::RemoveGuildBan { guild_id, user_id } => Cow::Owned(format!("guilds/{}/bans/{}", guild_id, user_id)),
            Route::CreateThread { channel_id } => Cow::Owned(format!("channels/{}/threads", channel_id)),
            Route::CreateThreadFromMessage { channel_id, message_id } => Cow::Owned(format!("channels/{}/messages/{}/threads", channel_id, message_id)),
            Route::JoinThread { channel_id } => Cow::Owned(format!("channels/{}/thread-members/@me", channel_id)),
            Route::LeaveThread { channel_id } => Cow::Owned(format!("channels/{}/thread-members/@me", channel_id)),
            Route::AddThreadMember { channel_id, user_id } => Cow::Owned(format!("channels/{}/thread-members/{}", channel_id, user_id)),
            Route::RemoveThreadMember { channel_id, user_id } => Cow::Owned(format!("channels/{}/thread-members/{}", channel_id, user_id)),
            Route::GetThreadMembers { channel_id } => Cow::Owned(format!("channels/{}/thread-members", channel_id)),
            Route::GetActiveThreads { guild_id } => Cow::Owned(format!("guilds/{}/threads/active", guild_id)),
            Route::CreateGuild => Cow::Borrowed("guilds"),
            Route::DeleteGuild { guild_id } => Cow::Owned(format!("guilds/{}", guild_id)),
            Route::LeaveGuild { guild_id } => Cow::Owned(format!("users/@me/guilds/{}", guild_id)),
            Route::GetGuildEmojis { guild_id } | Route::CreateGuildEmoji { guild_id } => Cow::Owned(format!("guilds/{}/emojis", guild_id)),
            Route::GetGuildEmoji { guild_id, emoji_id } | Route::EditGuildEmoji { guild_id, emoji_id } | Route::DeleteGuildEmoji { guild_id, emoji_id } => Cow::Owned(format!("guilds/{}/emojis/{}", guild_id, emoji_id)),
            Route::GetChannelWebhooks { channel_id } | Route::CreateWebhook { channel_id } => Cow::Owned(format!("channels/{}/webhooks", channel_id)),
            Route::GetGuildWebhooks { guild_id } => Cow::Owned(format!("guilds/{}/webhooks", guild_id)),
            Route::GetWebhook { webhook_id } | Route::EditWebhook { webhook_id } | Route::DeleteWebhook { webhook_id } => Cow::Owned(format!("webhooks/{}", webhook_id)),
            Route::GetWebhookWithToken { webhook_id, token } | Route::EditWebhookWithToken { webhook_id, token } | Route::DeleteWebhookWithToken { webhook_id, token } | Route::ExecuteWebhook { webhook_id, token } => Cow::Owned(format!("webhooks/{}/{}", webhook_id, token)),

            // Application commands
            Route::GetGlobalCommands { application_id } | Route::CreateGlobalCommand { application_id } | Route::BulkOverwriteGlobalCommands { application_id } => Cow::Owned(format!("applications/{}/commands", application_id)),
            Route::GetGlobalCommand { application_id, command_id } | Route::EditGlobalCommand { application_id, command_id } | Route::DeleteGlobalCommand { application_id, command_id } => Cow::Owned(format!("applications/{}/commands/{}", application_id, command_id)),
            Route::GetGuildCommands { application_id, guild_id } | Route::CreateGuildCommand { application_id, guild_id } | Route::BulkOverwriteGuildCommands { application_id, guild_id } => Cow::Owned(format!("applications/{}/guilds/{}/commands", application_id, guild_id)),
            Route::GetGuildCommand { application_id, guild_id, command_id } | Route::EditGuildCommand { application_id, guild_id, command_id } | Route::DeleteGuildCommand { application_id, guild_id, command_id } => Cow::Owned(format!("applications/{}/guilds/{}/commands/{}", application_id, guild_id, command_id)),

            // Interaction callbacks
            Route::CreateInteractionResponse { interaction_id, interaction_token } => Cow::Owned(format!("interactions/{}/{}/callback", interaction_id, interaction_token)),
            Route::GetOriginalInteractionResponse { application_id, interaction_token } | Route::EditOriginalInteractionResponse { application_id, interaction_token } | Route::DeleteOriginalInteractionResponse { application_id, interaction_token } => Cow::Owned(format!("webhooks/{}/{}/messages/@original", application_id, interaction_token)),
            Route::CreateFollowupMessage { application_id, interaction_token } => Cow::Owned(format!("webhooks/{}/{}", application_id, interaction_token)),
            Route::EditFollowupMessage { application_id, interaction_token, message_id } | Route::DeleteFollowupMessage { application_id, interaction_token, message_id } => Cow::Owned(format!("webhooks/{}/{}/messages/{}", application_id, interaction_token, message_id)),

            // Soundboard
            Route::ListDefaultSoundboardSounds => Cow::Borrowed("soundboard-default-sounds"),
            Route::GetGuildSoundboardSounds { guild_id } | Route::CreateGuildSoundboardSound { guild_id } => Cow::Owned(format!("guilds/{}/soundboard-sounds", guild_id)),
            Route::GetGuildSoundboardSound { guild_id, sound_id } | Route::EditGuildSoundboardSound { guild_id, sound_id } | Route::DeleteGuildSoundboardSound { guild_id, sound_id } => Cow::Owned(format!("guilds/{}/soundboard-sounds/{}", guild_id, sound_id)),
            Route::SendSoundboardSound { channel_id } => Cow::Owned(format!("channels/{}/send-soundboard-sound", channel_id)),

            // Polls
            Route::GetPollAnswerVoters { channel_id, message_id, answer_id } => Cow::Owned(format!("channels/{}/polls/{}/answers/{}", channel_id, message_id, answer_id)),
            Route::EndPoll { channel_id, message_id } => Cow::Owned(format!("channels/{}/polls/{}/expire", channel_id, message_id)),

            // Auto-moderation
            Route::GetAutoModerationRules { guild_id } | Route::CreateAutoModerationRule { guild_id } => Cow::Owned(format!("guilds/{}/auto-moderation/rules", guild_id)),
            Route::GetAutoModerationRule { guild_id, rule_id } | Route::EditAutoModerationRule { guild_id, rule_id } | Route::DeleteAutoModerationRule { guild_id, rule_id } => Cow::Owned(format!("guilds/{}/auto-moderation/rules/{}", guild_id, rule_id)),

            // Scheduled events
            Route::GetGuildScheduledEvents { guild_id } | Route::CreateGuildScheduledEvent { guild_id } => Cow::Owned(format!("guilds/{}/scheduled-events", guild_id)),
            Route::GetGuildScheduledEvent { guild_id, event_id } | Route::EditGuildScheduledEvent { guild_id, event_id } | Route::DeleteGuildScheduledEvent { guild_id, event_id } => Cow::Owned(format!("guilds/{}/scheduled-events/{}", guild_id, event_id)),
            Route::GetGuildScheduledEventUsers { guild_id, event_id } => Cow::Owned(format!("guilds/{}/scheduled-events/{}/users", guild_id, event_id)),

            // Stage instances
            Route::GetStageInstance { channel_id } | Route::EditStageInstance { channel_id } | Route::DeleteStageInstance { channel_id } => Cow::Owned(format!("stage-instances/{}", channel_id)),
            Route::CreateStageInstance => Cow::Borrowed("stage-instances"),

            // Voice
            Route::GetVoiceRegions => Cow::Borrowed("voice/regions"),
            Route::GetGuildVoiceRegions { guild_id } => Cow::Owned(format!("guilds/{}/regions", guild_id)),
            Route::EditMyVoiceState { guild_id } => Cow::Owned(format!("guilds/{}/voice-states/@me", guild_id)),
            Route::EditVoiceState { guild_id, user_id } => Cow::Owned(format!("guilds/{}/voice-states/{}", guild_id, user_id)),
        }
    }
}
