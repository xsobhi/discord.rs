use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub id: Snowflake,
    pub pack_id: Option<Snowflake>,
    pub name: String,
    pub description: Option<String>,
    pub tags: String, // CSV
    pub asset: Option<String>, // Deprecated
    pub type_id: StickerType, // 'type' is reserved
    pub format_type: StickerFormatType,
    #[serde(default)]
    pub available: bool,
    pub guild_id: Option<Snowflake>,
    pub user: Option<User>,
    pub sort_value: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum StickerType {
    Standard = 1,
    Guild = 2,
    Unknown(u8),
}

impl From<u8> for StickerType {
    fn from(v: u8) -> Self {
        match v {
            1 => StickerType::Standard,
            2 => StickerType::Guild,
            _ => StickerType::Unknown(v),
        }
    }
}

impl From<StickerType> for u8 {
    fn from(v: StickerType) -> Self {
        match v {
            StickerType::Standard => 1,
            StickerType::Guild => 2,
            StickerType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum StickerFormatType {
    Png = 1,
    Apng = 2,
    Lottie = 3,
    Gif = 4,
    Unknown(u8),
}

impl From<u8> for StickerFormatType {
    fn from(v: u8) -> Self {
        match v {
            1 => StickerFormatType::Png,
            2 => StickerFormatType::Apng,
            3 => StickerFormatType::Lottie,
            4 => StickerFormatType::Gif,
            _ => StickerFormatType::Unknown(v),
        }
    }
}

impl From<StickerFormatType> for u8 {
    fn from(v: StickerFormatType) -> Self {
        match v {
            StickerFormatType::Png => 1,
            StickerFormatType::Apng => 2,
            StickerFormatType::Lottie => 3,
            StickerFormatType::Gif => 4,
            StickerFormatType::Unknown(v) => v,
        }
    }
}
