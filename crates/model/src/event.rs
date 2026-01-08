use serde::{Deserialize, Serialize};
use crate::gateway::Ready;
use crate::message::Message;
use crate::guild::Guild;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")] // standard discord dispatch format mapping
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    Ready(Ready),
    Resumed(serde_json::Value),
    MessageCreate(Box<Message>),
    MessageUpdate(Box<Message>),
    MessageDelete(serde_json::Value),
    GuildCreate(Guild),
    GuildUpdate(Guild),
    GuildDelete(serde_json::Value),
    // We will add more events as we implement more models/features
    #[serde(other)]
    Unknown,
}
