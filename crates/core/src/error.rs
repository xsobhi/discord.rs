use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiscordError {
    #[error("Gateway error: {0}")]
    Gateway(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Sharding error: {0}")]
    Sharding(String),
}
