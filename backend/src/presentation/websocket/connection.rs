//! WebSocket connection management.
//!
//! Handles the lifecycle of WebSocket connections including:
//! - Connection state tracking
//! - Broadcast channel subscriptions
//! - Message sending utilities
//! - Connection cleanup

#![allow(dead_code)] // Connection state helpers for subscription management

use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::SplitSink};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::domain::models::{Candle, ChatMessage, Trade};
use crate::domain::ui_models::{LeaderboardEntryUI, MarketIndexUI};
use crate::service::news::NewsItem;

use super::messages::ServerMessage;
use super::state::AppState;

/// Represents an active WebSocket connection's state
pub struct ConnectionState {
    /// The authenticated user's ID (if any)
    pub user_id: Option<u64>,
    /// The session ID for this connection
    pub session_id: Option<u64>,
    /// Symbols the connection is subscribed to
    pub subscribed_symbols: Vec<String>,
}

impl ConnectionState {
    /// Create a new unauthenticated connection state
    pub fn new() -> Self {
        Self {
            user_id: None,
            session_id: None,
            subscribed_symbols: Vec::new(),
        }
    }

    /// Check if the connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Set the authenticated user
    pub fn set_authenticated(&mut self, user_id: u64, session_id: u64) {
        self.user_id = Some(user_id);
        self.session_id = Some(session_id);
    }

    /// Add a symbol subscription
    pub fn subscribe(&mut self, symbol: String) {
        if !self.subscribed_symbols.contains(&symbol) {
            self.subscribed_symbols.push(symbol);
        }
    }

    /// Remove a symbol subscription
    pub fn unsubscribe(&mut self, symbol: &str) {
        self.subscribed_symbols.retain(|s| s != symbol);
    }

    /// Check if subscribed to a symbol
    pub fn is_subscribed(&self, symbol: &str) -> bool {
        self.subscribed_symbols.iter().any(|s| s == symbol)
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Broadcast channel subscriptions for a connection
pub struct BroadcastSubscriptions {
    pub trades: broadcast::Receiver<Trade>,
    pub candles: broadcast::Receiver<Candle>,
    pub circuit_breakers: broadcast::Receiver<(String, i64)>,
    pub indices: broadcast::Receiver<MarketIndexUI>,
    pub news: broadcast::Receiver<NewsItem>,
    pub leaderboard: broadcast::Receiver<Vec<LeaderboardEntryUI>>,
    pub chat: broadcast::Receiver<ChatMessage>,
}

impl BroadcastSubscriptions {
    /// Subscribe to all broadcast channels from the app state
    pub fn from_state(state: &Arc<AppState>) -> Self {
        Self {
            trades: state.engine.subscribe_trades(),
            candles: state.market.subscribe_candles(),
            circuit_breakers: state.market.subscribe_circuit_breakers(),
            indices: state.indices.subscribe_indices(),
            news: state.news.subscribe(),
            leaderboard: state.leaderboard.subscribe(),
            chat: state.chat.subscribe(),
        }
    }
}

/// Send a message to the WebSocket client
pub async fn send_message(sender: &mut SplitSink<WebSocket, Message>, msg: &ServerMessage) {
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

/// Cleanup resources when a connection closes
pub fn cleanup_connection(state: &Arc<AppState>, conn_state: &ConnectionState) {
    if let Some(sid) = conn_state.session_id {
        state.sessions.remove_session(sid);
        info!(
            "Session {} cleaned up for user {:?}",
            sid, conn_state.user_id
        );
    }
}

/// Handle a trade broadcast and send appropriate messages
pub async fn handle_trade_broadcast(
    sender: &mut SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    conn_state: &ConnectionState,
    trade: Trade,
) {
    // Send trade update
    let msg = ServerMessage::TradeUpdate {
        symbol: trade.symbol.clone(),
        price: trade.price,
        qty: trade.qty,
        timestamp: trade.timestamp,
    };
    send_message(sender, &msg).await;

    // Send updated depth if user is subscribed to this symbol
    if conn_state.is_subscribed(&trade.symbol) {
        if let Some((bids, asks)) = state.engine.get_order_book_depth(&trade.symbol, 10) {
            let spread = match (bids.first(), asks.first()) {
                (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                _ => None,
            };
            let depth_msg = ServerMessage::DepthUpdate {
                symbol: trade.symbol.clone(),
                bids,
                asks,
                spread,
            };
            send_message(sender, &depth_msg).await;
        }
    }

    // If this user was involved in the trade, send portfolio update
    if let Some(uid) = conn_state.user_id {
        if trade.maker_user_id == uid || trade.taker_user_id == uid {
            if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                let net_worth =
                    crate::presentation::websocket::handlers::helpers::calculate_net_worth(
                        &user,
                        &state.market,
                    );
                let portfolio_msg = ServerMessage::PortfolioUpdate {
                    money: user.money,
                    locked: user.locked_money,
                    margin_locked: user.margin_locked,
                    net_worth,
                    items: user.portfolio,
                };
                send_message(sender, &portfolio_msg).await;
            }
        }
    }
}

/// Handle a candle broadcast
pub async fn handle_candle_broadcast(sender: &mut SplitSink<WebSocket, Message>, candle: Candle) {
    let msg = ServerMessage::CandleUpdate {
        symbol: candle.symbol.clone(),
        candle,
    };
    send_message(sender, &msg).await;
}

/// Handle a circuit breaker broadcast
pub async fn handle_circuit_breaker_broadcast(
    sender: &mut SplitSink<WebSocket, Message>,
    symbol: String,
    halted_until: i64,
) {
    let msg = ServerMessage::CircuitBreaker {
        symbol,
        halted_until,
        reason: "10% price movement threshold exceeded".to_string(),
    };
    send_message(sender, &msg).await;
}

/// Handle an index update broadcast
pub async fn handle_index_broadcast(
    sender: &mut SplitSink<WebSocket, Message>,
    index: MarketIndexUI,
) {
    // Send both old format (for backward compatibility) and new UI format
    let msg = ServerMessage::IndexUpdate {
        name: index.name.clone(),
        value: index.value,
    };
    send_message(sender, &msg).await;

    // Also send UI-ready format
    let ui_msg = ServerMessage::IndexUpdateUI { index };
    send_message(sender, &ui_msg).await;
}

/// Handle a news broadcast
pub async fn handle_news_broadcast(sender: &mut SplitSink<WebSocket, Message>, news: NewsItem) {
    let msg = ServerMessage::NewsUpdate { news };
    send_message(sender, &msg).await;
}

/// Handle a leaderboard broadcast
pub async fn handle_leaderboard_broadcast(
    sender: &mut SplitSink<WebSocket, Message>,
    entries: Vec<LeaderboardEntryUI>,
) {
    let msg = ServerMessage::LeaderboardUpdateUI { entries };
    send_message(sender, &msg).await;
}

/// Handle a chat message broadcast
pub async fn handle_chat_broadcast(
    sender: &mut SplitSink<WebSocket, Message>,
    message: ChatMessage,
) {
    let msg = ServerMessage::ChatUpdate { message };
    send_message(sender, &msg).await;
}

/// Log connection established
pub fn log_connection_established() {
    info!("New WebSocket connection established");
}

/// Log connection closed
pub fn log_connection_closed(conn_state: &ConnectionState) {
    info!(
        "Client disconnected (user_id={:?}, session_id={:?})",
        conn_state.user_id, conn_state.session_id
    );
}

/// Log connection error
pub fn log_connection_error(conn_state: &ConnectionState, error: &str) {
    error!(
        "WebSocket error: {} (user_id={:?})",
        error, conn_state.user_id
    );
}

/// Send initial configuration to a new connection
pub async fn send_initial_config(
    sender: &mut SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
) {
    use super::messages::CurrencyConfigPayload;

    let public_config = state.config.get_public_config();
    let config_msg = ServerMessage::Config {
        registration_mode: format!("{:?}", public_config.registration_mode),
        chat_enabled: public_config.chat_enabled,
        currency: CurrencyConfigPayload::from(&public_config.currency),
    };
    send_message(sender, &config_msg).await;
    debug!("Sent initial config to client");

    // Send frontend constants
    let constants = state.config.get_frontend_constants();
    let constants_msg = ServerMessage::FrontendConstants { constants };
    send_message(sender, &constants_msg).await;
    debug!("Sent frontend constants to client");
}
