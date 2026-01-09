use std::sync::Arc;
use discord_rs_core::{Result, Snowflake, DiscordError, traits::Http};
use crate::cache::Cache;
use discord_rs_model::{User, Guild, Channel, Member};

pub struct UserManager {
    cache: Arc<Cache>,
    http: Arc<dyn Http>,
}

impl UserManager {
    pub fn new(cache: Arc<Cache>, http: Arc<dyn Http>) -> Self {
        Self { cache, http }
    }

    pub fn get(&self, id: Snowflake) -> Option<Arc<User>> {
        self.cache.users.get(&id).map(|r| r.value().clone())
    }

    pub async fn fetch(&self, id: Snowflake) -> Result<Arc<User>> {
        let value = self.http.get(&format!("/users/{}", id)).await?;
        let user: User = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(self.cache.update_user(user))
    }
}

pub struct GuildManager {
    cache: Arc<Cache>,
    http: Arc<dyn Http>,
}

impl GuildManager {
    pub fn new(cache: Arc<Cache>, http: Arc<dyn Http>) -> Self {
        Self { cache, http }
    }

    pub fn get(&self, id: Snowflake) -> Option<Arc<Guild>> {
        self.cache.guilds.get(&id).map(|r| r.value().clone())
    }

    pub async fn fetch(&self, id: Snowflake) -> Result<Arc<Guild>> {
        let value = self.http.get(&format!("/guilds/{}", id)).await?;
        let guild: Guild = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(self.cache.update_guild(guild))
    }
}

pub struct ChannelManager {
    cache: Arc<Cache>,
    http: Arc<dyn Http>,
}

impl ChannelManager {
    pub fn new(cache: Arc<Cache>, http: Arc<dyn Http>) -> Self {
        Self { cache, http }
    }

    pub fn get(&self, id: Snowflake) -> Option<Arc<Channel>> {
        self.cache.channels.get(&id).map(|r| r.value().clone())
    }

    pub async fn fetch(&self, id: Snowflake) -> Result<Arc<Channel>> {
        let value = self.http.get(&format!("/channels/{}", id)).await?;
        let channel: Channel = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(self.cache.update_channel(channel))
    }
}

pub struct GuildMemberManager {
    guild_id: Snowflake,
    cache: Arc<Cache>,
    http: Arc<dyn Http>,
}

impl GuildMemberManager {
    pub fn new(guild_id: Snowflake, cache: Arc<Cache>, http: Arc<dyn Http>) -> Self {
        Self { guild_id, cache, http }
    }

    pub fn get(&self, user_id: Snowflake) -> Option<Arc<Member>> {
        self.cache.members.get(&self.guild_id)
            .and_then(|g| g.get(&user_id).map(|r| r.value().clone()))
    }

    pub async fn fetch(&self, user_id: Snowflake) -> Result<Arc<Member>> {
        let value = self.http.get(&format!("/guilds/{}/members/{}", self.guild_id, user_id)).await?;
        let member: Member = serde_json::from_value(value)
            .map_err(|e| DiscordError::Serialization(e.to_string()))?;
        Ok(self.cache.update_member(self.guild_id, member))
    }
}