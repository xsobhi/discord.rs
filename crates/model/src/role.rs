use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    pub color: u32,
    pub hoist: bool,
    pub icon: Option<String>,
    pub unicode_emoji: Option<String>,
    pub position: i32,
    pub permissions: String,
    pub managed: bool,
    pub mentionable: bool,
    #[serde(default)]
    pub tags: Option<RoleTags>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTags {
    pub bot_id: Option<Snowflake>,
    pub integration_id: Option<Snowflake>,
    pub premium_subscriber: Option<serde_json::Value>, // null if present
    pub subscription_listing_id: Option<Snowflake>,
    pub available_for_purchase: Option<serde_json::Value>,
    pub guild_connections: Option<serde_json::Value>,
}
