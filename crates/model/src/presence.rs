use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub since: Option<u64>,
    pub activities: Vec<Activity>,
    pub status: PresenceStatus,
    pub afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ActivityType,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Online,
    Dnd,
    Idle,
    Invisible,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum ActivityType {
    Game = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5,
    Unknown(u8),
}

impl From<u8> for ActivityType {
    fn from(v: u8) -> Self {
        match v {
            0 => ActivityType::Game,
            1 => ActivityType::Streaming,
            2 => ActivityType::Listening,
            3 => ActivityType::Watching,
            4 => ActivityType::Custom,
            5 => ActivityType::Competing,
            _ => ActivityType::Unknown(v),
        }
    }
}

impl From<ActivityType> for u8 {
    fn from(v: ActivityType) -> u8 {
        match v {
            ActivityType::Game => 0,
            ActivityType::Streaming => 1,
            ActivityType::Listening => 2,
            ActivityType::Watching => 3,
            ActivityType::Custom => 4,
            ActivityType::Competing => 5,
            ActivityType::Unknown(v) => v,
        }
    }
}
