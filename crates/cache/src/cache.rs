use dashmap::DashMap;
use discord_rs_model::{User, Guild, Channel, Role, Member, Snowflake};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Cache {
    pub users: DashMap<Snowflake, Arc<User>>,
    pub guilds: DashMap<Snowflake, Arc<Guild>>,
    pub channels: DashMap<Snowflake, Arc<Channel>>,
    pub members: DashMap<Snowflake, DashMap<Snowflake, Arc<Member>>>,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_user(&self, user: User) -> Arc<User> {
        let id = user.id;
        let arc = Arc::new(user);
        self.users.insert(id, arc.clone());
        arc
    }

    pub fn update_guild(&self, guild: Guild) -> Arc<Guild> {
        let id = guild.id;
        let arc = Arc::new(guild);
        self.guilds.insert(id, arc.clone());
        arc
    }

    pub fn update_channel(&self, channel: Channel) -> Arc<Channel> {
        let id = channel.id;
        let arc = Arc::new(channel);
        self.channels.insert(id, arc.clone());
        arc
    }

    pub fn update_member(&self, guild_id: Snowflake, member: Member) -> Arc<Member> {
        let user_id = member.user.as_ref().map(|u| u.id).expect("Member must have user for caching");
        let arc = Arc::new(member);
        let guild_members = self.members.entry(guild_id).or_insert_with(DashMap::new);
        guild_members.insert(user_id, arc.clone());
        arc
    }
}
