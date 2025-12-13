//! Chat handlers for WebSocket connections.
//!
//! Handles chat message sending and validation.

use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;

use crate::api::ws::AppState;
use crate::domain::constants::chat::MAX_MESSAGE_LENGTH;
use crate::domain::error::UserError;
use crate::domain::models::ChatMessage;
use crate::presentation::websocket::messages::ServerMessage;

use super::send_message;

/// Handle chat message sending
pub async fn handle_chat(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    message: String,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
            send_message(sender, &msg).await;
            return;
        }
    };

    // Validate message
    let message = message.trim();
    if message.is_empty() {
        return;
    }
    if message.len() > MAX_MESSAGE_LENGTH {
        let msg = ServerMessage::error(
            "MESSAGE_TOO_LONG",
            &format!("Message must be under {} characters", MAX_MESSAGE_LENGTH),
        );
        send_message(sender, &msg).await;
        return;
    }

    if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
        if !user.chat_enabled {
            let msg = ServerMessage::from_user_error(UserError::ChatDisabled { user_id: uid });
            send_message(sender, &msg).await;
            return;
        }

        let chat_msg = ChatMessage::new(uid, user.name.clone(), message.to_string());

        // Log chat message (if chat logging is enabled)
        state.event_log.log_chat_message(uid, &user.name, message);

        state.chat.broadcast_message(chat_msg);
    }
}
