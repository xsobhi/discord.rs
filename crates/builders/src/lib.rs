pub mod embed;
pub mod message;
pub mod interaction;

pub use embed::EmbedBuilder;
pub use message::MessageBuilder;
pub use interaction::InteractionResponseBuilder;

use discord_rs_core::{Context, Snowflake, Result, DiscordError};
use discord_rs_model::{Message, Interaction, InteractionResponse};
use async_trait::async_trait;

#[async_trait]
pub trait MessageSend {
    async fn send(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>>;
}

#[async_trait]
impl MessageSend for Snowflake {
    async fn send(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>> {
        let path = format!("/channels/{}/messages", self);
        let body = builder.build();
        let value = ctx.http.post(&path, body).await?;
        let msg: Message = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(Box::new(msg))
    }
}

#[async_trait]
pub trait MessageReply {
    async fn reply(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>>;
}

#[async_trait]
impl MessageReply for Message {
    async fn reply(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>> {
        let builder = builder.reply(self.id, Some(self.channel_id), true);
        self.channel_id.send(ctx, builder).await
    }
}

#[async_trait]
pub trait InteractionReply {
    async fn reply(&self, ctx: &Context, builder: InteractionResponseBuilder) -> Result<()>;
    async fn defer(&self, ctx: &Context) -> Result<()>;
    async fn follow_up(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>>;
    async fn edit_reply(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>>;
    async fn delete_reply(&self, ctx: &Context) -> Result<()>;
}

#[async_trait]
impl InteractionReply for Interaction {
    async fn reply(&self, ctx: &Context, builder: InteractionResponseBuilder) -> Result<()> {
        let path = format!("/interactions/{}/{}/callback", self.id, self.token);
        let body = serde_json::to_value(builder.build())
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        ctx.http.post(&path, body).await?;
        Ok(())
    }

    async fn defer(&self, ctx: &Context) -> Result<()> {
        let builder = InteractionResponseBuilder::defer();
        self.reply(ctx, builder).await
    }

    async fn follow_up(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>> {
        let path = format!("/webhooks/{}/{}/messages", ctx.config.application_id.expect("application_id required for follow-up"), self.token);
        let body = builder.build();
        let value = ctx.http.post(&path, body).await?;
        let msg: Message = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(Box::new(msg))
    }

    async fn edit_reply(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>> {
        let path = format!("/webhooks/{}/{}/messages/@original", ctx.config.application_id.expect("application_id required for editing reply"), self.token);
        let body = builder.build();
        let value = ctx.http.patch(&path, body).await?;
        let msg: Message = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(Box::new(msg))
    }

    async fn delete_reply(&self, ctx: &Context) -> Result<()> {
        let path = format!("/webhooks/{}/{}/messages/@original", ctx.config.application_id.expect("application_id required for deleting reply"), self.token);
        ctx.http.delete(&path).await?;
        Ok(())
    }
}
