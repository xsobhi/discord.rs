use discord_rs_core::{Config, DiscordError, Result};
use discord_rs_core::traits::Http;
use reqwest::{Client as ReqwestClient, header, Method, StatusCode};
use serde::Deserialize;
use std::sync::Arc;
use async_trait::async_trait;
use crate::ratelimit::RateLimiter;
use crate::routing::Route;
use crate::error::DiscordApiError;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

const API_VERSION: u8 = 10;
const BASE_URL: &str = "https://discord.com/api/v10";

#[derive(Debug, Clone)]
pub struct RestClient {
    http: ReqwestClient,
    config: Arc<Config>,
    ratelimiter: Arc<RateLimiter>,
}

#[async_trait]
impl Http for RestClient {
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        self.request(Method::GET, path, None, None).await
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.request(Method::POST, path, Some(body), None).await
    }

    async fn patch(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        self.request(Method::PATCH, path, Some(body), None).await
    }

    async fn delete(&self, path: &str) -> Result<serde_json::Value> {
        self.request(Method::DELETE, path, None, None).await
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

        Ok(Self {
            http,
            config,
            ratelimiter: Arc::new(RateLimiter::new()),
        })
    }

    pub async fn get_gateway_bot(&self) -> Result<GetGatewayBot> {
        let value = self.request(Method::GET, "/gateway/bot", None, None).await?;
        serde_json::from_value(value).map_err(|e| DiscordError::Serialization(e.to_string()))
    }

    /// Internal request helper with rate limiting and retries
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
        reason: Option<&str>,
    ) -> Result<serde_json::Value> {
        let route = Route::new(method.clone(), path);
        let url = format!("{}{}", BASE_URL, path);

        loop {
            self.ratelimiter.await_bucket(&route).await;

            let mut req = self.http.request(method.clone(), &url);

            if let Some(r) = reason {
                let encoded = utf8_percent_encode(r, NON_ALPHANUMERIC).to_string();
                if let Ok(hv) = header::HeaderValue::from_str(&encoded) {
                    req = req.header("X-Audit-Log-Reason", hv);
                }
            }

            if let Some(b) = &body {
                req = req.json(b);
            }

            let response = req.send().await.map_err(|e| DiscordError::Http(e.to_string()))?;
            let status = response.status();
            let headers = response.headers().clone();

            self.ratelimiter.update(&route, &headers).await;

            if status.is_success() {
                // Handle 204 No Content
                if status == StatusCode::NO_CONTENT {
                    return Ok(serde_json::Value::Null);
                }
                return response.json::<serde_json::Value>().await
                    .map_err(|e| DiscordError::Serialization(e.to_string()));
            }

            if status == StatusCode::TOO_MANY_REQUESTS {
                let bytes = response.bytes().await.map_err(|e| DiscordError::Http(e.to_string()))?;
                let body_json: serde_json::Value = serde_json::from_slice(&bytes)
                    .unwrap_or(serde_json::json!({}));
                
                let is_global = body_json.get("global").and_then(|v| v.as_bool()).unwrap_or(false);
                let retry_after = body_json.get("retry_after").and_then(|v| v.as_f64()).unwrap_or(0.0);

                if is_global {
                    self.ratelimiter.handle_global_limit(retry_after).await;
                } else {
                    // Bucket specific 429. The `update` call above should have set the reset time.
                    // We just continue loop to wait in `await_bucket`.
                    // But we might want to log it.
                    tracing::warn!("Rate limited on {}. Retry after {}s", path, retry_after);
                }
                continue;
            }

            // Error parsing
            let bytes = response.bytes().await.map_err(|e| DiscordError::Http(e.to_string()))?;
            if let Ok(api_error) = serde_json::from_slice::<DiscordApiError>(&bytes) {
                return Err(DiscordError::Http(api_error.to_string()));
            }
            
            return Err(DiscordError::Http(format!("Request failed with status: {}", status)));
        }
    }

    pub async fn request_multipart(
        &self,
        method: Method,
        path: &str,
        form: reqwest::multipart::Form,
        reason: Option<&str>,
    ) -> Result<serde_json::Value> {
        let route = Route::new(method.clone(), path);
        let url = format!("{}{}", BASE_URL, path);

        self.ratelimiter.await_bucket(&route).await;

        let mut req = self.http.request(method.clone(), &url);

        if let Some(r) = reason {
            let encoded = utf8_percent_encode(r, NON_ALPHANUMERIC).to_string();
            if let Ok(hv) = header::HeaderValue::from_str(&encoded) {
                req = req.header("X-Audit-Log-Reason", hv);
            }
        }

        req = req.multipart(form);

        let response = req.send().await.map_err(|e| DiscordError::Http(e.to_string()))?;
        let status = response.status();
        let headers = response.headers().clone();

        self.ratelimiter.update(&route, &headers).await;

        if status.is_success() {
             if status == StatusCode::NO_CONTENT {
                return Ok(serde_json::Value::Null);
            }
            return response.json::<serde_json::Value>().await
                .map_err(|e| DiscordError::Serialization(e.to_string()));
        }

         if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(DiscordError::RateLimit);
         }

        let bytes = response.bytes().await.map_err(|e| DiscordError::Http(e.to_string()))?;
        if let Ok(api_error) = serde_json::from_slice::<DiscordApiError>(&bytes) {
            return Err(DiscordError::Http(api_error.to_string()));
        }
        
        Err(DiscordError::Http(format!("Request failed with status: {}", status)))
    }
}