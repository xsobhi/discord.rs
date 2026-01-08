use serde::{Deserialize, Serialize};
use crate::emoji::Emoji;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub count: i32,
    pub me: bool,
    pub emoji: Emoji,
}
