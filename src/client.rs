use discord_rs_core::{Config, Intents, Result};
use discord_rs_gateway::GatewayManager;
use discord_rs_http::RestClient;
use std::sync::Arc;
use tracing::info;

pub struct Client {
    config: Arc<Config>,
    intents: Intents,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            config: Arc::new(Config::new(token)),
            intents: Intents::empty(),
        }
    }

    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
    }

    pub async fn login(&mut self) -> Result<()> {
        info!("Logging in...");
        let rest = RestClient::new(self.config.clone())?;
        
        // Get Gateway URL
        let gateway_info = rest.get_gateway_bot().await?;
        info!("Gateway URL: {}", gateway_info.url);
        
        let mut gateway = GatewayManager::new(self.config.clone(), self.intents);
        
        gateway.start(gateway_info.url).await?;
        
        Ok(())
    }
}
