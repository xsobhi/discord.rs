use serde::{Deserialize, Serialize};
use crate::emoji::Emoji;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum ComponentType {
    ActionRow = 1,
    Button = 2,
    StringSelect = 3,
    TextInput = 4,
    UserSelect = 5,
    RoleSelect = 6,
    MentionableSelect = 7,
    ChannelSelect = 8,
    Unknown(u8),
}

impl From<u8> for ComponentType {
    fn from(v: u8) -> Self {
        match v {
            1 => ComponentType::ActionRow,
            2 => ComponentType::Button,
            3 => ComponentType::StringSelect,
            4 => ComponentType::TextInput,
            5 => ComponentType::UserSelect,
            6 => ComponentType::RoleSelect,
            7 => ComponentType::MentionableSelect,
            8 => ComponentType::ChannelSelect,
            _ => ComponentType::Unknown(v),
        }
    }
}

impl From<ComponentType> for u8 {
    fn from(v: ComponentType) -> u8 {
        match v {
            ComponentType::ActionRow => 1,
            ComponentType::Button => 2,
            ComponentType::StringSelect => 3,
            ComponentType::TextInput => 4,
            ComponentType::UserSelect => 5,
            ComponentType::RoleSelect => 6,
            ComponentType::MentionableSelect => 7,
            ComponentType::ChannelSelect => 8,
            ComponentType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[serde(from = "u8", into = "u8")]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
    Link = 5,
    Unknown(u8),
}

impl From<u8> for ButtonStyle {
    fn from(v: u8) -> Self {
        match v {
            1 => ButtonStyle::Primary,
            2 => ButtonStyle::Secondary,
            3 => ButtonStyle::Success,
            4 => ButtonStyle::Danger,
            5 => ButtonStyle::Link,
            _ => ButtonStyle::Unknown(v),
        }
    }
}

impl From<ButtonStyle> for u8 {
    fn from(v: ButtonStyle) -> u8 {
        match v {
            ButtonStyle::Primary => 1,
            ButtonStyle::Secondary => 2,
            ButtonStyle::Success => 3,
            ButtonStyle::Danger => 4,
            ButtonStyle::Link => 5,
            ButtonStyle::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Component {
    #[serde(rename = "1")]
    ActionRow(ActionRow),
    #[serde(rename = "2")]
    Button(Button),
    #[serde(rename = "3")]
    StringSelect(SelectMenu),
    #[serde(rename = "4")]
    TextInput(TextInput),
    #[serde(rename = "5")]
    UserSelect(SelectMenu),
    #[serde(rename = "6")]
    RoleSelect(SelectMenu),
    #[serde(rename = "7")]
    MentionableSelect(SelectMenu),
    #[serde(rename = "8")]
    ChannelSelect(SelectMenu),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRow {
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Button {
    #[serde(default)]
    pub style: Option<ButtonStyle>,
    pub label: Option<String>,
    pub emoji: Option<Emoji>,
    pub custom_id: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectMenu {
    pub custom_id: String,
    pub options: Option<Vec<SelectMenuOption>>,
    pub channel_types: Option<Vec<u8>>,
    pub placeholder: Option<String>,
    pub min_values: Option<u32>,
    pub max_values: Option<u32>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectMenuOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<Emoji>,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TextInput {
    pub custom_id: String,
    pub style: u8, // 1 = Short, 2 = Paragraph
    pub label: String,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    #[serde(default)]
    pub required: bool,
    pub value: Option<String>,
    pub placeholder: Option<String>,
}
