pub mod client;
pub use client::Client;

// Re-exports
pub use discord_rs_core as core;
pub use discord_rs_model as model;
pub use discord_rs_gateway as gateway;
pub use discord_rs_http as http;
pub use discord_rs_cache as cache;
pub use discord_rs_builders as builders;
pub use discord_rs_voice as voice;
pub use discord_rs_sharding as sharding;
