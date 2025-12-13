//! Server-to-client WebSocket messages.
//!
//! This module defines all messages that the server can send to clients.
//! Messages are tagged with a "type" field for JSON serialization.

#![allow(dead_code)] // Message utility methods for type introspection

use crate::domain::error::{DomainError, ErrorResponse, MarketError, TradingError, UserError};
use crate::domain::models::{Candle, ChatMessage, Portfolio};
use crate::domain::ui_models::{
    AdminDashboardMetrics, AdminOpenOrderUI, AdminTradeHistoryItem, CandleUI, FullStateSyncPayload,
    LeaderboardEntryUI, MarketIndexUI, NewsItemUI, OpenOrderUI, OrderbookUI, PortfolioItemUI,
    TradeHistoryItem,
};
use crate::service::leaderboard::LeaderboardEntry;
use crate::service::news::NewsItem;
use serde::Serialize;

/// Company info for the company list
#[derive(Debug, Serialize, Clone)]
pub struct CompanyInfo {
    pub id: u64,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub volatility: i64,
}

/// Currency configuration payload for client
#[derive(Debug, Serialize, Clone)]
pub struct CurrencyConfigPayload {
    pub symbol: String,
    pub code: String,
    pub locale: String,
    pub decimals: u8,
    pub symbol_position: String,
}

impl From<&crate::config::CurrencyConfig> for CurrencyConfigPayload {
    fn from(config: &crate::config::CurrencyConfig) -> Self {
        Self {
            symbol: config.symbol.clone(),
            code: config.code.clone(),
            locale: config.locale.clone(),
            decimals: config.decimals,
            symbol_position: config.symbol_position.clone(),
        }
    }
}

/// Messages from server to client.
///
/// Each variant represents a different type of update or response.
/// Messages are serialized as JSON with a "type" tag and "payload".
///
/// Note: Some variants are defined for API completeness but may not be
/// actively sent yet. The frontend expects all these message types.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
#[allow(dead_code)] // Variants are part of the API contract with frontend
pub enum ServerMessage {
    // =========================================================================
    // AUTHENTICATION RESPONSES
    // =========================================================================
    /// Authentication successful
    AuthSuccess {
        user_id: u64,
        name: String,
        /// User role: "admin" or "trader"
        role: String,
        /// Secure authentication token for reconnection
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
    },

    /// Authentication failed
    AuthFailed { reason: String },

    /// Registration successful
    RegisterSuccess {
        user_id: u64,
        name: String,
        /// User role: "admin" or "trader"
        role: String,
        /// Secure authentication token for reconnection
        token: String,
    },

    /// Registration failed
    RegisterFailed { reason: String },

    /// Session kicked (another session took over)
    SessionKicked { reason: String },

    // =========================================================================
    // ORDER RESPONSES
    // =========================================================================
    /// Order acknowledged
    OrderAck {
        order_id: u64,
        status: String,
        filled_qty: u64,
        remaining_qty: u64,
    },

    /// Order rejected
    OrderRejected {
        reason: String,
        /// Error code for frontend handling
        error_code: String,
    },

    /// Order cancelled
    OrderCancelled { order_id: u64 },

    // =========================================================================
    // REAL-TIME MARKET UPDATES
    // =========================================================================
    /// Trade executed
    TradeUpdate {
        symbol: String,
        price: i64,
        qty: u64,
        timestamp: i64,
    },

    /// Candlestick update
    CandleUpdate { symbol: String, candle: Candle },

    /// Order book depth update
    DepthUpdate {
        symbol: String,
        bids: Vec<(i64, u64)>, // (price, qty)
        asks: Vec<(i64, u64)>,
        spread: Option<i64>,
    },

    /// Market index update (legacy format)
    IndexUpdate { name: String, value: i64 },

    /// UI-ready index update with change data
    IndexUpdateUI { index: MarketIndexUI },

    // =========================================================================
    // PORTFOLIO UPDATES
    // =========================================================================
    /// Portfolio update (legacy format)
    PortfolioUpdate {
        money: i64,
        locked: i64,
        margin_locked: i64,
        /// Calculated total value
        net_worth: i64,
        items: Vec<Portfolio>,
    },

    /// Enhanced portfolio with pre-computed values
    PortfolioUpdateUI {
        money: i64,
        locked_money: i64,
        margin_locked: i64,
        portfolio_value: i64,
        net_worth: i64,
        items: Vec<PortfolioItemUI>,
    },

    /// Open orders list update
    OpenOrdersUpdate { orders: Vec<OpenOrderUI> },

    // =========================================================================
    // MARKET EVENTS
    // =========================================================================
    /// Circuit breaker triggered
    CircuitBreaker {
        symbol: String,
        halted_until: i64,
        reason: String,
    },

    /// Market status changed
    MarketStatus { is_open: bool },

    // =========================================================================
    // SOCIAL & NEWS
    // =========================================================================
    /// News item
    NewsUpdate { news: NewsItem },

    /// Leaderboard update (legacy)
    LeaderboardUpdate { entries: Vec<LeaderboardEntry> },

    /// UI-ready leaderboard update
    LeaderboardUpdateUI { entries: Vec<LeaderboardEntryUI> },

    /// Chat message
    ChatUpdate { message: ChatMessage },

    // =========================================================================
    // CONFIGURATION & SYSTEM
    // =========================================================================
    /// List of all tradeable companies
    CompanyList { companies: Vec<CompanyInfo> },

    /// Public config for initialization
    Config {
        registration_mode: String,
        chat_enabled: bool,
        currency: CurrencyConfigPayload,
    },

    /// Frontend constants for UI configuration
    FrontendConstants {
        constants: crate::config::FrontendConstants,
    },

    /// General error
    Error { code: String, message: String },

    /// Pong response
    Pong { timestamp: i64 },

    /// System announcement
    System { message: String },

    // =========================================================================
    // FULL STATE SYNC
    // =========================================================================
    /// Full state sync - sent on connect/reconnect
    FullStateSync { payload: FullStateSyncPayload },

    // =========================================================================
    // COMPONENT SYNC RESPONSES
    // =========================================================================
    /// Portfolio sync response
    PortfolioSync {
        sync_id: u64,
        money: i64,
        locked_money: i64,
        margin_locked: i64,
        portfolio_value: i64,
        net_worth: i64,
        items: Vec<PortfolioItemUI>,
    },

    /// Open orders sync response
    OpenOrdersSync {
        sync_id: u64,
        orders: Vec<OpenOrderUI>,
    },

    /// Leaderboard sync response
    LeaderboardSync {
        sync_id: u64,
        entries: Vec<LeaderboardEntryUI>,
    },

    /// Indices sync response
    IndicesSync {
        sync_id: u64,
        indices: Vec<MarketIndexUI>,
    },

    /// Orderbook sync response
    OrderbookSync {
        sync_id: u64,
        symbol: String,
        orderbook: OrderbookUI,
    },

    /// Candles sync response
    CandlesSync {
        sync_id: u64,
        symbol: String,
        candles: Vec<CandleUI>,
    },

    /// News sync response
    NewsSync { sync_id: u64, news: Vec<NewsItemUI> },

    /// Chat sync response
    ChatSync {
        sync_id: u64,
        messages: Vec<ChatMessage>,
    },

    // =========================================================================
    // TRADE HISTORY
    // =========================================================================
    /// Trade history response (for user's trades)
    TradeHistory {
        trades: Vec<TradeHistoryItem>,
        total_count: u64,
        page: u32,
        page_size: u32,
        has_more: bool,
    },

    /// Stock trade history response (for orderbook tab)
    StockTradeHistory {
        symbol: String,
        trades: Vec<TradeHistoryItem>,
    },

    // =========================================================================
    // ADMIN RESPONSES
    // =========================================================================
    /// Admin: All trades with filters (enhanced with both parties)
    AdminTradeHistory {
        trades: Vec<AdminTradeHistoryItem>,
        total_count: u64,
        page: u32,
        page_size: u32,
        has_more: bool,
    },

    /// Admin: All open orders (enhanced with user info)
    AdminOpenOrders {
        orders: Vec<AdminOpenOrderUI>,
        total_count: usize,
    },

    /// Admin: Dashboard metrics
    AdminDashboardMetrics { metrics: AdminDashboardMetrics },

    /// Admin: Orderbook view with individual orders
    AdminOrderbook {
        symbol: String,
        bids: Vec<AdminOpenOrderUI>,
        asks: Vec<AdminOpenOrderUI>,
    },
}

impl ServerMessage {
    /// Create an error message
    pub fn error(code: &str, message: &str) -> Self {
        ServerMessage::Error {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    /// Create error from TradingError
    pub fn from_trading_error(err: TradingError) -> Self {
        let response = ErrorResponse::from(err);
        ServerMessage::Error {
            code: response.code,
            message: response.message,
        }
    }

    /// Create error from UserError
    pub fn from_user_error(err: UserError) -> Self {
        let response = ErrorResponse::from(err);
        ServerMessage::Error {
            code: response.code,
            message: response.message,
        }
    }

    /// Create error from MarketError
    pub fn from_market_error(err: MarketError) -> Self {
        let response = ErrorResponse::from(err);
        ServerMessage::Error {
            code: response.code,
            message: response.message,
        }
    }

    /// Create error from DomainError
    pub fn from_domain_error(err: DomainError) -> Self {
        let response = ErrorResponse::from(err);
        ServerMessage::Error {
            code: response.code,
            message: response.message,
        }
    }

    /// Create OrderRejected from TradingError
    pub fn order_rejected_from(err: TradingError) -> Self {
        ServerMessage::OrderRejected {
            reason: err.to_string(),
            error_code: err.error_code().to_string(),
        }
    }

    /// Get the message type name for logging/debugging
    pub fn message_type(&self) -> &'static str {
        match self {
            ServerMessage::AuthSuccess { .. } => "AuthSuccess",
            ServerMessage::AuthFailed { .. } => "AuthFailed",
            ServerMessage::RegisterSuccess { .. } => "RegisterSuccess",
            ServerMessage::RegisterFailed { .. } => "RegisterFailed",
            ServerMessage::SessionKicked { .. } => "SessionKicked",
            ServerMessage::OrderAck { .. } => "OrderAck",
            ServerMessage::OrderRejected { .. } => "OrderRejected",
            ServerMessage::OrderCancelled { .. } => "OrderCancelled",
            ServerMessage::TradeUpdate { .. } => "TradeUpdate",
            ServerMessage::CandleUpdate { .. } => "CandleUpdate",
            ServerMessage::DepthUpdate { .. } => "DepthUpdate",
            ServerMessage::IndexUpdate { .. } => "IndexUpdate",
            ServerMessage::IndexUpdateUI { .. } => "IndexUpdateUI",
            ServerMessage::PortfolioUpdate { .. } => "PortfolioUpdate",
            ServerMessage::PortfolioUpdateUI { .. } => "PortfolioUpdateUI",
            ServerMessage::OpenOrdersUpdate { .. } => "OpenOrdersUpdate",
            ServerMessage::CircuitBreaker { .. } => "CircuitBreaker",
            ServerMessage::MarketStatus { .. } => "MarketStatus",
            ServerMessage::NewsUpdate { .. } => "NewsUpdate",
            ServerMessage::LeaderboardUpdate { .. } => "LeaderboardUpdate",
            ServerMessage::LeaderboardUpdateUI { .. } => "LeaderboardUpdateUI",
            ServerMessage::ChatUpdate { .. } => "ChatUpdate",
            ServerMessage::CompanyList { .. } => "CompanyList",
            ServerMessage::Config { .. } => "Config",
            ServerMessage::FrontendConstants { .. } => "FrontendConstants",
            ServerMessage::Error { .. } => "Error",
            ServerMessage::Pong { .. } => "Pong",
            ServerMessage::System { .. } => "System",
            ServerMessage::FullStateSync { .. } => "FullStateSync",
            ServerMessage::PortfolioSync { .. } => "PortfolioSync",
            ServerMessage::OpenOrdersSync { .. } => "OpenOrdersSync",
            ServerMessage::LeaderboardSync { .. } => "LeaderboardSync",
            ServerMessage::IndicesSync { .. } => "IndicesSync",
            ServerMessage::OrderbookSync { .. } => "OrderbookSync",
            ServerMessage::CandlesSync { .. } => "CandlesSync",
            ServerMessage::NewsSync { .. } => "NewsSync",
            ServerMessage::ChatSync { .. } => "ChatSync",
            ServerMessage::TradeHistory { .. } => "TradeHistory",
            ServerMessage::StockTradeHistory { .. } => "StockTradeHistory",
            ServerMessage::AdminTradeHistory { .. } => "AdminTradeHistory",
            ServerMessage::AdminOpenOrders { .. } => "AdminOpenOrders",
            ServerMessage::AdminDashboardMetrics { .. } => "AdminDashboardMetrics",
            ServerMessage::AdminOrderbook { .. } => "AdminOrderbook",
        }
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            ServerMessage::Error { .. }
                | ServerMessage::AuthFailed { .. }
                | ServerMessage::RegisterFailed { .. }
                | ServerMessage::OrderRejected { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message() {
        let msg = ServerMessage::error("TEST_ERROR", "Test message");
        match msg {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, "TEST_ERROR");
                assert_eq!(message, "Test message");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_serialize_auth_success() {
        let msg = ServerMessage::AuthSuccess {
            user_id: 123,
            name: "Test User".to_string(),
            role: "trader".to_string(),
            token: Some("abc123".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("AuthSuccess"));
        assert!(json.contains("123"));
        assert!(json.contains("Test User"));
        assert!(json.contains("trader"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_message_type() {
        let msg = ServerMessage::Pong { timestamp: 12345 };
        assert_eq!(msg.message_type(), "Pong");
    }

    #[test]
    fn test_is_error() {
        assert!(ServerMessage::error("ERR", "msg").is_error());
        assert!(ServerMessage::AuthFailed {
            reason: "x".to_string()
        }
        .is_error());
        assert!(!ServerMessage::Pong { timestamp: 0 }.is_error());
    }
}
