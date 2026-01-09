use discord_rs_core::{Config, DiscordError, Result, Intents};
use discord_rs_model::gateway::{GatewayPayload, OpCode, Identify, IdentifyProperties, Hello, Ready};
use discord_rs_model::Event;
use discord_rs_model::presence::PresenceUpdate;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use url::Url;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::UnboundedSender;
use rand::Rng;

#[cfg(feature = "gateway_zlib")]
use flate2::Decompress;

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

    pub async fn start(&mut self, initial_url: String) -> Result<()> {
        let mut attempt = 0;

        loop {
            // Use resume_url if we have one and we have a session
            let target_url_str = if self.session_id.is_some() && self.resume_url.is_some() {
                self.resume_url.as_ref().unwrap().clone()
            } else {
                initial_url.clone()
            };

            // Parse and normalize URL
            let mut url = Url::parse(&target_url_str)
                .map_err(|e| DiscordError::Gateway(format!("Invalid Gateway URL: {}", e)))?;

            // Ensure params
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
                    attempt = 0; // Reset backoff
                    
                    let should_resume = self.session_id.is_some() && self.last_sequence.lock().await.is_some();
                    
                    match self.handle_connection(ws_stream, should_resume).await {
                        Ok(_) => {
                            warn!("Connection closed cleanly. Reconnecting...");
                        }
                        Err(e) => {
                            error!("Connection error: {}. Reconnecting...", e);
                            if let DiscordError::Gateway(msg) = &e {
                                if msg.contains("Authentication failed") {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect: {}. Retrying...", e);
                }
            }

            // Exponential Backoff with Jitter
            attempt += 1;
            let cap = 120; // max 120 seconds
            let base = 2u64.pow(std::cmp::min(attempt, 6) as u32); // 2, 4, 8, 16, 32, 64
            let jitter = rand::thread_rng().gen_range(0..1000);
            let sleep_ms = std::cmp::min(base * 1000 + jitter, cap * 1000);
            
            warn!("Waiting {}ms before reconnect...", sleep_ms);
            tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        }
    }

    async fn handle_connection(&mut self, mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>, resume: bool) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        // Update last_ack to now so we don't immediately timeout
        *self.last_ack.lock().await = Instant::now();
        
        // Zlib State
        #[cfg(feature = "gateway_zlib")]
        let mut decompressor = Decompress::new(true); // zlib header? true usually means zlib w/ header
        #[cfg(feature = "gateway_zlib")]
        let mut buffer = Vec::new();

        // Writer task
        let write_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        // Heartbeat state
        let heartbeat_interval = Arc::new(tokio::sync::Mutex::new(None::<Duration>));
        let heartbeat_shutdown = Arc::new(AtomicBool::new(false));
        
        while let Some(msg) = read.next().await {
            let json_text = match msg {
                Ok(Message::Text(text)) => text,
                #[cfg(feature = "gateway_zlib")]
                Ok(Message::Binary(data)) => {
                    // Decompress
                    // zlib-stream means we keep context.
                    // We need to extend buffer?
                    // Decompress::decompress_vec appends to buffer.
                    // Discord sends Z_SYNC_FLUSH (0x00 0x00 0xff 0xff) at the end.
                    // We should check for it? flate2 handles it?
                    // flate2::Decompress::decompress_vec handles streaming decompression.
                    
                    match decompressor.decompress_vec(&data, &mut buffer, flate2::FlushDecompress::Sync) {
                        Ok(_) => {
                            // Check if we have a full json?
                            // Usually with FlushDecompress::Sync it flushes.
                            // Convert buffer to string
                            // But wait, buffer accumulates? 
                            // We need to clear buffer after processing?
                            // No, `decompress_vec` appends.
                            // If we have a complete message, we parse it and clear buffer.
                            // How do we know it's complete?
                            // Discord says "Every message... ends with a Z_SYNC_FLUSH suffix".
                            // So `Sync` flush should produce the output.
                            
                            let s = match String::from_utf8(buffer.clone()) {
                                Ok(s) => s,
                                Err(e) => {
                                    error!("Invalid UTF-8 in decompressed data: {}", e);
                                    buffer.clear();
                                    continue;
                                }
                            };
                            buffer.clear(); // Clear for next frame
                            s
                        }
                        Err(e) => {
                            error!("Decompression error: {}", e);
                            continue;
                        }
                    }
                }
                Ok(Message::Close(frame)) => {
                    info!("Gateway Closed: {:?}", frame);
                    break;
                }
                Err(e) => {
                    error!("WebSocket Error: {}", e);
                    break;
                }
                _ => continue,
            };

            match self.process_payload(&json_text, &tx, heartbeat_interval.clone(), heartbeat_shutdown.clone(), resume).await {
                Ok(should_break) => if should_break { break; },
                Err(e) => error!("Payload Error: {}", e),
            }
        }

        heartbeat_shutdown.store(true, Ordering::Relaxed);
        let _ = write_handle.await;

        Ok(())
    }

    async fn process_payload(
        &mut self, 
        text: &str, 
        tx: &UnboundedSender<Message>,
        heartbeat_interval: Arc<tokio::sync::Mutex<Option<Duration>>>,
        heartbeat_shutdown: Arc<AtomicBool>,
        resume: bool
    ) -> Result<bool> {
        // Parse payload
        let payload: GatewayPayload<serde_json::Value> = match serde_json::from_str(&text) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to parse payload: {} | Text: {}", e, text);
                return Ok(false);
            }
        };

        // Update sequence
        if let Some(s) = payload.s {
            *self.last_sequence.lock().await = Some(s);
        }

        match payload.op {
            OpCode::Hello => {
                let hello: Hello = serde_json::from_value(payload.d.unwrap())
                    .map_err(|e| DiscordError::Serialization(e.to_string()))?;
                
                info!("Received Hello. Heartbeat interval: {}ms", hello.heartbeat_interval);

                // Start Heartbeat task
                let interval = Duration::from_millis(hello.heartbeat_interval);
                *heartbeat_interval.lock().await = Some(interval);
                
                let tx_clone = tx.clone();
                let shutdown_clone = heartbeat_shutdown.clone();
                let seq_clone = self.last_sequence.clone();
                let last_ack_clone = self.last_ack.clone();
                
                // Spawn heartbeat task
                tokio::spawn(async move {
                    let mut interval_timer = tokio::time::interval(interval);
                    loop {
                        interval_timer.tick().await;
                        if shutdown_clone.load(Ordering::Relaxed) {
                            break;
                        }
                        
                        // Zombie Detection
                        let last_ack_time = *last_ack_clone.lock().await;
                        if last_ack_time.elapsed() > interval * 2 {
                            error!("Zombie connection detected (no ACK in 2 intervals). Closing...");
                            break;
                        }

                        debug!("Sending Heartbeat");
                        let last_seq = *seq_clone.lock().await;
                        let heartbeat_msg = serde_json::json!({
                            "op": 1,
                            "d": last_seq
                        });
                        if let Err(_) = tx_clone.send(Message::Text(heartbeat_msg.to_string())) {
                            break;
                        }
                    }
                });

                if resume {
                    self.resume(&tx).await?;
                } else {
                    self.identify(&tx).await?;
                }
            }
            OpCode::Dispatch => {
                // Deserialize into Event enum
                match serde_json::from_str::<Event>(&text) {
                    Ok(event) => {
                        // Handle internal state updates
                        match &event {
                            Event::Ready(ready) => {
                                self.session_id = Some(ready.session_id.clone());
                                self.resume_url = Some(ready.resume_gateway_url.clone());
                                info!("READY! Logged in as {}#{}", ready.user.username, ready.user.discriminator);
                            }
                            _ => {}
                        }

                        // Dispatch to client
                        if let Err(e) = self.event_tx.send(event) {
                            error!("Failed to dispatch event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize event: {} | Text: {}", e, text);
                    }
                }
            }
            OpCode::HeartbeatAck => {
                debug!("Heartbeat Acknowledged");
                *self.last_ack.lock().await = Instant::now();
            }
            OpCode::Reconnect => {
                info!("Received OpCode 7 Reconnect. Reconnecting...");
                return Ok(true);
            }
            OpCode::InvalidSession => {
                let resumable = payload.d.and_then(|v| v.as_bool()).unwrap_or(false);
                if resumable {
                    info!("Received Invalid Session (Resumable). Attempting Resume...");
                    self.resume(&tx).await?;
                } else {
                    info!("Received Invalid Session (Not Resumable). Re-identifying...");
                    self.session_id = None;
                    *self.last_sequence.lock().await = None;
                    // We need to wait a random 1-5s
                    let delay = rand::thread_rng().gen_range(1000..5000);
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    self.identify(&tx).await?;
                }
            }
            _ => {
                debug!("Received OpCode: {:?}", payload.op);
            }
        }
        Ok(false)
    }

    async fn identify(&self, tx: &UnboundedSender<Message>) -> Result<()> {
        info!("Identifying...");
        let identify = Identify {
            token: self.config.token.clone(),
            properties: IdentifyProperties {
                os: std::env::consts::OS.to_string(),
                browser: "discord.rs".to_string(),
                device: "discord.rs".to_string(),
            },
            compress: None,
            large_threshold: None,
            shard: self.shard,
            presence: self.presence.clone(), // Use stored presence
            intents: self.intents,
        };

        let payload = GatewayPayload {
            op: OpCode::Identify,
            d: Some(identify),
            s: None,
            t: None,
        };

        let json = serde_json::to_string(&payload)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;

        tx.send(Message::Text(json))
            .map_err(|_| DiscordError::Gateway("Failed to send Identify".to_string()))?;
        
        Ok(())
    }

    async fn resume(&self, tx: &UnboundedSender<Message>) -> Result<()> {
        info!("Resuming...");
        let resume_payload = serde_json::json!({
            "token": self.config.token,
            "session_id": self.session_id.as_ref().unwrap(),
            "seq": *self.last_sequence.lock().await
        });

        let payload = serde_json::json!({
            "op": 6,
            "d": resume_payload
        });

        let json = payload.to_string();

        tx.send(Message::Text(json))
            .map_err(|_| DiscordError::Gateway("Failed to send Resume".to_string()))?;
        
        Ok(())
    }

    pub async fn update_presence(&self, presence: PresenceUpdate, tx: &UnboundedSender<Message>) -> Result<()> {
        let payload = GatewayPayload {
            op: OpCode::PresenceUpdate,
            d: Some(presence),
            s: None,
            t: None,
        };

        let json = serde_json::to_string(&payload)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;

        tx.send(Message::Text(json))
            .map_err(|_| DiscordError::Gateway("Failed to send Presence Update".to_string()))?;
        Ok(())
    }
}
