use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emoji {
    pub id: Option<Snowflake>,
    pub name: Option<String>,
    #[serde(default)]
    pub roles: Vec<Snowflake>,
    pub user: Option<User>,
    #[serde(default)]
    pub require_colons: bool,
    #[serde(default)]
    pub managed: bool,
    #[serde(default)]
    pub animated: bool,
    #[serde(default)]
    pub available: bool,
}
