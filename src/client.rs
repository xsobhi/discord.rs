use discord_rs_core::{Config, Intents, Result, Context};
use discord_rs_gateway::GatewayManager;
use discord_rs_http::RestClient;
use discord_rs_model::{Event, Message, gateway::Ready};
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
    // Handlers
    ready_handlers: Vec<Handler<Ready>>,
    message_create_handlers: Vec<Handler<Box<Message>>>,
}

impl Client {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            config: Arc::new(Config::new(token)),
            intents: Intents::empty(),
            ready_handlers: Vec::new(),
            message_create_handlers: Vec::new(),
        }
    }

    pub fn intents(mut self, intents: Intents) -> Self {
        self.intents = intents;
        self
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

    // --- Runtime ---

    pub async fn login(self) -> Result<()> {
        info!("Logging in...");
        
        let config = self.config.clone();
        
        // HTTP Client
        let rest = Arc::new(RestClient::new(config.clone())?);
        
        // Context
        let ctx = Context::new(config.clone(), rest.clone());
        
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
        // We move `self` into this loop to own the handlers
        let ready_handlers = Arc::new(self.ready_handlers);
        let message_create_handlers = Arc::new(self.message_create_handlers);

        while let Some(event) = event_rx.recv().await {
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
                _ => {
                    // Unhandled event
                }
            }
        }
        
        Ok(())
    }
}