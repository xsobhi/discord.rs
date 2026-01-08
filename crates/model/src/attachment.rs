use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Snowflake,
    pub filename: String,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub size: i32,
    pub url: String,
    pub proxy_url: String,
    pub height: Option<i32>,
    pub width: Option<i32>,
    #[serde(default)]
    pub ephemeral: bool,
}
