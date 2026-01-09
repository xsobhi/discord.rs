use discord_rs_core::DiscordError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DiscordApiError {
    pub code: u64,
    pub message: String,
    pub errors: Option<serde_json::Value>,
}

impl std::fmt::Display for DiscordApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Discord API Error {}: {}", self.code, self.message)?;
        if let Some(errors) = &self.errors {
            write!(f, " | Details: {:?}", errors)?;
        }
        Ok(())
    }
}
