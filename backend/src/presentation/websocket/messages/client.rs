//! Client-to-server WebSocket messages.
//!
//! This module defines all messages that clients can send to the server.
//! Messages are tagged with a "type" field for JSON serialization.

#![allow(dead_code)] // Message utility methods for logging and validation

use serde::Deserialize;

/// Messages from client to server.
///
/// Each variant represents a different action the client can request.
/// The message is serialized as JSON with a "type" tag and optional "payload".
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    // =========================================================================
    // AUTHENTICATION
    // =========================================================================
    /// Authenticate with user ID token (for reconnection/stored sessions)
    Auth { token: String },

    /// Login with registration number and password
    Login { regno: String, password: String },

    /// Register a new user
    Register {
        regno: String,
        name: String,
        password: String,
    },

    // =========================================================================
    // TRADING
    // =========================================================================
    /// Place a trading order
    PlaceOrder {
        symbol: String,
        /// Order side: "Buy", "Sell", "Short"
        side: String,
        /// Order type: "Market", "Limit"
        order_type: String,
        /// Time in force: "GTC" (default), "IOC"
        time_in_force: Option<String>,
        qty: u64,
        /// Price in scaled format (multiply by PRICE_SCALE)
        price: i64,
    },

    /// Cancel an existing order
    CancelOrder { symbol: String, order_id: u64 },

    // =========================================================================
    // MARKET DATA
    // =========================================================================
    /// Subscribe to market data for a symbol
    Subscribe { symbol: String },

    /// Request order book depth
    GetDepth {
        symbol: String,
        levels: Option<usize>,
    },

    // =========================================================================
    // PORTFOLIO & HISTORY
    // =========================================================================
    /// Request current portfolio
    GetPortfolio,

    /// Get user's trade history
    GetTradeHistory {
        page: Option<u32>,
        page_size: Option<u32>,
        /// Filter by symbol
        symbol: Option<String>,
    },

    /// Get stock trade history (for orderbook tab)
    GetStockTrades {
        symbol: String,
        count: Option<usize>,
    },

    // =========================================================================
    // SYNC & STATE
    // =========================================================================
    /// Request full state sync (on connect/reconnect)
    RequestSync {
        /// Optional component name for partial sync, None for full sync
        /// Components: "portfolio", "orders", "leaderboard", "indices",
        /// "news", "chat", "orderbook:{symbol}", "candles:{symbol}",
        /// "stock_trades:{symbol}", "trade_history"
        component: Option<String>,
    },

    /// Get public config (registration mode, etc.)
    GetConfig {},

    // =========================================================================
    // SOCIAL
    // =========================================================================
    /// Send chat message
    Chat { message: String },

    // =========================================================================
    // ADMIN
    // =========================================================================
    /// Admin actions (requires admin privileges)
    ///
    /// Supported actions:
    /// - "ToggleMarket" - payload: { open: bool }
    /// - "SetVolatility" - payload: { symbol: string, volatility: i64 }
    /// - "CreateCompany" - payload: { symbol, name, sector, volatility }
    /// - "InitGame" - payload: { starting_cash?, shares_per_trader? }
    /// - "SetBankrupt" - payload: { symbol: string }
    /// - "BanTrader" - payload: { user_id: u64, banned: bool }
    /// - "MuteTrader" - payload: { user_id: u64, muted: bool }
    /// - "GetAllTrades" - payload: { user_id?, symbol?, page?, page_size? }
    /// - "GetAllOpenOrders" - payload: { symbol? }
    /// - "GetOrderbook" - payload: { symbol: string }
    /// - "GetDashboardMetrics" - no payload
    AdminAction {
        action: String,
        payload: serde_json::Value,
    },

    // =========================================================================
    // SYSTEM
    // =========================================================================
    /// Ping for keepalive
    Ping {},
}

impl ClientMessage {
    /// Get the message type name for logging/debugging
    pub fn message_type(&self) -> &'static str {
        match self {
            ClientMessage::Auth { .. } => "Auth",
            ClientMessage::Login { .. } => "Login",
            ClientMessage::Register { .. } => "Register",
            ClientMessage::PlaceOrder { .. } => "PlaceOrder",
            ClientMessage::CancelOrder { .. } => "CancelOrder",
            ClientMessage::Subscribe { .. } => "Subscribe",
            ClientMessage::GetDepth { .. } => "GetDepth",
            ClientMessage::GetPortfolio => "GetPortfolio",
            ClientMessage::GetTradeHistory { .. } => "GetTradeHistory",
            ClientMessage::GetStockTrades { .. } => "GetStockTrades",
            ClientMessage::RequestSync { .. } => "RequestSync",
            ClientMessage::GetConfig { .. } => "GetConfig",
            ClientMessage::Chat { .. } => "Chat",
            ClientMessage::AdminAction { .. } => "AdminAction",
            ClientMessage::Ping { .. } => "Ping",
        }
    }

    /// Check if this message requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // These don't require auth
            ClientMessage::Auth { .. }
            | ClientMessage::Login { .. }
            | ClientMessage::Register { .. }
            | ClientMessage::GetConfig { .. }
            | ClientMessage::Ping { .. } => false,
            // Everything else requires auth
            _ => true,
        }
    }

    /// Check if this is an admin-only action
    pub fn is_admin_action(&self) -> bool {
        matches!(self, ClientMessage::AdminAction { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_auth() {
        let json = r#"{"type": "Auth", "payload": {"token": "123"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Auth { token } if token == "123"));
    }

    #[test]
    fn test_deserialize_place_order() {
        let json = r#"{
            "type": "PlaceOrder",
            "payload": {
                "symbol": "AAPL",
                "side": "Buy",
                "order_type": "Limit",
                "qty": 100,
                "price": 1500000
            }
        }"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::PlaceOrder { symbol, .. } if symbol == "AAPL"));
    }

    #[test]
    fn test_deserialize_ping() {
        let json = r#"{"type": "Ping", "payload": {}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping {}));
    }

    #[test]
    fn test_requires_auth() {
        let auth = ClientMessage::Auth {
            token: "123".to_string(),
        };
        assert!(!auth.requires_auth());

        let order = ClientMessage::PlaceOrder {
            symbol: "AAPL".to_string(),
            side: "Buy".to_string(),
            order_type: "Limit".to_string(),
            time_in_force: None,
            qty: 100,
            price: 1500000,
        };
        assert!(order.requires_auth());
    }

    #[test]
    fn test_message_type() {
        let msg = ClientMessage::GetPortfolio;
        assert_eq!(msg.message_type(), "GetPortfolio");
    }
}
