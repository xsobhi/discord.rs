use discord_rs_model::interaction::{InteractionResponse, InteractionResponseType, InteractionResponseData};
use discord_rs_model::embed::Embed;

pub struct InteractionResponseBuilder {
    kind: InteractionResponseType,
    data: Option<InteractionResponseData>,
}

impl InteractionResponseBuilder {
    pub fn new(kind: InteractionResponseType) -> Self {
        Self { kind, data: None }
    }

    pub fn reply(content: impl Into<String>) -> Self {
        let mut builder = Self::new(InteractionResponseType::ChannelMessageWithSource);
        builder.content(content);
        builder
    }

    pub fn defer() -> Self {
        Self::new(InteractionResponseType::DeferredChannelMessageWithSource)
    }

    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        let data = self.data.get_or_insert_with(|| InteractionResponseData {
            tts: None,
            content: None,
            embeds: None,
            allowed_mentions: None,
            flags: None,
            components: None,
            choices: None,
            custom_id: None,
            title: None,
        });
        data.content = Some(content.into());
        self
    }

    pub fn add_embed(&mut self, embed: impl Into<Embed>) -> &mut Self {
        let data = self.data.get_or_insert_with(|| InteractionResponseData {
            tts: None,
            content: None,
            embeds: None,
            allowed_mentions: None,
            flags: None,
            components: None,
            choices: None,
            custom_id: None,
            title: None,
        });
        let embeds = data.embeds.get_or_insert_with(Vec::new);
        embeds.push(embed.into());
        self
    }

    pub fn ephemeral(&mut self) -> &mut Self {
        let data = self.data.get_or_insert_with(|| InteractionResponseData {
            tts: None,
            content: None,
            embeds: None,
            allowed_mentions: None,
            flags: None,
            components: None,
            choices: None,
            custom_id: None,
            title: None,
        });
        let flags = data.flags.get_or_insert(0);
        *flags |= 1 << 6; // EPHEMERAL
        self
    }

    pub fn build(self) -> InteractionResponse {
        InteractionResponse {
            kind: self.kind,
            data: self.data,
        }
    }
}

impl From<InteractionResponseBuilder> for InteractionResponse {
    fn from(builder: InteractionResponseBuilder) -> Self {
        builder.build()
    }
}
