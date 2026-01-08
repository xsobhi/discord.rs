use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::role::Role;
use crate::emoji::Emoji;
use crate::sticker::Sticker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub icon_hash: Option<String>,
    pub splash: Option<String>,
    pub discovery_splash: Option<String>,
    pub owner: Option<bool>,
    pub owner_id: Snowflake,
    pub permissions: Option<String>,
    pub afk_channel_id: Option<Snowflake>,
    pub afk_timeout: i32,
    #[serde(default)]
    pub widget_enabled: bool,
    pub widget_channel_id: Option<Snowflake>,
    pub verification_level: i32,
    pub default_message_notifications: i32,
    pub explicit_content_filter: i32,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default)]
    pub emojis: Vec<Emoji>,
    #[serde(default)]
    pub features: Vec<String>,
    pub mfa_level: i32,
    pub application_id: Option<Snowflake>,
    pub system_channel_id: Option<Snowflake>,
    pub system_channel_flags: i32,
    pub rules_channel_id: Option<Snowflake>,
    pub max_presences: Option<i32>,
    pub max_members: Option<i32>,
    pub vanity_url_code: Option<String>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub premium_tier: i32,
    pub premium_subscription_count: Option<i32>,
    pub preferred_locale: String,
    pub public_updates_channel_id: Option<Snowflake>,
    pub max_video_channel_users: Option<i32>,
    pub approximate_member_count: Option<i32>,
    pub approximate_presence_count: Option<i32>,
    pub nsfw_level: i32,
    #[serde(default)]
    pub stickers: Vec<Sticker>,
    #[serde(default)]
    pub premium_progress_bar_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnavailableGuild {
    pub id: Snowflake,
    #[serde(default)]
    pub unavailable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialGuild {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub owner: Option<bool>,
    pub permissions: Option<String>,
}
