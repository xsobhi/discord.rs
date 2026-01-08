use async_trait::async_trait;
use serde::de::DeserializeOwned;
use crate::Result;

#[async_trait]
pub trait Http: Send + Sync {
    // Basic interface for now, will expand as needed
    async fn get(&self, path: &str) -> Result<serde_json::Value>;
    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value>;
    // Add specific methods as needed by Context shortcuts
}
