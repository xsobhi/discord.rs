use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;
use crate::member::Member;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: ChannelType,
    pub guild_id: Option<Snowflake>,
    pub position: Option<i32>,
    #[serde(default)]
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub name: Option<String>,
    pub topic: Option<String>,
    #[serde(default)]
    pub nsfw: bool,
    pub last_message_id: Option<Snowflake>,
    pub bitrate: Option<i32>,
    pub user_limit: Option<i32>,
    pub rate_limit_per_user: Option<i32>,
    #[serde(default)]
    pub recipients: Vec<User>,
    pub icon: Option<String>,
    pub owner_id: Option<Snowflake>,
    pub application_id: Option<Snowflake>,
    pub parent_id: Option<Snowflake>,
    pub last_pin_timestamp: Option<String>,
    pub rtc_region: Option<String>,
    pub video_quality_mode: Option<i32>,
    pub message_count: Option<i32>,
    pub member_count: Option<i32>,
    pub thread_metadata: Option<ThreadMetadata>,
    pub member: Option<Member>, // Thread member
    pub default_auto_archive_duration: Option<i32>,
    pub permissions: Option<String>,
    #[serde(default)]
    pub flags: i32,
    pub total_message_sent: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum ChannelType {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
    Unknown(u8),
}

impl From<u8> for ChannelType {
    fn from(v: u8) -> Self {
        match v {
            0 => ChannelType::GuildText,
            1 => ChannelType::DM,
            2 => ChannelType::GuildVoice,
            3 => ChannelType::GroupDM,
            4 => ChannelType::GuildCategory,
            5 => ChannelType::GuildAnnouncement,
            10 => ChannelType::AnnouncementThread,
            11 => ChannelType::PublicThread,
            12 => ChannelType::PrivateThread,
            13 => ChannelType::GuildStageVoice,
            14 => ChannelType::GuildDirectory,
            15 => ChannelType::GuildForum,
            _ => ChannelType::Unknown(v),
        }
    }
}

impl From<ChannelType> for u8 {
    fn from(v: ChannelType) -> Self {
        match v {
            ChannelType::GuildText => 0,
            ChannelType::DM => 1,
            ChannelType::GuildVoice => 2,
            ChannelType::GroupDM => 3,
            ChannelType::GuildCategory => 4,
            ChannelType::GuildAnnouncement => 5,
            ChannelType::AnnouncementThread => 10,
            ChannelType::PublicThread => 11,
            ChannelType::PrivateThread => 12,
            ChannelType::GuildStageVoice => 13,
            ChannelType::GuildDirectory => 14,
            ChannelType::GuildForum => 15,
            ChannelType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionOverwrite {
    pub id: Snowflake,
    #[serde(rename = "type")]
    pub kind: i32, // 0 role, 1 member
    pub allow: String,
    pub deny: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMetadata {
    pub archived: bool,
    pub auto_archive_duration: i32,
    pub archive_timestamp: String,
    pub locked: bool,
    #[serde(default)]
    pub invitable: bool,
    pub create_timestamp: Option<String>,
}
