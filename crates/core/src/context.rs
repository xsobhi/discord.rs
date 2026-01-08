use std::sync::Arc;
use crate::config::Config;
use crate::traits::Http;

#[derive(Clone)]
pub struct Context {
    pub config: Arc<Config>,
    pub http: Arc<dyn Http>,
    // cache will go here later
}

impl Context {
    pub fn new(config: Arc<Config>, http: Arc<dyn Http>) -> Self {
        Self {
            config,
            http,
        }
    }
}
