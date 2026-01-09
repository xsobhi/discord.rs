pub mod client;
pub mod collector;

pub use client::Client;
pub use collector::{Collector, CollectorBuilder};

// Ergonomic Re-exports
pub use discord_rs_core::{Intents, Snowflake, Context};
pub use discord_rs_model::{User, Message, Guild, Channel, Role, Member, Interaction, Event};
pub use discord_rs_builders::{MessageBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, SelectMenuBuilder, InteractionResponseBuilder};
pub use discord_rs_cache::{Cache, ContextCacheExt};

// Internal crates re-exports for advanced users
pub mod core { pub use discord_rs_core::*; }
pub mod model { pub use discord_rs_model::*; }
pub mod gateway { pub use discord_rs_gateway::*; }
pub mod http { pub use discord_rs_http::*; }
pub mod cache { pub use discord_rs_cache::*; }
pub mod builders { pub use discord_rs_builders::*; }
pub mod sharding { pub use discord_rs_sharding::*; }
