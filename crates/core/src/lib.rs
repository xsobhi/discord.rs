pub mod config;
pub mod error;
pub mod intents;
pub mod snowflake;

pub use config::Config;
pub use error::DiscordError;
pub use intents::Intents;
pub use snowflake::Snowflake;

// Common result type
pub type Result<T> = std::result::Result<T, DiscordError>;
