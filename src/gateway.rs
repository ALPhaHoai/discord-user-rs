//! Discord WebSocket Gateway connection

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, RwLock},
};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::{
    error::Result,
    events::DispatchEvent,
    types::{ConnectionStage, ReconnectType, UserStatus},
};

/// Hardcoded gateway endpoint (mimics Chrome browser session)
const GATEWAY_URL: &str = "wss://gateway-us-east1-c.discord.gg/?encoding=json&v=9";

/// Capabilities bitmask for the IDENTIFY payload (user-account equivalent of
/// gateway intents). Mirrors discord-control's hardcoded value of 16381.
const DEFAULT_CAPABILITIES: u32 = 16381;

/// Fallback build number used when the live fetch fails.
const CLIENT_BUILD_NUMBER_FALLBACK: u32 = 534982;

/// Fetch the Discord client build number from the login page once, then cache
/// it.
///
/// Discord embeds `"BUILD_NUMBER":"<digits>"` in the login page's inline
/// script. Falls back to `CLIENT_BUILD_NUMBER_FALLBACK` on any network or parse
/// error.
async fn client_build_number() -> u32 {
    static CACHED: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    if let Some(&n) = CACHED.get() {
        return n;
    }
    let n = fetch_build_number_from_discord().await.unwrap_or_else(|| {
        warn!("Could not fetch Discord build number, using fallback {CLIENT_BUILD_NUMBER_FALLBACK}");
        CLIENT_BUILD_NUMBER_FALLBACK
    });
    info!("Discord client build number: {n}");
    let _ = CACHED.set(n);
    n
}

async fn fetch_build_number_from_discord() -> Option<u32> {
    let html = reqwest::get("https://discord.com/login").await.ok()?.text().await.ok()?;
    let marker = "\"BUILD_NUMBER\":\"";
    let start = html.find(marker)? + marker.len();
    let end = html[start..].find('"')? + start;
    html[start..end].parse().ok()
}

type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>;
type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Gateway opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Opcode::Dispatch,
            1 => Opcode::Heartbeat,
            2 => Opcode::Identify,
            3 => Opcode::PresenceUpdate,
            4 => Opcode::VoiceStateUpdate,
            6 => Opcode::Resume,
            7 => Opcode::Reconnect,
            8 => Opcode::RequestGuildMembers,
            9 => Opcode::InvalidSession,
            10 => Opcode::Hello,
            11 => Opcode::HeartbeatAck,
            _ => Opcode::Dispatch,
        }
    }
}

/// Gateway payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPayload {
    pub op: u8,
    #[serde(default)]
    pub d: Option<Value>,
    #[serde(default)]
    pub s: Option<u64>,
    #[serde(default)]
    pub t: Option<String>,
}

/// Outcome of a single gateway read-loop iteration. Drives the supervisor's
/// reconnect / give-up decision.
#[derive(Debug)]
enum GatewayLoopOutcome {
    /// Caller requested shutdown — exit the supervisor.
    Shutdown,
    /// Discord returned a fatal close code; reconnecting will not help.
    Fatal { code: u16, reason: &'static str },
    /// Discord told us our session is gone — clear it so the next connection
    /// performs a full IDENTIFY rather than RESUME.
    ReconnectFresh,
    /// Connection ended unexpectedly — reconnect and resume if possible.
    Reconnect,
}

/// Classify a Discord WebSocket close code into a supervisor outcome.
///
/// Codes in the 4000–4999 range are Discord-specific; codes < 4000 are
/// standard WebSocket codes (1000 normal, 1001 going away, 1006 abnormal).
/// See <https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-gateway-close-event-codes>.
fn classify_close(code: Option<u16>) -> GatewayLoopOutcome {
    match code {
        Some(4004) => GatewayLoopOutcome::Fatal { code: 4004, reason: "Authentication failed" },
        Some(4010) => GatewayLoopOutcome::Fatal { code: 4010, reason: "Invalid shard" },
        Some(4011) => GatewayLoopOutcome::Fatal { code: 4011, reason: "Sharding required" },
        Some(4012) => GatewayLoopOutcome::Fatal { code: 4012, reason: "Invalid API version" },
        Some(4013) => GatewayLoopOutcome::Fatal { code: 4013, reason: "Invalid intents" },
        Some(4014) => GatewayLoopOutcome::Fatal { code: 4014, reason: "Disallowed intents" },
        // Invalid seq / rate-limited / session timed out — session is unrecoverable, force re-IDENTIFY.
        Some(4007..=4009) => GatewayLoopOutcome::ReconnectFresh,
        Some(_) | None => GatewayLoopOutcome::Reconnect,
    }
}

/// Gateway connection manager
pub struct Gateway {
    token: String,
    custom_status: UserStatus,
    /// Capabilities bitmask sent in the IDENTIFY payload (user-account
    /// equivalent of intents)
    capabilities: u32,
    writer: Arc<RwLock<Option<WsWriter>>>,
    event_sender: broadcast::Sender<DispatchEvent>,
    heartbeat_interval: Arc<RwLock<u64>>,
    sequence: Arc<RwLock<Option<u64>>>,
    session_id: Arc<RwLock<Option<String>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Flag indicating we're waiting for a heartbeat ACK
    awaiting_heartbeat_ack: Arc<AtomicBool>,
    /// Timestamp of the last heartbeat sent (used for zombie detection)
    last_heartbeat_sent: Arc<RwLock<Option<Instant>>>,
    /// Round-trip latency computed from the most recent heartbeat/ACK pair.
    /// `None` until the first ACK is received.
    latency: Arc<RwLock<Option<Duration>>>,
    /// Current connection stage
    stage: Arc<RwLock<ConnectionStage>>,
    /// Number of guild IDs from READY still awaiting their GUILD_CREATE.
    /// Set to `guilds.len()` on READY; decremented on each GUILD_CREATE.
    /// When it reaches 0, a synthetic CACHE_READY event is fired.
    pending_guilds: Arc<AtomicUsize>,
}

impl Gateway {
    /// Create a new gateway connection
    pub fn new(token: String, custom_status: UserStatus, event_buffer_size: usize) -> (Self, broadcast::Receiver<DispatchEvent>) {
        Self::new_with_capabilities(token, custom_status, event_buffer_size, DEFAULT_CAPABILITIES)
    }

    /// Create a gateway connection with a custom capabilities bitmask.
    ///
    /// The `capabilities` value is sent as the `capabilities` field in the
    /// IDENTIFY payload — the user-account equivalent of gateway intents.
    /// Use [`GatewayIntents`](crate::types::GatewayIntents) bits or a raw
    /// `u32`.
    pub fn new_with_capabilities(token: String, custom_status: UserStatus, event_buffer_size: usize, capabilities: u32) -> (Self, broadcast::Receiver<DispatchEvent>) {
        let (event_sender, event_receiver) = broadcast::channel(event_buffer_size);

        (
            Self {
                token,
                custom_status,
                capabilities,
                writer: Arc::new(RwLock::new(None)),
                event_sender,
                heartbeat_interval: Arc::new(RwLock::new(41250)),
                sequence: Arc::new(RwLock::new(None)),
                session_id: Arc::new(RwLock::new(None)),
                shutdown_tx: None,
                awaiting_heartbeat_ack: Arc::new(AtomicBool::new(false)),
                last_heartbeat_sent: Arc::new(RwLock::new(None)),
                latency: Arc::new(RwLock::new(None)),
                stage: Arc::new(RwLock::new(ConnectionStage::Disconnected)),
                pending_guilds: Arc::new(AtomicUsize::new(0)),
            },
            event_receiver,
        )
    }

    /// Get event sender for subscribing
    pub fn event_sender(&self) -> broadcast::Sender<DispatchEvent> {
        self.event_sender.clone()
    }

    /// Connect to the gateway
    pub async fn connect(&mut self) -> Result<()> {
        *self.stage.write().await = ConnectionStage::Connecting;

        let (ws_stream, _) = connect_async(GATEWAY_URL).await?;
        let (writer, reader) = ws_stream.split();

        *self.writer.write().await = Some(writer);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Start message processing
        self.process_messages(reader, shutdown_rx).await?;

        Ok(())
    }

    /// Connect to the gateway with automatic reconnection and exponential
    /// backoff
    ///
    /// # Arguments
    /// * `max_retries` - Maximum number of connection attempts before giving up
    ///
    /// # Returns
    /// * `Ok(())` - Successfully connected
    /// * `Err(...)` - Failed to connect after max_retries attempts
    ///
    /// # Backoff Strategy
    /// - Base delay: 1 second
    /// - Maximum delay: 60 seconds
    /// - Exponential multiplier: 2x after each failure
    /// - Added jitter: random 0-500ms to prevent thundering herd
    pub async fn connect_with_auto_reconnect(&mut self, max_retries: u32) -> Result<()> {
        let base_delay_secs = 1u64;
        let max_delay_secs = 60u64;

        for attempt in 0..max_retries {
            match self.connect().await {
                Ok(_) => {
                    info!("Gateway connected successfully on attempt {}", attempt + 1);
                    return Ok(());
                }
                Err(e) => {
                    if attempt + 1 >= max_retries {
                        error!(error = %e, max_retries = max_retries, "Failed to connect, giving up");
                        return Err(e);
                    }

                    // Calculate exponential backoff delay
                    let delay_secs = std::cmp::min(base_delay_secs * 2u64.pow(attempt), max_delay_secs);

                    // Add jitter (0-500ms) to prevent thundering herd
                    let jitter_ms = rand::random::<u64>() % 500;
                    let total_delay = Duration::from_secs(delay_secs) + Duration::from_millis(jitter_ms);

                    warn!("Connection attempt {} failed: {}. Retrying in {:?}", attempt + 1, e, total_delay);

                    tokio::time::sleep(total_delay).await;
                }
            }
        }

        Err(crate::error::DiscordError::MaxRetriesExceeded)
    }

    /// Spawn the gateway supervisor task.
    ///
    /// Owns the WebSocket reader and runs an outer reconnect loop:
    /// 1. Read messages from the gateway until disconnection
    /// 2. Classify the disconnection reason (close code, error, timeout)
    /// 3. Decide whether to reconnect (and resume vs re-identify) or give up
    /// 4. On reconnect, replace `self.writer` with a fresh WS write half; the
    ///    next `Hello` triggers RESUME (if `session_id` is set) or IDENTIFY
    ///    automatically via [`Self::handle_message`].
    ///
    /// The task exits only on shutdown or a fatal close code (4004 auth fail,
    /// 4014 disallowed intents, etc. — see [`classify_close`]).
    async fn process_messages(&self, initial_reader: WsReader, mut shutdown_rx: mpsc::Receiver<()>) -> Result<()> {
        let token = self.token.clone();
        let custom_status = self.custom_status;
        let capabilities = self.capabilities;
        let writer = Arc::clone(&self.writer);
        let event_sender = self.event_sender.clone();
        let heartbeat_interval = Arc::clone(&self.heartbeat_interval);
        let sequence = Arc::clone(&self.sequence);
        let session_id = Arc::clone(&self.session_id);
        let awaiting_ack = Arc::clone(&self.awaiting_heartbeat_ack);
        let last_sent = Arc::clone(&self.last_heartbeat_sent);
        let latency = Arc::clone(&self.latency);
        let stage = Arc::clone(&self.stage);
        let pending_guilds = Arc::clone(&self.pending_guilds);

        tokio::spawn(async move {
            // The first iteration uses the reader from the initial connect();
            // subsequent iterations re-establish the WebSocket here.
            let mut current_reader: Option<WsReader> = Some(initial_reader);
            let mut backoff_secs: u64 = 1;
            const MAX_BACKOFF_SECS: u64 = 60;
            const RECONNECT_PAUSE: Duration = Duration::from_millis(500);

            'supervisor: loop {
                // ── 1. Acquire a reader: initial, or reconnect with backoff ──
                let mut reader = match current_reader.take() {
                    Some(r) => r,
                    None => {
                        info!(backoff_secs, "Reconnecting to Discord gateway…");
                        *stage.write().await = ConnectionStage::Connecting;
                        // Drop any stale writer before opening a new socket.
                        *writer.write().await = None;
                        // Reset per-connection heartbeat bookkeeping.
                        awaiting_ack.store(false, Ordering::SeqCst);
                        *last_sent.write().await = None;

                        match connect_async(GATEWAY_URL).await {
                            Ok((ws_stream, _)) => {
                                let (w, r) = ws_stream.split();
                                *writer.write().await = Some(w);
                                info!("Gateway reconnected successfully");
                                backoff_secs = 1;
                                r
                            }
                            Err(e) => {
                                error!(error = %e, backoff_secs, "Gateway reconnect failed; sleeping before retry");
                                tokio::select! {
                                    _ = shutdown_rx.recv() => break 'supervisor,
                                    _ = tokio::time::sleep(Duration::from_secs(backoff_secs)) => {}
                                }
                                backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
                                continue 'supervisor;
                            }
                        }
                    }
                };

                // ── 2. Inner read loop — runs until the connection ends ──
                let mut heartbeat_handle: Option<tokio::task::JoinHandle<()>> = None;
                let outcome: GatewayLoopOutcome = 'inner: loop {
                    // Read timeout = 2× heartbeat interval (or default 90s before Hello
                    // is received). Cap at 5 minutes to guard against a malformed Hello.
                    // Discord normally sends ~41 250 ms.
                    const MAX_HEARTBEAT_MS: u64 = 300_000;
                    let hb_ms = (*heartbeat_interval.read().await).min(MAX_HEARTBEAT_MS);
                    let read_timeout = Duration::from_millis(hb_ms.saturating_mul(2));

                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            info!("Gateway shutdown requested");
                            break 'inner GatewayLoopOutcome::Shutdown;
                        }
                        result = tokio::time::timeout(read_timeout, reader.next()) => {
                            let msg = match result {
                                Ok(msg) => msg,
                                Err(_) => {
                                    error!(read_timeout = ?read_timeout, "WebSocket read timed out with no message. Connection may be dead.");
                                    break 'inner GatewayLoopOutcome::Reconnect;
                                }
                            };
                            match msg {
                                Some(Ok(WsMessage::Text(text))) => {
                                    if let Err(e) = Self::handle_message(
                                        &text, &token, custom_status, capabilities,
                                        &writer, &event_sender, &heartbeat_interval,
                                        &sequence, &session_id, &mut heartbeat_handle,
                                        &awaiting_ack, &last_sent, &latency, &stage, &pending_guilds,
                                    ).await {
                                        error!(error = %e, "Error handling message");
                                        // Discord op-7 RECONNECT — bail out and reconnect now
                                        // rather than waiting for Discord to close the socket.
                                        if matches!(&e, crate::error::DiscordError::GatewayReconnectRequested) {
                                            break 'inner GatewayLoopOutcome::Reconnect;
                                        }
                                    }
                                }
                                Some(Ok(WsMessage::Binary(bytes))) => {
                                    // Discord may send zlib-compressed JSON payloads as binary frames.
                                    // Decompress with flate2 and feed to the normal message handler.
                                    match Self::decompress_zlib(&bytes) {
                                        Ok(text) => {
                                            if let Err(e) = Self::handle_message(
                                                &text, &token, custom_status, capabilities,
                                                &writer, &event_sender, &heartbeat_interval,
                                                &sequence, &session_id, &mut heartbeat_handle,
                                                &awaiting_ack, &last_sent, &latency, &stage, &pending_guilds,
                                            ).await {
                                                error!(error = %e, "Error handling decompressed binary message");
                                                if matches!(&e, crate::error::DiscordError::GatewayReconnectRequested) {
                                                    break 'inner GatewayLoopOutcome::Reconnect;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to decompress binary gateway frame ({} bytes): {}", bytes.len(), e);
                                        }
                                    }
                                }
                                Some(Ok(WsMessage::Close(frame))) => {
                                    let close_code: Option<u16> = frame.as_ref().map(|f| u16::from(f.code));
                                    match &frame {
                                        Some(f) => info!("Gateway connection closed: code={} reason={}", f.code, f.reason),
                                        None => info!("Gateway connection closed"),
                                    }
                                    break 'inner classify_close(close_code);
                                }
                                Some(Err(e)) => {
                                    error!(error = %e, "WebSocket error");
                                    break 'inner GatewayLoopOutcome::Reconnect;
                                }
                                None => {
                                    info!("WebSocket stream ended");
                                    break 'inner GatewayLoopOutcome::Reconnect;
                                }
                                _ => {}
                            }
                        }
                    }
                };

                // ── 3. Tear down per-connection state ──
                if let Some(handle) = heartbeat_handle.take() {
                    handle.abort();
                }
                *stage.write().await = ConnectionStage::Disconnected;

                // ── 4. Decide what to do next ──
                match outcome {
                    GatewayLoopOutcome::Shutdown => break 'supervisor,
                    GatewayLoopOutcome::Fatal { code, reason } => {
                        error!(close_code = code, reason, "Gateway closed with fatal code; not reconnecting");
                        break 'supervisor;
                    }
                    GatewayLoopOutcome::ReconnectFresh => {
                        *session_id.write().await = None;
                        *sequence.write().await = None;
                        info!("Clearing session state; will re-identify on reconnect");
                    }
                    GatewayLoopOutcome::Reconnect => {
                        info!("Gateway disconnected; will attempt to resume on reconnect");
                    }
                }

                // Brief pause before reconnecting to avoid a tight loop on rapid
                // disconnects; full backoff applies only when the WS connect itself fails.
                tokio::select! {
                    _ = shutdown_rx.recv() => break 'supervisor,
                    _ = tokio::time::sleep(RECONNECT_PAUSE) => {}
                }
                // current_reader stays None — supervisor loop will reconnect.
            }

            info!("Gateway supervisor task exiting");
        });

        Ok(())
    }

    /// Handle a single gateway message
    #[allow(clippy::too_many_arguments)]
    async fn handle_message(text: &str, token: &str, custom_status: UserStatus, capabilities: u32, writer: &Arc<RwLock<Option<WsWriter>>>, event_sender: &broadcast::Sender<DispatchEvent>, heartbeat_interval: &Arc<RwLock<u64>>, sequence: &Arc<RwLock<Option<u64>>>, session_id: &Arc<RwLock<Option<String>>>, heartbeat_handle: &mut Option<tokio::task::JoinHandle<()>>, awaiting_ack: &Arc<AtomicBool>, last_sent: &Arc<RwLock<Option<Instant>>>, latency: &Arc<RwLock<Option<Duration>>>, stage: &Arc<RwLock<ConnectionStage>>, pending_guilds: &Arc<AtomicUsize>) -> Result<()> {
        let payload: GatewayPayload = serde_json::from_str(text)?;
        let opcode = Opcode::from(payload.op);

        // Update sequence number
        if let Some(s) = payload.s {
            *sequence.write().await = Some(s);
        }

        match opcode {
            Opcode::Hello => {
                debug!("Received Hello");
                *stage.write().await = ConnectionStage::Handshake;
                if let Some(d) = &payload.d {
                    if let Some(hb_interval) = d["heartbeat_interval"].as_u64() {
                        *heartbeat_interval.write().await = hb_interval;

                        // Start heartbeat task
                        let writer_clone = Arc::clone(writer);
                        let seq_clone = Arc::clone(sequence);
                        let awaiting_ack_clone = Arc::clone(awaiting_ack);
                        let last_sent_clone = Arc::clone(last_sent);
                        let interval_ms = hb_interval;

                        // Reset heartbeat state for new connection
                        awaiting_ack.store(false, Ordering::SeqCst);
                        *last_sent.write().await = None;

                        if let Some(handle) = heartbeat_handle.take() {
                            handle.abort();
                        }

                        *heartbeat_handle = Some(tokio::spawn(async move {
                            let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
                            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                            // Skip the first immediate tick
                            ticker.tick().await;

                            loop {
                                ticker.tick().await;

                                // Check if we're still waiting for an ACK from the previous heartbeat
                                if awaiting_ack_clone.load(Ordering::SeqCst) {
                                    if let Some(sent_time) = *last_sent_clone.read().await {
                                        let elapsed = sent_time.elapsed();

                                        // If we've been waiting for more than 2 heartbeat intervals
                                        // since we sent the heartbeat, connection is likely dead
                                        if elapsed > Duration::from_millis(interval_ms * 2) {
                                            error!(elapsed = ?elapsed, "No heartbeat ACK received since last heartbeat sent (>2 intervals). Connection may be zombie.");
                                            // Break out to trigger reconnection
                                            break;
                                        }

                                        warn!("Still awaiting heartbeat ACK after {:?} since last send. Sending heartbeat anyway.", elapsed);
                                    }
                                }

                                let seq = *seq_clone.read().await;
                                let heartbeat = json!({ "op": 1, "d": seq });
                                if let Some(ref mut w) = *writer_clone.write().await {
                                    if let Err(e) = w.send(WsMessage::Text(heartbeat.to_string())).await {
                                        error!(error = %e, "Failed to send heartbeat");
                                        break;
                                    }
                                    // Record when we sent this heartbeat and mark awaiting ACK
                                    *last_sent_clone.write().await = Some(Instant::now());
                                    awaiting_ack_clone.store(true, Ordering::SeqCst);
                                    debug!("Sent heartbeat, awaiting ACK");
                                }
                            }
                        }));

                        // Wait briefly then send identify or resume
                        tokio::time::sleep(Duration::from_millis(500)).await;

                        let has_session = {
                            let sid = session_id.read().await;
                            let seq = sequence.read().await;
                            sid.is_some() && seq.is_some()
                        };

                        if has_session {
                            *stage.write().await = ConnectionStage::Resuming;
                            Self::send_resume(writer, token, session_id, sequence).await?;
                        } else {
                            *stage.write().await = ConnectionStage::Identifying;
                            Self::send_identify(writer, token, custom_status, capabilities).await?;
                        }
                    }
                }
            }
            Opcode::HeartbeatAck => {
                // Compute round-trip latency from the most recent heartbeat send time.
                if let Some(sent) = *last_sent.read().await {
                    *latency.write().await = Some(sent.elapsed());
                }
                // Clear the awaiting flag — the heartbeat was acknowledged
                awaiting_ack.store(false, Ordering::SeqCst);
                debug!("Received heartbeat ACK");
            }
            Opcode::Dispatch => {
                if let Some(event_type) = &payload.t {
                    debug!("Dispatch event: {}", event_type);

                    // Capture session_id from READY event; prime pending-guild counter.
                    if event_type == "READY" {
                        *stage.write().await = ConnectionStage::Connected;
                        if let Some(d) = &payload.d {
                            if let Some(sid) = d["session_id"].as_str() {
                                *session_id.write().await = Some(sid.to_string());
                                info!("Captured session_id: {}", sid);
                            }
                            // Count unavailable guilds so we know when CACHE_READY fires.
                            let guild_count = d["guilds"].as_array().map(|g| g.len()).unwrap_or(0);
                            pending_guilds.store(guild_count, Ordering::SeqCst);
                            debug!("READY: waiting for {} GUILD_CREATE(s) before CACHE_READY", guild_count);
                            if guild_count == 0 {
                                // No guilds — fire immediately after READY is dispatched.
                                let cache_ready = DispatchEvent { event_type: "CACHE_READY".to_string(), data: Value::Null };
                                if let Err(e) = event_sender.send(cache_ready) {
                                    warn!("Failed to dispatch CACHE_READY: {}", e);
                                }
                            }
                        }
                    }

                    // Each GUILD_CREATE decrements the pending counter.
                    // When it reaches zero, all guilds are populated — fire CACHE_READY.
                    // Note: fetch_update only decrements if n > 0, so stray GUILD_CREATE
                    // events arriving before READY (n == 0) are silently ignored and will
                    // not trigger CACHE_READY. This is intentional — READY sets the counter.
                    if event_type == "GUILD_CREATE" {
                        let prev = pending_guilds.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| if n > 0 { Some(n - 1) } else { None });
                        if let Ok(1) = prev {
                            // We just decremented from 1 → 0: all guilds are ready.
                            info!("All guilds received — firing CACHE_READY");
                            let cache_ready = DispatchEvent { event_type: "CACHE_READY".to_string(), data: Value::Null };
                            if let Err(e) = event_sender.send(cache_ready) {
                                warn!("Failed to dispatch CACHE_READY: {}", e);
                            }
                        }
                    }

                    // Mark stage as connected after successful RESUME acknowledgment
                    if event_type == "RESUMED" {
                        *stage.write().await = ConnectionStage::Connected;
                        info!("Session successfully resumed");
                    }

                    let event = DispatchEvent { event_type: event_type.clone(), data: payload.d.unwrap_or(Value::Null) };
                    if let Err(e) = event_sender.send(event) {
                        warn!("Failed to dispatch event '{}': no active receivers ({})", event_type, e);
                    }
                }
            }
            Opcode::Reconnect => {
                warn!("Received reconnect request from Discord gateway");
                // Return an error to break the message loop and trigger reconnection
                // The caller should use connect_with_auto_reconnect to handle this gracefully
                return Err(crate::error::DiscordError::GatewayReconnectRequested);
            }
            Opcode::InvalidSession => {
                warn!("Received invalid session");

                // Clear session state so we perform a full identify next time
                *session_id.write().await = None;
                *sequence.write().await = None;

                // Re-identify after delay
                tokio::time::sleep(Duration::from_secs(5)).await;
                Self::send_identify(writer, token, custom_status, capabilities).await?;
            }
            _ => {
                debug!("Received opcode: {:?}", opcode);
            }
        }

        Ok(())
    }

    /// Send the resume payload to reconnect
    async fn send_resume(writer: &Arc<RwLock<Option<WsWriter>>>, token: &str, session_id: &Arc<RwLock<Option<String>>>, sequence: &Arc<RwLock<Option<u64>>>) -> Result<()> {
        let sid = session_id.read().await;
        let seq = sequence.read().await;

        if let (Some(sid), Some(seq)) = (sid.as_ref(), *seq) {
            let resume = json!({
                "op": 6,
                "d": {
                    "token": token,
                    "session_id": sid,
                    "seq": seq
                }
            });

            if let Some(ref mut w) = *writer.write().await {
                w.send(WsMessage::Text(resume.to_string())).await?;
                info!("Sent resume payload (session: {}, seq: {})", sid, seq);
            }
        }

        Ok(())
    }

    /// Send the identify payload to authenticate
    async fn send_identify(writer: &Arc<RwLock<Option<WsWriter>>>, token: &str, status: UserStatus, capabilities: u32) -> Result<()> {
        let identify = json!({
            "op": 2,
            "d": {
                "token": token,
                "capabilities": capabilities,
                "properties": {
                    "os": "Windows",
                    "browser": "Chrome",
                    "device": "",
                    "system_locale": "en-US",
                    "browser_user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                    "browser_version": "120.0.0.0",
                    "os_version": "10",
                    "referrer": "",
                    "referring_domain": "",
                    "referrer_current": "",
                    "referring_domain_current": "",
                    "release_channel": "stable",
                    "client_build_number": client_build_number().await,
                    "client_event_source": null
                },
                "presence": {
                    "status": status.as_str(),
                    "since": 0,
                    "activities": [],
                    "afk": false
                },
                "compress": false,
                "client_state": {
                    "guild_versions": {},
                    "highest_last_message_id": "0",
                    "read_state_version": 0,
                    "user_guild_settings_version": -1,
                    "private_channels_version": "0",
                    "api_code_version": 0
                }
            }
        });

        if let Some(ref mut w) = *writer.write().await {
            w.send(WsMessage::Text(identify.to_string())).await?;
            info!("Sent identify payload");
        }

        Ok(())
    }

    /// Send a Voice State Update (opcode 4) to join, move, or leave a voice
    /// channel.
    ///
    /// Pass `channel_id = None` to disconnect from any voice channel in the
    /// guild. Pass `self_mute = true` / `self_deaf = true` to mute/deafen
    /// yourself locally.
    ///
    /// Mirrors serenity's `VoiceGatewayManager::join()`; Discord responds with
    /// a `VOICE_STATE_UPDATE` dispatch event containing the assigned session.
    pub async fn send_voice_state_update(&self, guild_id: u64, channel_id: Option<u64>, self_mute: bool, self_deaf: bool) -> Result<()> {
        let payload = json!({
            "op": Opcode::VoiceStateUpdate as u8,
            "d": {
                "guild_id": guild_id.to_string(),
                "channel_id": channel_id.map(|id| id.to_string()),
                "self_mute": self_mute,
                "self_deaf": self_deaf,
            }
        });
        if let Some(ref mut w) = *self.writer.write().await {
            w.send(WsMessage::Text(payload.to_string())).await?;
        }
        Ok(())
    }

    /// Send a presence update (status change)
    pub async fn send_presence(&self, status: UserStatus) -> Result<()> {
        let presence = json!({
            "op": 3,
            "d": {
                "status": status.as_str(),
                "since": 0,
                "activities": [],
                "afk": false
            }
        });

        if let Some(ref mut w) = *self.writer.write().await {
            w.send(WsMessage::Text(presence.to_string())).await?;
        }

        Ok(())
    }

    /// Send raw JSON payload
    pub async fn send_raw(&self, payload: Value) -> Result<()> {
        if let Some(ref mut w) = *self.writer.write().await {
            w.send(WsMessage::Text(payload.to_string())).await?;
        }
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.writer.read().await.is_some()
    }

    /// Disconnect from gateway
    pub async fn disconnect(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(ref mut w) = *self.writer.write().await {
            let _ = w.close().await;
        }
        *self.writer.write().await = None;
        *self.stage.write().await = ConnectionStage::Disconnected;
    }

    /// Get current connection stage
    pub async fn stage(&self) -> ConnectionStage {
        *self.stage.read().await
    }

    /// Return the round-trip gateway latency measured from the most recent
    /// heartbeat/ACK pair.  Returns `None` until the first ACK is received.
    ///
    /// Mirrors serenity's `Shard::latency()`.
    pub async fn latency(&self) -> Option<Duration> {
        *self.latency.read().await
    }

    /// Return whether the next reconnection should attempt a RESUME or a full
    /// REIDENTIFY.  Mirrors serenity's `reconnection_type()`.
    ///
    /// Returns `Resume` when a `session_id` is cached from the last READY
    /// event, `Reidentify` otherwise.
    pub async fn reconnection_type(&self) -> ReconnectType {
        if self.session_id.read().await.is_some() {
            ReconnectType::Resume
        } else {
            ReconnectType::Reidentify
        }
    }

    /// Decompress a zlib-compressed binary WebSocket frame into a UTF-8 string.
    ///
    /// Discord sends zlib-compressed gateway payloads as raw deflate or zlib
    /// streams depending on whether the `?compress=zlib-stream` query parameter
    /// was negotiated.  We handle both deflate and zlib wrappers here.
    fn decompress_zlib(bytes: &[u8]) -> std::result::Result<String, String> {
        use std::io::Read;

        // Try zlib (deflate with zlib header — the most common Discord format)
        let mut decoder = flate2::read::ZlibDecoder::new(bytes);
        let mut out = String::new();
        if decoder.read_to_string(&mut out).is_ok() {
            return Ok(out);
        }

        // Fall back to raw deflate (no zlib header)
        let mut decoder = flate2::read::DeflateDecoder::new(bytes);
        let mut out = String::new();
        decoder.read_to_string(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}
