//! WebSocket message handlers.
//!
//! This module contains all the handlers for different types of WebSocket messages.
//! Each handler is responsible for processing a specific category of messages.

pub mod admin;
pub mod auth;
pub mod chat;
pub mod market;
pub mod sync;
pub mod trading;

// Re-export commonly used items
pub use admin::handle_admin_action;
pub use auth::{handle_auth, handle_login, handle_register};
pub use chat::handle_chat;
pub use market::{
    handle_get_depth, handle_get_stock_trades, handle_get_trade_history, handle_subscribe,
};
pub use sync::handle_request_sync;
pub use trading::{handle_cancel_order, handle_get_portfolio, handle_place_order};

use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use tracing::error;

use crate::presentation::websocket::messages::ServerMessage;

/// Send a message to the WebSocket client
pub async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &ServerMessage,
) {
    match serde_json::to_string(msg) {
        Ok(json) => {
            if let Err(e) = sender.send(Message::Text(json)).await {
                error!("Failed to send WebSocket message: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to serialize message: {}", e);
        }
    }
}

/// Helper module for common calculations
pub mod helpers {
    use crate::domain::models::User;
    use crate::service::market::MarketService;

    /// Calculate user's net worth including portfolio value
    pub fn calculate_net_worth(user: &User, market: &MarketService) -> i64 {
        let mut portfolio_value: i64 = 0;

        for item in &user.portfolio {
            let current_price = market
                .get_last_price(&item.symbol)
                .unwrap_or(item.average_buy_price);

            // Long positions add value
            portfolio_value += (item.qty as i64) * current_price;

            // Short positions subtract value (liability)
            portfolio_value -= (item.short_qty as i64) * current_price;
        }

        user.money + user.locked_money + user.margin_locked + portfolio_value
    }
}
