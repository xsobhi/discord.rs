pub mod config;
pub mod error;
pub mod intents;
pub mod snowflake;
pub mod context;
pub mod traits;

pub use config::Config;
pub use error::DiscordError;
pub use intents::Intents;
pub use snowflake::Snowflake;
pub use context::Context;

// Common result type
pub type Result<T> = std::result::Result<T, DiscordError>;