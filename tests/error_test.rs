use discord_user::error::DiscordError;

#[test]
fn test_is_retryable() {
    // Retryable errors
    let rate_limited = DiscordError::RateLimited { retry_after: 5.0, bucket: None, global: false, scope: None };
    assert!(rate_limited.is_retryable(), "RateLimited should be retryable");

    let timeout = DiscordError::Timeout;
    assert!(timeout.is_retryable(), "Timeout should be retryable");

    let reconnect = DiscordError::GatewayReconnectRequested;
    assert!(reconnect.is_retryable(), "GatewayReconnectRequested should be retryable");

    let gateway_conn = DiscordError::GatewayConnection("Connection lost".to_string());
    assert!(gateway_conn.is_retryable(), "GatewayConnection should be retryable");

    // Non-retryable errors
    let auth_failed = DiscordError::AuthenticationFailed;
    assert!(!auth_failed.is_retryable(), "AuthenticationFailed should NOT be retryable");

    let not_found = DiscordError::NotFound { resource_type: "user".to_string(), id: "123".to_string() };
    assert!(!not_found.is_retryable(), "NotFound should NOT be retryable");

    let permission_denied = DiscordError::PermissionDenied { permission: "SEND_MESSAGES".to_string() };
    assert!(!permission_denied.is_retryable(), "PermissionDenied should NOT be retryable");

    let invalid_token = DiscordError::InvalidToken;
    assert!(!invalid_token.is_retryable(), "InvalidToken should NOT be retryable");
}

#[test]
fn test_context_retryable() {
    // Create a retryable source error
    let source = DiscordError::Timeout;

    // Wrap it in context manually or via trait
    // The trait returns Result<T>, so let's construct manually for testing the
    // error type method
    let context_error = DiscordError::Context { context: "Failed operation".to_string(), source: Box::new(source) };

    assert!(context_error.is_retryable(), "Context wrapping retryable error should be retryable");

    // Non-retryable source
    let source_fatal = DiscordError::AuthenticationFailed;
    let context_fatal = DiscordError::Context { context: "Login failed".to_string(), source: Box::new(source_fatal) };

    assert!(!context_fatal.is_retryable(), "Context wrapping fatal error should NOT be retryable");
}
