//! Chat message entity for the market bounded context.
//!
//! Chat messages sent by users.

use serde::{Deserialize, Serialize};

/// A chat message sent by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique message identifier
    pub id: String,
    /// ID of the user who sent the message
    pub user_id: u64,
    /// Display name of the user
    pub username: String,
    /// Message content
    pub message: String,
    /// Unix timestamp when message was sent
    pub timestamp: i64,
}

impl ChatMessage {
    /// Create a new chat message.
    pub fn new(user_id: u64, username: String, message: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}
