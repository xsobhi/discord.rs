use discord_rs_model::message::{AllowedMentions, MessageReference};
use discord_rs_model::embed::Embed;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MessageBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mentions: Option<AllowedMentions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_reference: Option<MessageReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts: Option<bool>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn add_embed(mut self, embed: impl Into<Embed>) -> Self {
        let embeds = self.embeds.get_or_insert_with(Vec::new);
        embeds.push(embed.into());
        self
    }

    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        self.embeds = Some(embeds);
        self
    }

    pub fn allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    pub fn mention_everyone(mut self) -> Self {
        let am = self.allowed_mentions.get_or_insert_with(AllowedMentions::default);
        if !am.parse.contains(&discord_rs_model::message::AllowedMentionType::Everyone) {
            am.parse.push(discord_rs_model::message::AllowedMentionType::Everyone);
        }
        self
    }

    pub fn reply(mut self, message_id: discord_rs_core::Snowflake, channel_id: Option<discord_rs_core::Snowflake>, fail_if_not_exists: bool) -> Self {
        self.message_reference = Some(MessageReference {
            message_id: Some(message_id),
            channel_id,
            guild_id: None,
            fail_if_not_exists: Some(fail_if_not_exists),
        });
        self
    }

    pub fn tts(mut self, tts: bool) -> Self {
        self.tts = Some(tts);
        self
    }

    pub fn build(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}

impl From<MessageBuilder> for serde_json::Value {
    fn from(builder: MessageBuilder) -> Self {
        builder.build()
    }
}
