use serde::{Deserialize, Serialize};
use serde_json::Value;
use discord_rs_core::{Intents, Snowflake};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum OpCode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
    Unknown(u8),
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        match v {
            0 => OpCode::Dispatch,
            1 => OpCode::Heartbeat,
            2 => OpCode::Identify,
            3 => OpCode::PresenceUpdate,
            4 => OpCode::VoiceStateUpdate,
            6 => OpCode::Resume,
            7 => OpCode::Reconnect,
            8 => OpCode::RequestGuildMembers,
            9 => OpCode::InvalidSession,
            10 => OpCode::Hello,
            11 => OpCode::HeartbeatAck,
            _ => OpCode::Unknown(v),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(v: OpCode) -> Self {
        match v {
            OpCode::Dispatch => 0,
            OpCode::Heartbeat => 1,
            OpCode::Identify => 2,
            OpCode::PresenceUpdate => 3,
            OpCode::VoiceStateUpdate => 4,
            OpCode::Resume => 6,
            OpCode::Reconnect => 7,
            OpCode::RequestGuildMembers => 8,
            OpCode::InvalidSession => 9,
            OpCode::Hello => 10,
            OpCode::HeartbeatAck => 11,
            OpCode::Unknown(u) => u,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayPayload<T> {
    pub op: OpCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identify {
    pub token: String,
    pub properties: IdentifyProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_threshold: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<[u64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Value>,
    pub intents: Intents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentifyProperties {
    pub os: String,
    pub browser: String,
    pub device: String,
}

#[derive(Debug, Deserialize)]
pub struct Hello {
    pub heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct Ready {
    pub v: u8,
    pub user: CurrentUser,
    pub guilds: Vec<UnavailableGuild>,
    pub session_id: String,
    pub resume_gateway_url: String,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUser {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub bot: bool,
}

#[derive(Debug, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    pub unavailable: bool,
}
