pub mod cache;
pub mod managers;

pub use cache::Cache;
pub use managers::*;

use discord_rs_model::event::Event;
use std::sync::Arc;
use discord_rs_core::Context;

pub fn update_cache_from_event(cache: &Arc<Cache>, event: &Event) {
    match event {
        Event::Ready(ready) => {
            cache.update_user(ready.user.clone());
        }
        Event::GuildCreate(guild) => {
            cache.update_guild(guild.clone());
        }
        Event::MessageCreate(msg) => {
            cache.update_user(msg.author.clone());
            if let (Some(guild_id), Some(member)) = (msg.guild_id, &msg.member) {
                let mut member = member.clone();
                member.user = Some(msg.author.clone());
                cache.update_member(guild_id, member);
            }
        }
        _ => {}
    }
}

pub trait ContextCacheExt {
    fn cache(&self) -> Arc<Cache>;
    fn users(&self) -> UserManager;
    fn guilds(&self) -> GuildManager;
    fn channels(&self) -> ChannelManager;
}

impl ContextCacheExt for Context {
    fn cache(&self) -> Arc<Cache> {
        self.cache.clone().downcast::<Cache>().expect("Cache not found in Context")
    }

    fn users(&self) -> UserManager {
        UserManager::new(self.cache(), self.http.clone())
    }

    fn guilds(&self) -> GuildManager {
        GuildManager::new(self.cache(), self.http.clone())
    }

    fn channels(&self) -> ChannelManager {
        ChannelManager::new(self.cache(), self.http.clone())
    }
}
