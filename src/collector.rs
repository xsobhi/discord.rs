use discord_rs_core::Context;
use discord_rs_model::{Event, Message, Interaction};
use std::sync::Arc;
use tokio::sync::broadcast;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;
use tokio::time::timeout;

// Collector Event wrapper
#[derive(Debug, Clone)]
pub enum CollectorEvent {
    Message(Box<Message>),
    Interaction(Interaction),
    // Add more as needed
}

pub struct Collector {
    rx: broadcast::Receiver<Event>,
    filter: Box<dyn Fn(&Event) -> bool + Send + Sync>,
    // we could add end conditions
}

impl Collector {
    pub fn new(ctx: &Context, filter: impl Fn(&Event) -> bool + Send + Sync + 'static) -> Self {
        let broadcaster = ctx.broadcaster.clone()
            .downcast::<broadcast::Sender<Event>>()
            .expect("Broadcaster not found in Context");
        
        Self {
            rx: broadcaster.subscribe(),
            filter: Box::new(filter),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        loop {
            match self.rx.recv().await {
                Ok(event) => {
                    if (self.filter)(&event) {
                        return Some(event);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }

    pub async fn next_timeout(&mut self, duration: Duration) -> Option<Event> {
        match timeout(duration, self.next()).await {
            Ok(event) => event,
            Err(_) => None,
        }
    }
}

// Fluent builder for Collector
pub struct CollectorBuilder {
    ctx: Context,
    filter: Option<Box<dyn Fn(&Event) -> bool + Send + Sync>>,
}

impl CollectorBuilder {
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx,
            filter: None,
        }
    }

    pub fn filter<F>(mut self, filter: F) -> Self 
    where F: Fn(&Event) -> bool + Send + Sync + 'static 
    {
        self.filter = Some(Box::new(filter));
        self
    }

    pub fn build(self) -> Collector {
        let filter = self.filter.unwrap_or_else(|| Box::new(|_| true));
        Collector::new(&self.ctx, filter)
    }
}
