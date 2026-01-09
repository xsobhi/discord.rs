use discord_rs_core::{Config, Intents, Result, DiscordError};
use discord_rs_gateway::GatewayManager;
use discord_rs_http::RestClient;
use discord_rs_model::{Event, presence::PresenceUpdate};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

pub struct ShardManager {
    config: Arc<Config>,
    intents: Intents,
    event_tx: mpsc::UnboundedSender<Event>,
    presence: Option<PresenceUpdate>,
}

impl ShardManager {
    pub fn new(config: Arc<Config>, intents: Intents, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            config,
            intents,
            event_tx,
            presence: None,
        }
    }

    pub fn presence(mut self, presence: PresenceUpdate) -> Self {
        self.presence = Some(presence);
        self
    }

    pub async fn start(&self) -> Result<()> {
        let rest = RestClient::new(self.config.clone())?;
        let gateway_info = rest.get_gateway_bot().await?;
        
        info!("Recommended shards: {}", gateway_info.shards);
        info!("Concurrency limit: {}", gateway_info.session_start_limit.max_concurrency);

        let shard_count = gateway_info.shards;
        let max_concurrency = gateway_info.session_start_limit.max_concurrency;

        // Start shards with coordination
        for shard_id in 0..shard_count {
            let config = self.config.clone();
            let intents = self.intents;
            let event_tx = self.event_tx.clone();
            let url = gateway_info.url.clone();
            let presence = self.presence.clone();

            // Simple coordination: Wait between shards if we hit concurrency limit
            // In a real distributed system, this would be more complex.
            if shard_id > 0 && shard_id % max_concurrency as u32 == 0 {
                info!("Reached max concurrency, waiting before next batch...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }

            tokio::spawn(async move {
                loop {
                    let mut manager = GatewayManager::new(config.clone(), intents, event_tx.clone())
                        .shard(shard_id as u64, shard_count as u64);
                    
                    if let Some(p) = presence.clone() {
                        manager = manager.presence(p);
                    }
                    
                    // manager.start now has an internal loop for reconnects.
                    // If it returns, it implies a fatal error or requested exit.
                    match manager.start(url.clone()).await {
                        Ok(_) => {
                            info!("Shard {} stopped gracefully.", shard_id);
                            break;
                        }
                        Err(e) => {
                            error!("Shard {} fatal error: {}. Restarting in 5s...", shard_id, e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
            
            // Discord requires at least 5 seconds between identifies
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }
}
