use std::sync::Arc;
use crate::config::Config;
use crate::traits::Http;

#[derive(Clone)]
pub struct Context {
    pub config: Arc<Config>,
    pub http: Arc<dyn Http>,
    pub cache: Arc<dyn std::any::Any + Send + Sync>,
}

impl Context {
    pub fn new(config: Arc<Config>, http: Arc<dyn Http>, cache: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        Self {
            config,
            http,
            cache,
        }
    }
}
