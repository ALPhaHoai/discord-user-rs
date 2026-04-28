//! HTTP client for Discord API requests

use std::{collections::HashMap, time::Duration};

use reqwest::{header, Client, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, warn};

use crate::error::{DiscordError, Result};

const API_BASE: &str = "https://discord.com/api/v9";
const DEFAULT_RETRY_COUNT: u32 = 5;
const MAX_RETRY_AFTER_SECONDS: u64 = 30;

#[derive(Debug, Clone)]
pub struct RatelimitInfo {
    pub timeout: std::time::Duration,
    pub limit: u64,
    pub method: reqwest::Method,
    pub path: String,
    pub global: bool,
}

struct RateLimitInfo {
    retry_after: f64,
    bucket: Option<String>,
    global: bool,
    scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RatelimitingBucket {
    pub remaining: u64,
    pub limit: u64,
    pub reset_at: f64,
}

#[derive(Clone, Default)]
pub struct Ratelimit {
    pub buckets: std::sync::Arc<dashmap::DashMap<String, RatelimitingBucket>>,
    pub callback: Option<std::sync::Arc<dyn Fn(RatelimitInfo) + Send + Sync>>,
    pub global: std::sync::Arc<tokio::sync::Mutex<()>>,
}

impl Ratelimit {
    pub fn get_route_key(method: &Method, endpoint: &str) -> String {
        let path = if let Some(stripped) = endpoint.split("/api/v9/").nth(1) { stripped } else { endpoint.trim_start_matches('/') };

        let path = path.split('?').next().unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        let mut route = String::from(method.as_str());

        if parts.is_empty() {
            return route;
        }

        let mut iter = parts.iter().peekable();
        while let Some(&part) = iter.next() {
            route.push('/');
            match part {
                "channels" | "guilds" | "webhooks" => {
                    route.push_str(part);
                    if let Some(&id) = iter.next() {
                        route.push('/');
                        route.push_str(id);
                    }
                }
                _ => {
                    if part.chars().all(|c| c.is_ascii_digit()) && part.len() > 10 {
                        route.push_str("{id}");
                    } else {
                        route.push_str(part);
                    }
                }
            }
        }

        route
    }

    pub async fn pre_hook(&self, method: &Method, endpoint: &str, route_key: &str) {
        let wait_info = {
            if let Some(bucket) = self.buckets.get(route_key) {
                if bucket.remaining == 0 {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64();

                    if bucket.reset_at > now {
                        Some((bucket.reset_at - now, bucket.limit))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some((secs, limit)) = wait_info {
            if secs > 0.0 {
                tracing::debug!("Preemptive rate limit hit for {}, waiting {:.3}s", route_key, secs);
                if let Some(cb) = &self.callback {
                    cb(RatelimitInfo {
                        timeout: std::time::Duration::from_secs_f64(secs),
                        limit,
                        method: method.clone(),
                        path: endpoint.to_string(),
                        global: false,
                    });
                }
                tokio::time::sleep(std::time::Duration::from_secs_f64(secs)).await;
            }
        }
    }

    pub fn post_hook(&self, route_key: &str, headers: &reqwest::header::HeaderMap) {
        let remaining = headers.get("x-ratelimit-remaining").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<u64>().ok());

        let limit = headers.get("x-ratelimit-limit").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<u64>().ok());

        let reset_at = headers.get("x-ratelimit-reset").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<f64>().ok());

        if let (Some(remaining), Some(limit), Some(reset_at)) = (remaining, limit, reset_at) {
            self.buckets.insert(route_key.to_string(), RatelimitingBucket { remaining, limit, reset_at });
        }
    }
}

/// HTTP client for making Discord API requests
#[derive(Clone)]
pub struct DiscordHttpClient {
    client: Client,
    token: String,
    custom_headers: HashMap<String, String>,
    ratelimit: Ratelimit,
    ratelimiter_disabled: bool,
}

impl DiscordHttpClient {
    /// Create a new Discord HTTP client with a token
    pub fn new(token: impl Into<String>, proxy: Option<String>, ratelimiter_disabled: bool) -> Self {
        let mut builder = Client::builder().timeout(Duration::from_secs(30)).pool_max_idle_per_host(10).pool_idle_timeout(Duration::from_secs(90)).tcp_keepalive(Duration::from_secs(60)).tcp_nodelay(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36").http1_only();

        if let Some(proxy_url) = proxy {
            if let Ok(p) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(p);
            }
        }

        let client = builder.build().unwrap_or_else(|e| panic!("Failed to create HTTP client: {}", e));

        Self {
            client,
            token: token.into(),
            custom_headers: HashMap::new(),
            ratelimit: Ratelimit::default(),
            ratelimiter_disabled,
        }
    }

    /// Create a new Discord HTTP client with custom headers
    pub fn with_headers(headers: HashMap<String, String>, proxy: Option<String>, ratelimiter_disabled: bool) -> Option<Self> {
        let token = headers.iter().find(|(k, _)| k.to_lowercase() == "authorization").map(|(_, v)| v.clone())?;

        let mut builder = Client::builder().timeout(Duration::from_secs(30)).pool_max_idle_per_host(10).pool_idle_timeout(Duration::from_secs(90)).tcp_keepalive(Duration::from_secs(60)).tcp_nodelay(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36").http1_only();

        if let Some(proxy_url) = proxy {
            if let Ok(p) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(p);
            }
        }

        let client = builder.build().unwrap_or_else(|e| panic!("Failed to create HTTP client: {}", e));

        Some(Self { client, token, custom_headers: headers, ratelimit: Ratelimit::default(), ratelimiter_disabled })
    }

    /// Get the token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Set a callback to execute when rate limited
    pub fn set_ratelimit_callback(&mut self, callback: std::sync::Arc<dyn Fn(RatelimitInfo) + Send + Sync>) {
        self.ratelimit.callback = Some(callback);
    }

    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, route: crate::route::Route<'_>) -> Result<T> {
        self.request(Method::GET, route, None::<()>).await
    }

    /// Make a POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, route: crate::route::Route<'_>, body: B) -> Result<T> {
        self.request(Method::POST, route, Some(body)).await
    }

    /// Make a PATCH request
    pub async fn patch<T: DeserializeOwned, B: Serialize>(&self, route: crate::route::Route<'_>, body: B) -> Result<T> {
        self.request(Method::PATCH, route, Some(body)).await
    }

    /// Make a PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, route: crate::route::Route<'_>, body: B) -> Result<T> {
        self.request(Method::PUT, route, Some(body)).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, route: crate::route::Route<'_>) -> Result<()> {
        self.request_no_response(Method::DELETE, route, None::<()>).await
    }

    /// Make a DELETE request and deserialize the response body.
    pub async fn delete_with_response<T: DeserializeOwned>(&self, route: crate::route::Route<'_>) -> Result<T> {
        self.request(Method::DELETE, route, None::<()>).await
    }

    /// Make a POST request with no body and no response body (e.g. typing
    /// indicator)
    pub async fn post_empty(&self, route: crate::route::Route<'_>) -> Result<()> {
        self.request_no_response(Method::POST, route, None::<()>).await
    }

    /// Make a POST request with a body but discard the response body.
    pub async fn post_no_response<B: Serialize>(&self, route: crate::route::Route<'_>, body: B) -> Result<()> {
        self.request_no_response(Method::POST, route, Some(body)).await
    }

    /// Make a PATCH request with a body but discard the response body.
    pub async fn patch_no_response<B: Serialize>(&self, route: crate::route::Route<'_>, body: B) -> Result<()> {
        self.request_no_response(Method::PATCH, route, Some(body)).await
    }

    /// Make a POST request with custom Referer header
    pub async fn post_with_referer<T: DeserializeOwned, B: Serialize>(&self, route: crate::route::Route<'_>, body: B, referer: &str) -> Result<T> {
        self.request_with_referer(Method::POST, route, Some(body), Some(referer)).await
    }

    /// POST a multipart message with file attachments.
    ///
    /// `payload_json` carries the message body (content, embeds, etc.).
    /// Each attachment is sent as a `files[N]` part.
    pub async fn post_multipart<T: DeserializeOwned>(&self, route: crate::route::Route<'_>, payload_json: serde_json::Value, attachments: Vec<crate::types::CreateAttachment>) -> Result<T> {
        use reqwest::multipart::{Form, Part};

        let path_cow = route.path();
        let endpoint = path_cow.as_ref();
        let url = if endpoint.starts_with("http") { endpoint.to_string() } else { format!("{}/{}", API_BASE, endpoint.trim_start_matches('/')) };
        let route_key = Ratelimit::get_route_key(&Method::POST, endpoint);

        for attempt in 0..DEFAULT_RETRY_COUNT {
            if !self.ratelimiter_disabled {
                drop(self.ratelimit.global.lock().await);
                self.ratelimit.pre_hook(&Method::POST, endpoint, &route_key).await;
            }

            let mut form = Form::new().part("payload_json", Part::text(payload_json.to_string()).mime_str("application/json").unwrap_or_else(|_| Part::text(payload_json.to_string())));

            for (i, att) in attachments.iter().enumerate() {
                let part = Part::bytes(att.data.clone()).file_name(att.filename.clone()).mime_str(&att.mime_type).unwrap_or_else(|_| Part::bytes(att.data.clone()).file_name(att.filename.clone()));
                form = form.part(format!("files[{}]", i), part);
            }

            let mut request = self.client.post(&url).header(header::AUTHORIZATION, &self.token).multipart(form);
            for (key, value) in &self.custom_headers {
                if key.to_lowercase() != "authorization" {
                    request = request.header(key.as_str(), value.as_str());
                }
            }

            debug!("Multipart request attempt {}/{}: POST {}", attempt + 1, DEFAULT_RETRY_COUNT, url);
            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if !self.ratelimiter_disabled {
                        self.ratelimit.post_hook(&route_key, response.headers());
                    }
                    if status.is_success() {
                        let text = response.text().await?;
                        return serde_json::from_str(&text).map_err(|e| {
                            error!(error = %e, body = %text, "Failed to parse multipart response");
                            DiscordError::Json(e)
                        });
                    }
                    if status.as_u16() == 429 {
                        let info = Self::extract_rate_limit_info(response.headers());
                        let wait_time = if info.retry_after < 2.0 { 2.0 } else { info.retry_after };
                        warn!("Rate limited on multipart, waiting {:.2}s", wait_time);
                        tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
                        continue;
                    }
                    let error_body = response.text().await.unwrap_or_default();
                    return Err(if status.is_server_error() { DiscordError::ServiceError { status: status.as_u16(), body: error_body } } else { DiscordError::UnexpectedStatusCode { status: status.as_u16(), body: error_body } });
                }
                Err(e) => {
                    if attempt < DEFAULT_RETRY_COUNT - 1 {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(DiscordError::Http(e));
                }
            }
        }
        Err(DiscordError::MaxRetriesExceeded)
    }

    /// POST a pre-built multipart form to a path string and deserialize the
    /// response.
    ///
    /// Use this for endpoints like sticker creation that need custom multipart
    /// fields (not the `files[N]` + `payload_json` structure of message
    /// attachments).
    pub async fn post_raw_multipart<T: DeserializeOwned>(&self, path: String, form: reqwest::multipart::Form) -> Result<T> {
        let url = if path.starts_with("http") { path.clone() } else { format!("{}/{}", API_BASE, path.trim_start_matches('/')) };
        let route_key = Ratelimit::get_route_key(&Method::POST, &path);

        // Form is consumed on first use; rebuild is not possible, so we
        // accept a single attempt (multipart bodies cannot be cloned cheaply).
        if !self.ratelimiter_disabled {
            drop(self.ratelimit.global.lock().await);
            self.ratelimit.pre_hook(&Method::POST, &path, &route_key).await;
        }

        let mut request = self.client.post(&url).header(header::AUTHORIZATION, &self.token).multipart(form);
        for (key, value) in &self.custom_headers {
            if key.to_lowercase() != "authorization" {
                request = request.header(key.as_str(), value.as_str());
            }
        }
        debug!("Raw multipart POST: {}", url);
        let response = request.send().await?;
        let status = response.status();
        if !self.ratelimiter_disabled {
            self.ratelimit.post_hook(&route_key, response.headers());
        }
        if status.is_success() {
            let text = response.text().await?;
            return serde_json::from_str(&text).map_err(DiscordError::Json);
        }
        let error_body = response.text().await.unwrap_or_default();
        Err(if status.is_server_error() { DiscordError::ServiceError { status: status.as_u16(), body: error_body } } else { DiscordError::UnexpectedStatusCode { status: status.as_u16(), body: error_body } })
    }

    /// Make a request and return the response
    async fn request<T: DeserializeOwned, B: Serialize>(&self, method: Method, route: crate::route::Route<'_>, body: Option<B>) -> Result<T> {
        self.request_with_referer(method, route, body, None).await
    }

    /// Make a request with optional referer header
    async fn request_with_referer<T: DeserializeOwned, B: Serialize>(&self, method: Method, route: crate::route::Route<'_>, body: Option<B>, referer: Option<&str>) -> Result<T> {
        let response = self.do_request_with_referer(method, route, body, referer).await?;
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            error!(error = %e, body = %text, "Failed to parse response");
            DiscordError::Json(e)
        })
    }

    /// Make a request without expecting a response body
    async fn request_no_response<B: Serialize>(&self, method: Method, route: crate::route::Route<'_>, body: Option<B>) -> Result<()> {
        self.do_request_with_referer(method, route, body, None).await?;
        Ok(())
    }

    /// Internal request method with retry logic and optional referer
    async fn do_request_with_referer<B: Serialize>(&self, method: Method, route: crate::route::Route<'_>, body: Option<B>, referer: Option<&str>) -> Result<Response> {
        let path_cow = route.path();
        let endpoint = path_cow.as_ref();
        let url = if endpoint.starts_with("http") { endpoint.to_string() } else { format!("{}/{}", API_BASE, endpoint.trim_start_matches('/')) };
        let route_key = Ratelimit::get_route_key(&method, endpoint);

        for attempt in 0..DEFAULT_RETRY_COUNT {
            if !self.ratelimiter_disabled {
                // Wait for any active global rate limits to expire
                drop(self.ratelimit.global.lock().await);

                self.ratelimit.pre_hook(&method, endpoint, &route_key).await;
            }

            let mut request = self.client.request(method.clone(), &url);

            // Add authorization header
            request = request.header(header::AUTHORIZATION, &self.token);

            // Add referer header if provided
            if let Some(ref_url) = referer {
                request = request.header(header::REFERER, ref_url);
            }

            // Add custom headers
            for (key, value) in &self.custom_headers {
                if key.to_lowercase() != "authorization" {
                    request = request.header(key.as_str(), value.as_str());
                }
            }

            // Add body if present
            if let Some(ref b) = body {
                request = request.json(b);
            }

            debug!("Request attempt {}/{}: {} {}", attempt + 1, DEFAULT_RETRY_COUNT, method, url);

            let result = request.send().await;

            match result {
                Ok(response) => {
                    let status = response.status();
                    if !self.ratelimiter_disabled {
                        self.ratelimit.post_hook(&route_key, response.headers());
                    }

                    if status.is_success() {
                        return Ok(response);
                    }

                    // Handle rate limiting
                    if status.as_u16() == 429 {
                        let info = Self::extract_rate_limit_info(response.headers());
                        if self.ratelimiter_disabled {
                            return Err(DiscordError::RateLimited { retry_after: info.retry_after, bucket: info.bucket, global: info.global, scope: info.scope });
                        }

                        if info.retry_after > MAX_RETRY_AFTER_SECONDS as f64 {
                            return Err(DiscordError::RateLimited { retry_after: info.retry_after, bucket: info.bucket, global: info.global, scope: info.scope });
                        }

                        // Enforce minimum wait time of 2.0s to avoid spamming if server sends 0 or
                        // missing header
                        let wait_time = if info.retry_after < 2.0 { 2.0 } else { info.retry_after };
                        warn!("Rate limited (global: {}), waiting {:.2} seconds", info.global, wait_time);

                        // If it's a global rate limit, acquire the global lock across all routes
                        // This prevents any other requests from proceeding while we sleep
                        let _global_guard = if info.global { Some(self.ratelimit.global.lock().await) } else { None };

                        if let Some(cb) = &self.ratelimit.callback {
                            let limit = response.headers().get("x-ratelimit-limit").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

                            cb(RatelimitInfo {
                                timeout: Duration::from_secs_f64(wait_time),
                                limit,
                                method: method.clone(),
                                path: endpoint.to_string(),
                                global: info.global,
                            });
                        }

                        tokio::time::sleep(Duration::from_secs_f64(wait_time)).await;
                        // global_guard is dropped here so other routes can proceed
                        continue;
                    }

                    // Parse error response body; if reading fails, surface the read error directly
                    let error_body = match response.text().await {
                        Ok(text) => text,
                        Err(e) => {
                            return Err(DiscordError::Http(e));
                        }
                    };

                    // Check for specific error messages
                    if error_body.contains("verify your account") {
                        return Err(DiscordError::VerificationRequired);
                    }

                    if error_body.contains("captcha_key") {
                        let service = serde_json::from_str::<serde_json::Value>(&error_body).ok().and_then(|v| v["captcha_service"].as_str().map(|s| s.to_string())).unwrap_or_else(|| "unknown".to_string());
                        return Err(DiscordError::CaptchaRequired { service });
                    }

                    if status.as_u16() == 401 {
                        // Check if the error body indicates an invalid token
                        if error_body.contains("401: Unauthorized") || error_body.contains("token") {
                            return Err(DiscordError::InvalidToken);
                        }
                        return Err(DiscordError::AuthenticationFailed);
                    }

                    // Handle 403 Forbidden - permission denied
                    if status.as_u16() == 403 {
                        let permission = serde_json::from_str::<serde_json::Value>(&error_body).ok().and_then(|v| v["message"].as_str().map(|s| s.to_string())).unwrap_or_else(|| "unknown".to_string());
                        return Err(DiscordError::PermissionDenied { permission });
                    }

                    // Handle 404 Not Found
                    if status.as_u16() == 404 {
                        // Try to extract resource info from the URL
                        let resource_type = Self::extract_resource_type(&url);
                        let id = Self::extract_resource_id(&url);
                        return Err(DiscordError::NotFound { resource_type, id });
                    }

                    // Handle 400 Bad Request
                    if status.as_u16() == 400 {
                        return Err(DiscordError::InvalidRequest(error_body));
                    }

                    error!(status = %status, error_body = %error_body, "HTTP error");

                    // Retry on server errors
                    if status.is_server_error() && attempt < DEFAULT_RETRY_COUNT - 1 {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }

                    if status.is_server_error() {
                        return Err(DiscordError::ServiceError { status: status.as_u16(), body: error_body });
                    }

                    return Err(DiscordError::UnexpectedStatusCode { status: status.as_u16(), body: error_body });
                }
                Err(e) => {
                    error!(error = %e, "Request error");
                    if attempt < DEFAULT_RETRY_COUNT - 1 {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(DiscordError::Http(e));
                }
            }
        }

        Err(DiscordError::MaxRetriesExceeded)
    }

    /// Extract rate limit info from headers
    fn extract_rate_limit_info(headers: &header::HeaderMap) -> RateLimitInfo {
        let retry_after = headers.get("retry-after").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(5.0);

        let bucket = headers.get("x-ratelimit-bucket").and_then(|h| h.to_str().ok()).map(|s| s.to_string());

        let global = headers.get("x-ratelimit-global").and_then(|h| h.to_str().ok()).map(|s| s == "true").unwrap_or(false);

        let scope = headers.get("x-ratelimit-scope").and_then(|h| h.to_str().ok()).map(|s| s.to_string());

        RateLimitInfo { retry_after, bucket, global, scope }
    }

    /// Extract resource type from Discord API URL
    fn extract_resource_type(url: &str) -> String {
        // Parse URL to find resource type (channels, guilds, users, messages, etc.)
        let parts: Vec<&str> = url.split('/').collect();
        for part in &parts {
            match *part {
                "channels" => return "channel".to_string(),
                "guilds" => return "guild".to_string(),
                "users" => return "user".to_string(),
                "messages" => return "message".to_string(),
                "members" => return "member".to_string(),
                "roles" => return "role".to_string(),
                "invites" => return "invite".to_string(),
                "webhooks" => return "webhook".to_string(),
                "emojis" => return "emoji".to_string(),
                _ => continue,
            }
        }
        "resource".to_string()
    }

    /// Extract resource ID from Discord API URL
    fn extract_resource_id(url: &str) -> String {
        // Find the last numeric ID in the URL path
        let parts: Vec<&str> = url.split('/').collect();
        for part in parts.iter().rev() {
            // Discord IDs are snowflakes (large integers)
            if part.chars().all(|c| c.is_ascii_digit()) && part.len() > 10 {
                return (*part).to_string();
            }
        }
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::*;

    #[test]
    fn test_extract_rate_limit_info() {
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("12.5"));
        headers.insert("x-ratelimit-bucket", HeaderValue::from_static("test-bucket"));
        headers.insert("x-ratelimit-global", HeaderValue::from_static("true"));
        headers.insert("x-ratelimit-scope", HeaderValue::from_static("shared"));

        let info = DiscordHttpClient::extract_rate_limit_info(&headers);

        assert_eq!(info.retry_after, 12.5);
        assert_eq!(info.bucket.unwrap(), "test-bucket");
        assert!(info.global);
        assert_eq!(info.scope.unwrap(), "shared");
    }

    #[test]
    fn test_extract_rate_limit_info_defaults() {
        let headers = HeaderMap::new();
        let info = DiscordHttpClient::extract_rate_limit_info(&headers);

        assert_eq!(info.retry_after, 5.0);
        assert!(info.bucket.is_none());
        assert!(!info.global);
        assert!(info.scope.is_none());
    }
}
