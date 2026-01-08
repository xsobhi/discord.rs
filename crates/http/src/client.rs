use discord_rs_core::{Config, DiscordError, Result};
use discord_rs_core::traits::Http;
use reqwest::{Client as ReqwestClient, header};
use serde::Deserialize;
use std::sync::Arc;
use async_trait::async_trait;

const API_VERSION: u8 = 10;
const BASE_URL: &str = "https://discord.com/api/v10";

#[derive(Debug, Clone)]
pub struct RestClient {
    http: ReqwestClient,
    config: Arc<Config>,
}

#[async_trait]
impl Http for RestClient {
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}", BASE_URL, path);
        let res = self.http.get(&url).send().await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if !res.status().is_success() {
             return Err(DiscordError::Http(format!("Status: {}", res.status())));
        }

        res.json::<serde_json::Value>().await
            .map_err(|e| DiscordError::Serialization(e.to_string()))
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let url = format!("{}{}", BASE_URL, path);
        let res = self.http.post(&url).json(&body).send().await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if !res.status().is_success() {
             return Err(DiscordError::Http(format!("Status: {}", res.status())));
        }

        res.json::<serde_json::Value>().await
            .map_err(|e| DiscordError::Serialization(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct GetGatewayBot {
    pub url: String,
    pub shards: u32,
    pub session_start_limit: SessionStartLimit,
}

#[derive(Debug, Deserialize)]
pub struct SessionStartLimit {
    pub total: u32,
    pub remaining: u32,
    pub reset_after: u32,
    pub max_concurrency: u32,
}

impl RestClient {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        let auth_value = format!("Bot {}", config.token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value)
                .map_err(|e| DiscordError::Validation(e.to_string()))?,
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("DiscordBot (discord.rs, 0.1.0)"),
        );

        let http = ReqwestClient::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        Ok(Self { http, config })
    }

    pub async fn get_gateway_bot(&self) -> Result<GetGatewayBot> {
        let url = format!("{}/gateway/bot", BASE_URL);
        let res = self.http.get(&url).send().await
            .map_err(|e| DiscordError::Http(e.to_string()))?;

        if !res.status().is_success() {
             return Err(DiscordError::Http(format!("Status: {}", res.status())));
        }

        res.json::<GetGatewayBot>().await
            .map_err(|e| DiscordError::Serialization(e.to_string()))
    }
}
