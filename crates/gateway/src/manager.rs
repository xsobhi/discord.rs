use discord_rs_core::{Config, DiscordError, Intents, Result};
use discord_rs_model::gateway::{GatewayPayload, Hello, Identify, IdentifyProperties, OpCode};
use discord_rs_model::presence::PresenceUpdate;
use discord_rs_model::Event;

use futures::{SinkExt, StreamExt};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};
use url::Url;

#[cfg(feature = "gateway_zlib")]
use flate2::{Decompress, FlushDecompress};

pub struct GatewayManager {
    config: Arc<Config>,
    intents: Intents,
    session_id: Option<String>,
    last_sequence: Arc<tokio::sync::Mutex<Option<u64>>>,
    resume_url: Option<String>,
    event_tx: UnboundedSender<Event>,
    shard: Option<[u64; 2]>,
    last_ack: Arc<tokio::sync::Mutex<Instant>>,
    presence: Option<PresenceUpdate>,
}

impl GatewayManager {
    pub fn new(config: Arc<Config>, intents: Intents, event_tx: UnboundedSender<Event>) -> Self {
        Self {
            config,
            intents,
            session_id: None,
            last_sequence: Arc::new(tokio::sync::Mutex::new(None)),
            resume_url: None,
            event_tx,
            shard: None,
            last_ack: Arc::new(tokio::sync::Mutex::new(Instant::now())),
            presence: None,
        }
    }

    pub fn shard(mut self, shard_id: u64, shard_count: u64) -> Self {
        self.shard = Some([shard_id, shard_count]);
        self
    }

    pub fn presence(mut self, presence: PresenceUpdate) -> Self {
        self.presence = Some(presence);
        self
    }

    fn gateway_token(&self) -> String {
        let mut t = self.config.token.trim().to_string();
        loop {
            if let Some(rest) = t.strip_prefix("Bot ") {
                t = rest.trim().to_string();
            } else {
                break;
            }
        }
        t
    }

    fn log_close(&self, frame: Option<&CloseFrame<'_>>) {
        if let Some(cf) = frame {
            info!("Gateway closed: code={:?}, reason={}", cf.code, cf.reason);
        } else {
            info!("Gateway closed: no close frame");
        }
    }

    #[cfg(feature = "gateway_zlib")]
    fn find_zlib_suffix(buf: &[u8]) -> Option<usize> {
        const SUFFIX: [u8; 4] = [0x00, 0x00, 0xFF, 0xFF];
        buf.windows(SUFFIX.len()).position(|w| w == SUFFIX)
    }

    pub async fn start(&mut self, initial_url: String) -> Result<()> {
        let mut attempt: u32 = 0;

        loop {
            let target_url_str = if self.session_id.is_some() && self.resume_url.is_some() {
                self.resume_url.as_ref().unwrap().clone()
            } else {
                initial_url.clone()
            };

            let mut url = Url::parse(&target_url_str)
                .map_err(|e| DiscordError::Gateway(format!("Invalid Gateway URL: {}", e)))?;

            if !url.query_pairs().any(|(k, _)| k == "v") {
                url.query_pairs_mut().append_pair("v", "10");
            }
            if !url.query_pairs().any(|(k, _)| k == "encoding") {
                url.query_pairs_mut().append_pair("encoding", "json");
            }

            #[cfg(feature = "gateway_zlib")]
            {
                if !url.query_pairs().any(|(k, _)| k == "compress") {
                    url.query_pairs_mut().append_pair("compress", "zlib-stream");
                }
            }

            let final_url = url.to_string();
            info!("Connecting to Gateway: {}", final_url);

            match connect_async(&final_url).await {
                Ok((ws_stream, _)) => {
                    info!("Connected to Gateway");
                    attempt = 0;

                    let should_resume =
                        self.session_id.is_some() && self.last_sequence.lock().await.is_some();

                    match self.handle_connection(ws_stream, should_resume).await {
                        Ok(()) => warn!("Connection ended. Reconnecting..."),
                        Err(e) => error!("Connection error: {}. Reconnecting...", e),
                    }
                }
                Err(e) => {
                    error!("Failed to connect: {}. Retrying...", e);
                }
            }

            attempt = attempt.saturating_add(1);
            let cap_s: u64 = 120;
            let base = 2u64.pow(std::cmp::min(attempt, 6));
            let jitter_ms: u64 = rand::thread_rng().gen_range(0..1000);
            let sleep_ms = std::cmp::min(base * 1000 + jitter_ms, cap_s * 1000);

            warn!("Waiting {}ms before reconnect...", sleep_ms);
            tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    }

    async fn handle_connection(
        &mut self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        resume: bool,
    ) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

        *self.last_ack.lock().await = Instant::now();

        let write_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                debug!(
                    "WS send: {}",
                    match &msg {
                        Message::Text(_) => "Text",
                        Message::Binary(_) => "Binary",
                        Message::Close(_) => "Close",
                        Message::Ping(_) => "Ping",
                        Message::Pong(_) => "Pong",
                        _ => "Other",
                    }
                );

                if let Err(e) = write.send(msg).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        let heartbeat_interval = Arc::new(tokio::sync::Mutex::new(None::<Duration>));
        let heartbeat_shutdown = Arc::new(AtomicBool::new(false));

        // zlib-stream state
        #[cfg(feature = "gateway_zlib")]
        let mut decompressor = Decompress::new(true);
        #[cfg(feature = "gateway_zlib")]
        let mut zlib_in: Vec<u8> = Vec::with_capacity(64 * 1024);
        #[cfg(feature = "gateway_zlib")]
        let mut out_buf: Vec<u8> = Vec::with_capacity(256 * 1024);

        while let Some(msg) = read.next().await {
            // Helpful visibility into inbound frames
            match &msg {
                Ok(Message::Text(_)) => debug!("WS recv: Text"),
                Ok(Message::Binary(b)) => debug!("WS recv: Binary ({} bytes)", b.len()),
                Ok(Message::Close(_)) => debug!("WS recv: Close"),
                Ok(Message::Ping(_)) => debug!("WS recv: Ping"),
                Ok(Message::Pong(_)) => debug!("WS recv: Pong"),
                Ok(_) => debug!("WS recv: Other"),
                Err(e) => debug!("WS recv: Error {}", e),
            }

            let json_texts: Vec<String> = match msg {
                Ok(Message::Text(text)) => vec![text],

                #[cfg(feature = "gateway_zlib")]
                Ok(Message::Binary(data)) => {
                    zlib_in.extend_from_slice(&data);

                    let mut texts: Vec<String> = Vec::new();

                    // Process as many complete flushed payloads as we have buffered.
                    while let Some(pos) = Self::find_zlib_suffix(&zlib_in) {
                        let end = pos + 4; // include suffix
                        let chunk: Vec<u8> = zlib_in.drain(..end).collect();

                        out_buf.clear();

                        match decompressor.decompress_vec(&chunk, &mut out_buf, FlushDecompress::Sync)
                        {
                            Ok(_) => {
                                match std::str::from_utf8(&out_buf) {
                                    Ok(s) => {
                                        if !s.trim().is_empty() {
                                            texts.push(s.to_string());
                                        }
                                    }
                                    Err(e) => {
                                        error!("Invalid UTF-8 in decompressed payload: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Decompression error: {}", e);
                                // Stream is likely corrupted; clear to avoid unbounded growth.
                                zlib_in.clear();
                                break;
                            }
                        }
                    }

                    texts
                }

                Ok(Message::Close(frame)) => {
                    self.log_close(frame.as_ref());
                    break;
                }

                Ok(Message::Ping(_)) => vec![],
                Ok(Message::Pong(_)) => vec![],

                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }

                _ => vec![],
            };

            for json_text in json_texts {
                match self
                    .process_text_payloads(
                        &json_text,
                        &tx,
                        heartbeat_interval.clone(),
                        heartbeat_shutdown.clone(),
                        resume,
                    )
                    .await
                {
                    Ok(should_reconnect) => {
                        if should_reconnect {
                            heartbeat_shutdown.store(true, Ordering::Relaxed);
                            let _ = write_handle.await;
                            return Ok(());
                        }
                    }
                    Err(e) => error!("Payload processing error: {}", e),
                }
            }
        }

        heartbeat_shutdown.store(true, Ordering::Relaxed);
        let _ = write_handle.await;
        Ok(())
    }

    async fn process_text_payloads(
        &mut self,
        text: &str,
        tx: &UnboundedSender<Message>,
        heartbeat_interval: Arc<tokio::sync::Mutex<Option<Duration>>>,
        heartbeat_shutdown: Arc<AtomicBool>,
        resume: bool,
    ) -> Result<bool> {
        let mut iter = serde_json::Deserializer::from_str(text)
            .into_iter::<GatewayPayload<serde_json::Value>>();

        while let Some(item) = iter.next() {
            let payload = match item {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to parse gateway payload stream: {} | Text: {}", e, text);
                    return Ok(false);
                }
            };

            let should_reconnect = self
                .process_single_payload(
                    payload,
                    tx,
                    heartbeat_interval.clone(),
                    heartbeat_shutdown.clone(),
                    resume,
                )
                .await?;

            if should_reconnect {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn process_single_payload(
        &mut self,
        payload: GatewayPayload<serde_json::Value>,
        tx: &UnboundedSender<Message>,
        heartbeat_interval: Arc<tokio::sync::Mutex<Option<Duration>>>,
        heartbeat_shutdown: Arc<AtomicBool>,
        resume: bool,
    ) -> Result<bool> {
        if let Some(s) = payload.s {
            *self.last_sequence.lock().await = Some(s);
        }

        match payload.op {
            OpCode::Hello => {
                let hello: Hello = serde_json::from_value(payload.d.unwrap_or_default())
                    .map_err(|e| DiscordError::Serialization(e.to_string()))?;

                info!(
                    "Received Hello. Heartbeat interval: {}ms",
                    hello.heartbeat_interval
                );

                let interval = Duration::from_millis(hello.heartbeat_interval);
                *heartbeat_interval.lock().await = Some(interval);

                // Send an immediate heartbeat once after Hello (d = last seq or null)
                self.send_heartbeat(tx).await?;

                // Start periodic heartbeats
                let tx_clone = tx.clone();
                let shutdown_clone = heartbeat_shutdown.clone();
                let seq_clone = self.last_sequence.clone();
                let last_ack_clone = self.last_ack.clone();

                tokio::spawn(async move {
                    let mut ticker = tokio::time::interval(interval);

                    loop {
                        ticker.tick().await;

                        if shutdown_clone.load(Ordering::Relaxed) {
                            break;
                        }

                        let last_ack_time = *last_ack_clone.lock().await;
                        if last_ack_time.elapsed() > interval * 2 {
                            error!("Zombie connection detected (no ACK for 2 intervals).");
                            break;
                        }

                        let last_seq = *seq_clone.lock().await;
                        let heartbeat_msg = serde_json::json!({ "op": 1, "d": last_seq });

                        debug!("Sending heartbeat with d={:?}", last_seq);

                        if tx_clone.send(Message::Text(heartbeat_msg.to_string())).is_err() {
                            error!("Heartbeat send failed: write channel closed");
                            break;
                        }
                    }
                });

                // Now identify or resume
                if resume {
                    self.resume(tx).await?;
                } else {
                    self.identify(tx).await?;
                }
            }

            OpCode::Heartbeat => {
                debug!("Heartbeat requested by server (OP 1). Sending immediately.");
                self.send_heartbeat(tx).await?;
            }

            OpCode::Dispatch => {
                let t = payload.t.as_deref().unwrap_or("");
                let d = payload.d.unwrap_or(serde_json::Value::Null);

                let event: Event = match t {
                    "READY" => Event::Ready(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    ),
                    "RESUMED" => Event::Resumed(d),
                    "MESSAGE_CREATE" => Event::MessageCreate(Box::new(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    )),
                    "MESSAGE_UPDATE" => Event::MessageUpdate(Box::new(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    )),
                    "MESSAGE_DELETE" => Event::MessageDelete(d),
                    "GUILD_CREATE" => Event::GuildCreate(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    ),
                    "GUILD_UPDATE" => Event::GuildUpdate(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    ),
                    "GUILD_DELETE" => Event::GuildDelete(d),
                    "INTERACTION_CREATE" => Event::InteractionCreate(
                        serde_json::from_value(d)
                            .map_err(|e| DiscordError::Serialization(e.to_string()))?,
                    ),
                    _ => Event::Unknown,
                };

                if let Event::Ready(ready) = &event {
                    self.session_id = Some(ready.session_id.clone());
                    self.resume_url = Some(ready.resume_gateway_url.clone());
                    info!(
                        "READY! Logged in as {}#{}",
                        ready.user.username, ready.user.discriminator
                    );
                }

                if let Err(e) = self.event_tx.send(event) {
                    error!("Failed to dispatch event to client: {}", e);
                }
            }

            OpCode::HeartbeatAck => {
                info!("Heartbeat ACK");
                *self.last_ack.lock().await = Instant::now();
            }

            OpCode::Reconnect => {
                info!("Server requested reconnect (OP 7).");
                return Ok(true);
            }

            OpCode::InvalidSession => {
                let resumable = payload.d.and_then(|v| v.as_bool()).unwrap_or(false);
                info!("Invalid Session received. d(resumable?) = {}", resumable);

                if resumable && self.session_id.is_some() {
                    info!("Invalid Session (resumable). Attempting RESUME...");
                    self.resume(tx).await?;
                } else {
                    info!("Invalid Session (not resumable). Will reconnect + identify.");
                    self.session_id = None;
                    self.resume_url = None;
                    *self.last_sequence.lock().await = None;

                    let delay_ms = rand::thread_rng().gen_range(1500..6000);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                    return Ok(true);
                }
            }

            _ => debug!("Received OpCode: {:?}", payload.op),
        }

        Ok(false)
    }

    async fn send_heartbeat(&self, tx: &UnboundedSender<Message>) -> Result<()> {
        let last_seq = *self.last_sequence.lock().await;
        let heartbeat_msg = serde_json::json!({ "op": 1, "d": last_seq });

        debug!("Sending immediate heartbeat with d={:?}", last_seq);

        tx.send(Message::Text(heartbeat_msg.to_string()))
            .map_err(|_| DiscordError::Gateway("Failed to send Heartbeat".to_string()))?;

        Ok(())
    }

    async fn identify(&self, tx: &UnboundedSender<Message>) -> Result<()> {
        info!("Identifying...");

        let identify = Identify {
            token: self.gateway_token(),
            properties: IdentifyProperties {
                os: std::env::consts::OS.to_string(),
                browser: "discord_rs".to_string(),
                device: "discord_rs".to_string(),
            },
            compress: None,
            large_threshold: None,
            shard: self.shard,
            presence: self.presence.clone(),
            intents: self.intents.bits(), // numeric bitmask
        };

        let payload = GatewayPayload {
            op: OpCode::Identify,
            d: Some(identify),
            s: None,
            t: None,
        };

        let json =
            serde_json::to_string(&payload).map_err(|e| DiscordError::Serialization(e.to_string()))?;

        tx.send(Message::Text(json))
            .map_err(|_| DiscordError::Gateway("Failed to send Identify".to_string()))?;

        Ok(())
    }

    async fn resume(&self, tx: &UnboundedSender<Message>) -> Result<()> {
        info!("Resuming...");

        let session_id = self
            .session_id
            .as_ref()
            .ok_or_else(|| DiscordError::Gateway("Cannot resume without session_id".to_string()))?;

        let token = self.gateway_token();
        let seq = *self.last_sequence.lock().await;

        let payload = serde_json::json!({
            "op": 6,
            "d": { "token": token, "session_id": session_id, "seq": seq }
        });

        tx.send(Message::Text(payload.to_string()))
            .map_err(|_| DiscordError::Gateway("Failed to send Resume".to_string()))?;

        Ok(())
    }

    pub async fn update_presence(
        &self,
        presence: PresenceUpdate,
        tx: &UnboundedSender<Message>,
    ) -> Result<()> {
        let payload = GatewayPayload {
            op: OpCode::PresenceUpdate,
            d: Some(presence),
            s: None,
            t: None,
        };

        let json =
            serde_json::to_string(&payload).map_err(|e| DiscordError::Serialization(e.to_string()))?;

        tx.send(Message::Text(json))
            .map_err(|_| DiscordError::Gateway("Failed to send Presence Update".to_string()))?;

        Ok(())
    }
}
