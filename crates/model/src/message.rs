use discord_rs_core::Snowflake;
use serde::{Deserialize, Serialize};
use crate::user::User;
use crate::member::Member; // Partial member usually
use crate::attachment::Attachment;
use crate::embed::Embed;
use crate::reaction::Reaction;
use crate::sticker::Sticker; // Actually StickerItem usually for messages
use crate::channel::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub author: User,
    pub member: Option<Member>,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
    pub tts: bool,
    pub mention_everyone: bool,
    #[serde(default)]
    pub mentions: Vec<User>, // With partial member field sometimes
    #[serde(default)]
    pub mention_roles: Vec<Snowflake>,
    #[serde(default)]
    pub mention_channels: Vec<ChannelMention>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    pub nonce: Option<serde_json::Value>, // int or string
    #[serde(default)]
    pub pinned: bool,
    pub webhook_id: Option<Snowflake>,
    #[serde(rename = "type")]
    pub kind: MessageType,
    pub activity: Option<MessageActivity>,
    pub application: Option<MessageApplication>,
    pub application_id: Option<Snowflake>,
    pub message_reference: Option<MessageReference>,
    #[serde(default)]
    pub flags: i32,
    pub referenced_message: Option<Box<Message>>, // recursive
    pub interaction: Option<serde_json::Value>, // TODO: MessageInteraction
    pub thread: Option<Channel>,
    #[serde(default)]
    pub components: Vec<serde_json::Value>, // TODO: Components
    #[serde(default)]
    pub sticker_items: Vec<StickerItem>,
    #[serde(default)]
    pub stickers: Vec<Sticker>, // Deprecated
    pub position: Option<i32>,
    pub role_subscription_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum MessageType {
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelNameChange = 4,
    ChannelIconChange = 5,
    ChannelPinnedMessage = 6,
    UserJoin = 7,
    GuildBoost = 8,
    GuildBoostTier1 = 9,
    GuildBoostTier2 = 10,
    GuildBoostTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryRequalified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ChatInputCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
    AutoModerationAction = 24,
    RoleSubscriptionPurchase = 25,
    InteractionPremiumUpsell = 26,
    StageStart = 27,
    StageEnd = 28,
    StageSpeaker = 29,
    StageTopic = 31,
    GuildApplicationPremiumSubscription = 32,
    Unknown(u8),
}

impl From<u8> for MessageType {
    fn from(v: u8) -> Self {
        match v {
            0 => MessageType::Default,
            1 => MessageType::RecipientAdd,
            2 => MessageType::RecipientRemove,
            3 => MessageType::Call,
            4 => MessageType::ChannelNameChange,
            5 => MessageType::ChannelIconChange,
            6 => MessageType::ChannelPinnedMessage,
            7 => MessageType::UserJoin,
            8 => MessageType::GuildBoost,
            9 => MessageType::GuildBoostTier1,
            10 => MessageType::GuildBoostTier2,
            11 => MessageType::GuildBoostTier3,
            12 => MessageType::ChannelFollowAdd,
            14 => MessageType::GuildDiscoveryDisqualified,
            15 => MessageType::GuildDiscoveryRequalified,
            16 => MessageType::GuildDiscoveryGracePeriodInitialWarning,
            17 => MessageType::GuildDiscoveryGracePeriodFinalWarning,
            18 => MessageType::ThreadCreated,
            19 => MessageType::Reply,
            20 => MessageType::ChatInputCommand,
            21 => MessageType::ThreadStarterMessage,
            22 => MessageType::GuildInviteReminder,
            23 => MessageType::ContextMenuCommand,
            24 => MessageType::AutoModerationAction,
            25 => MessageType::RoleSubscriptionPurchase,
            26 => MessageType::InteractionPremiumUpsell,
            27 => MessageType::StageStart,
            28 => MessageType::StageEnd,
            29 => MessageType::StageSpeaker,
            31 => MessageType::StageTopic,
            32 => MessageType::GuildApplicationPremiumSubscription,
            _ => MessageType::Unknown(v),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(v: MessageType) -> Self {
        match v {
            MessageType::Default => 0,
            MessageType::RecipientAdd => 1,
            MessageType::RecipientRemove => 2,
            MessageType::Call => 3,
            MessageType::ChannelNameChange => 4,
            MessageType::ChannelIconChange => 5,
            MessageType::ChannelPinnedMessage => 6,
            MessageType::UserJoin => 7,
            MessageType::GuildBoost => 8,
            MessageType::GuildBoostTier1 => 9,
            MessageType::GuildBoostTier2 => 10,
            MessageType::GuildBoostTier3 => 11,
            MessageType::ChannelFollowAdd => 12,
            MessageType::GuildDiscoveryDisqualified => 14,
            MessageType::GuildDiscoveryRequalified => 15,
            MessageType::GuildDiscoveryGracePeriodInitialWarning => 16,
            MessageType::GuildDiscoveryGracePeriodFinalWarning => 17,
            MessageType::ThreadCreated => 18,
            MessageType::Reply => 19,
            MessageType::ChatInputCommand => 20,
            MessageType::ThreadStarterMessage => 21,
            MessageType::GuildInviteReminder => 22,
            MessageType::ContextMenuCommand => 23,
            MessageType::AutoModerationAction => 24,
            MessageType::RoleSubscriptionPurchase => 25,
            MessageType::InteractionPremiumUpsell => 26,
            MessageType::StageStart => 27,
            MessageType::StageEnd => 28,
            MessageType::StageSpeaker => 29,
            MessageType::StageTopic => 31,
            MessageType::GuildApplicationPremiumSubscription => 32,
            MessageType::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMention {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    #[serde(rename = "type")]
    pub kind: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageActivity {
    #[serde(rename = "type")]
    pub kind: i32,
    pub party_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageApplication {
    pub id: Snowflake,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReference {
    pub message_id: Option<Snowflake>,
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
    pub fail_if_not_exists: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllowedMentions {
    #[serde(default)]
    pub parse: Vec<AllowedMentionType>,
    #[serde(default)]
    pub roles: Vec<Snowflake>,
    #[serde(default)]
    pub users: Vec<Snowflake>,
    #[serde(default)]
    pub replied_user: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AllowedMentionType {
    Roles,
    Users,
    Everyone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerItem {
    pub id: Snowflake,
    pub name: String,
    pub format_type: i32,
}
