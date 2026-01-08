use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: bool,
    #[serde(default)]
    pub system: bool,
    #[serde(default)]
    pub mfa_enabled: bool,
    pub banner: Option<String>,
    pub accent_color: Option<u32>,
    pub locale: Option<String>,
    #[serde(default)]
    pub verified: bool,
    pub email: Option<String>,
    #[serde(default)]
    pub flags: u64,
    #[serde(default)]
    pub premium_type: u8,
    #[serde(default)]
    pub public_flags: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialUser {
    pub id: Snowflake,
    pub username: Option<String>,
    pub discriminator: Option<String>,
    pub avatar: Option<String>,
}
