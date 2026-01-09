use discord_rs_core::{Config, Intents, Result, Context};
use discord_rs_gateway::GatewayManager;
use discord_rs_http::RestClient;
use discord_rs_model::{Event, Message, Interaction, gateway::Ready};
use discord_rs_cache::{Cache, update_cache_from_event};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};
use std::future::Future;
use std::pin::Pin;

// Type alias for async event handlers
type Handler<T> = Box<dyn Fn(Context, T) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

pub struct Client {
    config: Arc<Config>,
    intents: Intents,
    cache: Arc<Cache>,
    // Handlers
    ready_handlers: Vec<Handler<Ready>>,
    message_create_handlers: Vec<Handler<Box<Message>>>,
    interaction_create_handlers: Vec<Handler<Interaction>>,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            config: Arc::new(Config::new(token)),
            intents: Intents::empty(),
            cache: Arc::new(Cache::new()),
            ready_handlers: Vec::new(),
            message_create_handlers: Vec::new(),
            interaction_create_handlers: Vec::new(),
        }
    }

    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
    }

    pub fn cache(&self) -> Arc<Cache> {
        self.cache.clone()
    }

    // --- Event Registration ---

    pub fn on_ready<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Context, Ready) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.ready_handlers.push(Box::new(move |ctx, ready| Box::pin(handler(ctx, ready))));
        self
    }

    pub fn on_message_create<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Context, Box<Message>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.message_create_handlers.push(Box::new(move |ctx, msg| Box::pin(handler(ctx, msg))));
        self
    }

    pub fn on_interaction_create<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Context, Interaction) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.interaction_create_handlers.push(Box::new(move |ctx, interaction| Box::pin(handler(ctx, interaction))));
        self
    }

    // --- Runtime ---

    pub async fn login(self) -> Result<()> {
        info!("Logging in...");
        
        let config = self.config.clone();
        let cache = self.cache.clone();
        
        // HTTP Client
        let rest = Arc::new(RestClient::new(config.clone())?);
        
        // Context
        let ctx = Context::new(config.clone(), rest.clone(), cache.clone());
        
        // Gateway Channel
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        
        // Start Gateway
        let gateway_info = rest.get_gateway_bot().await?;
        info!("Gateway URL: {}", gateway_info.url);
        
        let mut gateway = GatewayManager::new(config.clone(), self.intents, event_tx);
        
        // Spawn Gateway Task
        tokio::spawn(async move {
            if let Err(e) = gateway.start(gateway_info.url).await {
                error!("Gateway fatal error: {}", e);
            }
        });

        // Dispatch Loop
        let ready_handlers = Arc::new(self.ready_handlers);
        let message_create_handlers = Arc::new(self.message_create_handlers);
        let interaction_create_handlers = Arc::new(self.interaction_create_handlers);

        while let Some(event) = event_rx.recv().await {
            // PHASE 5: Cache-before-dispatch
            update_cache_from_event(&cache, &event);

            let ctx_clone = ctx.clone();
            
            match event {
                Event::Ready(ready) => {
                    let handlers = ready_handlers.clone();
                    tokio::spawn(async move {
                        for handler in handlers.iter() {
                            if let Err(e) = handler(ctx_clone.clone(), ready.clone()).await {
                                error!("Error in Ready handler: {}", e);
                            }
                        }
                    });
                }
                Event::MessageCreate(msg) => {
                    let handlers = message_create_handlers.clone();
                    tokio::spawn(async move {
                        for handler in handlers.iter() {
                            if let Err(e) = handler(ctx_clone.clone(), msg.clone()).await {
                                error!("Error in MessageCreate handler: {}", e);
                            }
                        }
                    });
                }
                Event::InteractionCreate(interaction) => {
                    let handlers = interaction_create_handlers.clone();
                    tokio::spawn(async move {
                        for handler in handlers.iter() {
                            if let Err(e) = handler(ctx_clone.clone(), interaction.clone()).await {
                                error!("Error in InteractionCreate handler: {}", e);
                            }
                        }
                    });
                }
                _ => {
                    // Unhandled event
                }
            }
        }
        
        Ok(())
    }
}
