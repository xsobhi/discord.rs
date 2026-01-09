pub mod embed;
pub mod message;
pub mod interaction;
pub mod component;

pub use embed::EmbedBuilder;
pub use message::MessageBuilder;
pub use interaction::InteractionResponseBuilder;
pub use component::{ActionRowBuilder, ButtonBuilder, SelectMenuBuilder};

use discord_rs_core::{Context, Snowflake, Result, DiscordError};
use discord_rs_model::{Message, Interaction};
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
        let app_id = ctx.config.application_id.ok_or(DiscordError::Configuration("application_id required for follow-up".to_string()))?;
        let path = format!("/webhooks/{}/{}/messages", app_id, self.token);
        let body = builder.build();
        let value = ctx.http.post(&path, body).await?;
        let msg: Message = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(Box::new(msg))
    }

    async fn edit_reply(&self, ctx: &Context, builder: MessageBuilder) -> Result<Box<Message>> {
        let app_id = ctx.config.application_id.ok_or(DiscordError::Configuration("application_id required for editing reply".to_string()))?;
        let path = format!("/webhooks/{}/{}/messages/@original", app_id, self.token);
        let body = builder.build();
        let value = ctx.http.patch(&path, body).await?;
        let msg: Message = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(Box::new(msg))
    }

    async fn delete_reply(&self, ctx: &Context) -> Result<()> {
        let app_id = ctx.config.application_id.ok_or(DiscordError::Configuration("application_id required for deleting reply".to_string()))?;
        let path = format!("/webhooks/{}/{}/messages/@original", app_id, self.token);
        ctx.http.delete(&path).await?;
        Ok(())
    }
}
