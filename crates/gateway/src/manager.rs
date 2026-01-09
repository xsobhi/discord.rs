use discord_rs_core::{Config, DiscordError, Result, Intents};
use discord_rs_model::gateway::{GatewayPayload, OpCode, Identify, IdentifyProperties, Hello, Ready};
use discord_rs_model::Event;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use url::Url;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::UnboundedSender;

pub struct GatewayManager {
    config: Arc<Config>,
    intents: Intents,
    session_id: Option<String>,
    last_sequence: Arc<tokio::sync::Mutex<Option<u64>>>,
    resume_url: Option<String>,
    event_tx: UnboundedSender<Event>,
    shard: Option<[u64; 2]>,
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
        }
    }

    pub fn shard(mut self, shard_id: u64, shard_count: u64) -> Self {
        self.shard = Some([shard_id, shard_count]);
        self
    }

    pub async fn start(&mut self, mut url: String) -> Result<()> {
        // Ensure params
        if !url.contains("v=10") {
             let separator = if url.contains('?') { "&" } else { "?" };
             url = format!("{}{}{}", url, separator, "v=10&encoding=json");
        }

        info!("Connecting to Gateway: {}", url);
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| DiscordError::Gateway(format!("Connection failed: {}", e)))?;

        info!("Connected to Gateway");

        self.handle_connection(ws_stream).await
    }

    async fn handle_connection(&mut self, mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

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
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse payload
                    let payload: GatewayPayload<serde_json::Value> = match serde_json::from_str(&text) {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Failed to parse payload: {} | Text: {}", e, text);
                            continue;
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
                            
                            // Spawn heartbeat task
                            tokio::spawn(async move {
                                let mut interval_timer = tokio::time::interval(interval);
                                loop {
                                    interval_timer.tick().await;
                                    if shutdown_clone.load(Ordering::Relaxed) {
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

                            // Identify
                            // TODO: Check if we should Resume instead
                            self.identify(&tx).await?;
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
                        }
                        _ => {
                            debug!("Received OpCode: {:?}", payload.op);
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
                _ => {}
            }
        }

        heartbeat_shutdown.store(true, Ordering::Relaxed);
        let _ = write_handle.await;

        Ok(())
    }

    async fn identify(&self, tx: &tokio::sync::mpsc::UnboundedSender<Message>) -> Result<()> {
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
            presence: None,
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
}
