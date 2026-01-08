use crate::snowflake::Snowflake;

#[derive(Debug, Clone)]
pub struct Config {
    pub token: String,
    pub application_id: Option<Snowflake>,
}

impl Config {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            application_id: None,
        }
    }
}
