use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub roles: Vec<Snowflake>,
    pub joined_at: String, // ISO8601
    pub premium_since: Option<String>,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub flags: i32,
    #[serde(default)]
    pub pending: bool,
    pub permissions: Option<String>,
    pub communication_disabled_until: Option<String>,
}
