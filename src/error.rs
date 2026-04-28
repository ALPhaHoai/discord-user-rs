//! Error types for the discord-control library

use thiserror::Error;

/// Result type alias for discord operations
pub type Result<T> = std::result::Result<T, DiscordError>;

/// Errors that can occur during Discord operations
#[derive(Error, Debug)]
pub enum DiscordError {
    /// WebSocket connection error
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Rate limited by Discord
    #[error("Rate limited, retry after {retry_after}s (global: {global}, bucket: {bucket:?}, scope: {scope:?})")]
    RateLimited { retry_after: f64, bucket: Option<String>, global: bool, scope: Option<String> },

    /// Account verification required
    #[error("Account verification required")]
    VerificationRequired,

    /// Captcha required
    #[error("Captcha required: {service}")]
    CaptchaRequired { service: String },

    /// Gateway connection failed
    #[error("Gateway connection failed: {0}")]
    GatewayConnection(String),

    /// Authentication failed
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// WebSocket not initialized
    #[error("WebSocket not initialized, call init() first")]
    NotInitialized,

    /// Request timeout
    #[error("Request timed out")]
    Timeout,

    /// Invalid or expired token
    #[error("Invalid token")]
    InvalidToken,

    /// Resource not found
    #[error("{resource_type} not found: {id}")]
    NotFound { resource_type: String, id: String },

    /// Permission denied
    #[error("Missing permission: {permission}")]
    PermissionDenied { permission: String },

    /// Invalid request parameters
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Maximum retries exceeded
    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    /// Gateway requested a reconnection (opcode 7)
    #[error("Gateway requested reconnection")]
    GatewayReconnectRequested,

    /// An HTTP response with a non-2xx status that doesn't map to a more
    /// specific variant.  Preserves the raw status code and response body for
    /// inspection by callers.
    #[error("HTTP {status}: {body}")]
    UnexpectedStatusCode { status: u16, body: String },

    /// Discord's API returned a 5xx status indicating a server-side error.
    #[error("Discord service error ({status}): {body}")]
    ServiceError { status: u16, body: String },

    /// Model-level validation error — caught before any HTTP request is made.
    ///
    /// Returned by request structs / operation methods when the provided data
    /// violates a known Discord constraint (e.g. message too long, too many
    /// embeds).  No network request is ever sent when this variant is returned.
    #[error("Model validation error: {0}")]
    Model(ModelError),

    /// Generic error (use specific variants when possible)
    #[error("{0}")]
    Other(String),

    /// Error with context
    #[error("{context}")]
    Context {
        context: String,
        #[source]
        source: Box<DiscordError>,
    },
}

/// Specific model-level constraint violations.
///
/// Each variant maps to a Discord API constraint that can be checked locally
/// without making a network request.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ModelError {
    /// Message content exceeds the 2000-character limit.
    #[error("message content is too long: {0} characters (max 2000)")]
    MessageTooLong(usize),

    /// More than 10 embeds were supplied in a single message.
    #[error("too many embeds: {0} (max 10)")]
    EmbedAmount(usize),

    /// An embed's total character count exceeds 6000.
    #[error("embed is too large: {0} characters (max 6000)")]
    EmbedTooLarge(usize),

    /// More than 3 stickers were attached to a single message.
    #[error("too many stickers: {0} (max 3)")]
    StickerAmount(usize),

    /// Bulk-delete requires between 2 and 100 message IDs.
    #[error("invalid bulk-delete count: {0} (must be 2-100)")]
    BulkDeleteAmount(usize),

    /// A name is shorter than the minimum allowed length.
    #[error("name too short: {0} characters (min {1})")]
    NameTooShort(usize, usize),

    /// A name exceeds the maximum allowed length.
    #[error("name too long: {0} characters (max {1})")]
    NameTooLong(usize, usize),

    /// The member lacks a required permission.
    #[error("invalid permissions: required {required:#b}, present {present:#b}")]
    InvalidPermissions { required: u64, present: u64 },

    /// A role or channel hierarchy constraint was violated.
    #[error("hierarchy constraint violated")]
    Hierarchy,

    /// Cannot send messages to a bot account.
    #[error("cannot send messages to a bot")]
    MessagingBot,

    /// The channel type is incompatible with the requested operation.
    #[error("invalid channel type for this operation")]
    InvalidChannelType,

    /// Guild name must be 2-100 characters.
    #[error("guild name must be 2-100 characters (got {0})")]
    GuildNameLength(usize),

    /// Channel topic must be 0-1024 characters.
    #[error("channel topic must be 0-1024 characters (got {0})")]
    ChannelTopicLength(usize),

    /// Role name must be 1-100 characters.
    #[error("role name must be 1-100 characters (got {0})")]
    RoleNameLength(usize),

    /// Webhook name must be 1-80 characters.
    #[error("webhook name must be 1-80 characters (got {0})")]
    WebhookNameLength(usize),

    /// Invite max_age must be 0-604800 seconds.
    #[error("invite max_age must be 0-604800 seconds (got {0})")]
    InviteMaxAge(u32),

    /// Invite max_uses must be 0-100.
    #[error("invite max_uses must be 0-100 (got {0})")]
    InviteMaxUses(u32),
}

impl From<tokio_tungstenite::tungstenite::Error> for DiscordError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        DiscordError::WebSocket(Box::new(err))
    }
}

impl From<ModelError> for DiscordError {
    fn from(e: ModelError) -> Self {
        DiscordError::Model(e)
    }
}

impl DiscordError {
    /// Check if the error is retryable.
    ///
    /// This returns true for:
    /// - Rate limits
    /// - Timeouts
    /// - Connection errors
    /// - Gateway reconnection requests
    /// - Temporary HTTP errors (5xx, timeouts)
    pub fn is_retryable(&self) -> bool {
        match self {
            // Rate limits are temporary
            Self::RateLimited { .. } => true,

            // Timeouts are temporary
            Self::Timeout => true,

            // Gateway requested reconnection
            Self::GatewayReconnectRequested => true,

            // Connection failures are usually retryable
            Self::GatewayConnection(_) => true,
            Self::WebSocket(_) => true,

            // HTTP errors depend on the status
            Self::Http(e) => {
                if e.is_timeout() || e.is_connect() {
                    return true;
                }
                if let Some(status) = e.status() {
                    // 5xx errors are server errors and might be temporary
                    if status.is_server_error() {
                        return true;
                    }
                    // 429 is handled by RateLimited, but check just in case
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        return true;
                    }
                }
                false
            }

            // Recursively check source for Context errors
            Self::Context { source, .. } => source.is_retryable(),

            // 5xx service errors are retryable
            Self::ServiceError { .. } => true,

            // Auth errors, invalid data, permissions, etc. are not retryable
            Self::VerificationRequired | Self::CaptchaRequired { .. } | Self::AuthenticationFailed | Self::NotInitialized | Self::InvalidToken | Self::NotFound { .. } | Self::PermissionDenied { .. } | Self::InvalidRequest(_) | Self::MaxRetriesExceeded | Self::Json(_) | Self::Other(_) | Self::Model(_) | Self::UnexpectedStatusCode { .. } => false,
        }
    }
}

/// Trait for adding context to results
pub trait WithContext<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> WithContext<T> for std::result::Result<T, E>
where
    E: Into<DiscordError>,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| DiscordError::Context { context: context.to_string(), source: Box::new(e.into()) })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| DiscordError::Context { context: f().to_string(), source: Box::new(e.into()) })
    }
}
