use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;
use crate::member::Member;
use crate::message::Message;
use crate::channel::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: Snowflake,
    pub application_id: Snowflake,
    #[serde(rename = "type")]
    pub kind: InteractionType,
    pub data: Option<InteractionData>,
    pub guild_id: Option<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub member: Option<Member>,
    pub user: Option<User>,
    pub token: String,
    pub version: u8,
    pub message: Option<Message>,
    pub app_permissions: Option<String>,
    pub locale: Option<String>,
    pub guild_locale: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum InteractionType {
    Ping = 1,
    ApplicationCommand = 2,
    MessageComponent = 3,
    ApplicationCommandAutocomplete = 4,
    ModalSubmit = 5,
    Unknown(u8),
}

impl From<u8> for InteractionType {
    fn from(v: u8) -> Self {
        match v {
            1 => InteractionType::Ping,
            2 => InteractionType::ApplicationCommand,
            3 => InteractionType::MessageComponent,
            4 => InteractionType::ApplicationCommandAutocomplete,
            5 => InteractionType::ModalSubmit,
            _ => InteractionType::Unknown(v),
        }
    }
}

impl From<InteractionType> for u8 {
    fn from(v: InteractionType) -> u8 {
        match v {
            InteractionType::Ping => 1,
            InteractionType::ApplicationCommand => 2,
            InteractionType::MessageComponent => 3,
            InteractionType::ApplicationCommandAutocomplete => 4,
            InteractionType::ModalSubmit => 5,
            InteractionType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InteractionData {
    ApplicationCommand(ApplicationCommandData),
    MessageComponent(MessageComponentData),
    ModalSubmit(ModalSubmitData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCommandData {
    pub id: Snowflake,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ApplicationCommandType,
    pub resolved: Option<ResolvedData>,
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    pub guild_id: Option<Snowflake>,
    pub target_id: Option<Snowflake>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum ApplicationCommandType {
    ChatInput = 1,
    User = 2,
    Message = 3,
    Unknown(u8),
}

impl From<u8> for ApplicationCommandType {
    fn from(v: u8) -> Self {
        match v {
            1 => ApplicationCommandType::ChatInput,
            2 => ApplicationCommandType::User,
            3 => ApplicationCommandType::Message,
            _ => ApplicationCommandType::Unknown(v),
        }
    }
}

impl From<ApplicationCommandType> for u8 {
    fn from(v: ApplicationCommandType) -> u8 {
        match v {
            ApplicationCommandType::ChatInput => 1,
            ApplicationCommandType::User => 2,
            ApplicationCommandType::Message => 3,
            ApplicationCommandType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedData {
    pub users: Option<std::collections::HashMap<Snowflake, User>>,
    pub members: Option<std::collections::HashMap<Snowflake, Member>>,
    pub roles: Option<std::collections::HashMap<Snowflake, crate::role::Role>>,
    pub channels: Option<std::collections::HashMap<Snowflake, Channel>>,
    pub messages: Option<std::collections::HashMap<Snowflake, Message>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCommandInteractionDataOption {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: u8, // ApplicationCommandOptionType
    pub value: Option<serde_json::Value>,
    pub options: Option<Vec<ApplicationCommandInteractionDataOption>>,
    pub focused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageComponentData {
    pub custom_id: String,
    pub component_type: u8,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalSubmitData {
    pub custom_id: String,
    pub components: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    pub kind: InteractionResponseType,
    pub data: Option<InteractionResponseData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum InteractionResponseType {
    Pong = 1,
    ChannelMessageWithSource = 4,
    DeferredChannelMessageWithSource = 5,
    DeferredUpdateMessage = 6,
    UpdateMessage = 7,
    ApplicationCommandAutocompleteResult = 8,
    Modal = 9,
    Unknown(u8),
}

impl From<u8> for InteractionResponseType {
    fn from(v: u8) -> Self {
        match v {
            1 => InteractionResponseType::Pong,
            4 => InteractionResponseType::ChannelMessageWithSource,
            5 => InteractionResponseType::DeferredChannelMessageWithSource,
            6 => InteractionResponseType::DeferredUpdateMessage,
            7 => InteractionResponseType::UpdateMessage,
            8 => InteractionResponseType::ApplicationCommandAutocompleteResult,
            9 => InteractionResponseType::Modal,
            _ => InteractionResponseType::Unknown(v),
        }
    }
}

impl From<InteractionResponseType> for u8 {
    fn from(v: InteractionResponseType) -> u8 {
        match v {
            InteractionResponseType::Pong => 1,
            InteractionResponseType::ChannelMessageWithSource => 4,
            InteractionResponseType::DeferredChannelMessageWithSource => 5,
            InteractionResponseType::DeferredUpdateMessage => 6,
            InteractionResponseType::UpdateMessage => 7,
            InteractionResponseType::ApplicationCommandAutocompleteResult => 8,
            InteractionResponseType::Modal => 9,
            InteractionResponseType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResponseData {
    pub tts: Option<bool>,
    pub content: Option<String>,
    pub embeds: Option<Vec<crate::embed::Embed>>,
    pub allowed_mentions: Option<crate::message::AllowedMentions>,
    pub flags: Option<u64>,
    pub components: Option<Vec<serde_json::Value>>,
    pub choices: Option<Vec<serde_json::Value>>,
    pub custom_id: Option<String>,
    pub title: Option<String>,
}
