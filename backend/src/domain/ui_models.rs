//! UI-ready data transfer objects for frontend communication.
//!
//! These structs are serialized to JSON and sent to the frontend via WebSocket.
//! The `dead_code` lint is suppressed because Serde serialization uses all fields
//! at runtime, which the compiler cannot detect statically.

#![allow(dead_code)] // Serde serialization uses these at runtime

use crate::domain::models::{
    ChatMessage, OrderId, OrderSide, OrderStatus, OrderType, Price, Quantity, TimeInForce, TradeId,
    UserId,
};
use serde::{Deserialize, Serialize};

// --- UI-Ready Portfolio Item ---
// Pre-computed values for frontend display, no calculations needed on client

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItemUI {
    pub symbol: String,
    pub qty: Quantity,
    pub short_qty: Quantity,
    pub locked_qty: Quantity,
    pub average_buy_price: Price,
    pub current_price: Price,
    pub market_value: Price,         // qty * current_price
    pub cost_basis: Price,           // qty * average_buy_price
    pub unrealized_pnl: Price,       // market_value - cost_basis
    pub unrealized_pnl_percent: f64, // ((market_value - cost_basis) / cost_basis) * 100
    // Short position info
    pub short_market_value: Price, // short_qty * current_price (liability)
    pub short_unrealized_pnl: Price, // For shorts: positive if price went down
}

// --- UI-Ready Open Order ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrderUI {
    pub order_id: OrderId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub qty: Quantity,
    pub filled_qty: Quantity,
    pub remaining_qty: Quantity, // qty - filled_qty (pre-computed)
    pub price: Price,
    pub status: OrderStatus,
    pub timestamp: i64,
    pub time_in_force: TimeInForce,
}

// --- UI-Ready Market Index ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndexUI {
    pub name: String,
    pub value: Price,
    pub previous_value: Price,
    pub change: Price,       // value - previous_value
    pub change_percent: f64, // ((value - previous_value) / previous_value) * 100
    pub timestamp: i64,
}

// --- UI-Ready Leaderboard Entry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntryUI {
    pub rank: usize,
    pub user_id: UserId, // For admin linking
    pub name: String,
    pub net_worth: Price, // CORRECT: money + locked + margin + portfolio_value
    pub change_rank: i32, // Positive = moved up, negative = moved down
}

// --- Trade History Item ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistoryItem {
    pub trade_id: TradeId,
    pub symbol: String,
    pub side: String, // "Buy", "Sell", "Short", "Cover"
    pub qty: Quantity,
    pub price: Price,
    pub total_value: Price, // qty * price
    pub counterparty_id: Option<UserId>,
    pub counterparty_name: Option<String>, // For admin view
    pub timestamp: i64,
}

// --- UI-Ready Company Info ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUI {
    pub id: u64,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub current_price: Option<Price>, // Last traded price
    pub price_change: Option<Price>,  // Change from open
    pub price_change_percent: Option<f64>,
    pub volume: Quantity, // Today's volume
    pub bankrupt: bool,
}

// --- UI-Ready News Item ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItemUI {
    pub id: String,
    pub headline: String,
    pub symbol: Option<String>,
    pub sentiment: String, // "positive", "negative", "neutral"
    pub impact: String,    // "high", "medium", "low"
    pub timestamp: i64,
}

// --- UI-Ready Orderbook ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookUI {
    pub symbol: String,
    pub bids: Vec<OrderbookLevelUI>,
    pub asks: Vec<OrderbookLevelUI>,
    pub spread: Option<Price>,
    pub spread_percent: Option<f64>,
    pub last_price: Option<Price>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevelUI {
    pub price: Price,
    pub qty: Quantity,
    pub order_count: usize,       // Number of orders at this level
    pub cumulative_qty: Quantity, // Cumulative quantity up to this level
}

// --- Full Portfolio State ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStateUI {
    pub money: Price,
    pub locked_money: Price,
    pub margin_locked: Price,
    pub total_available: Price, // money (available for new orders)
    pub portfolio_value: Price, // Sum of all position market values
    pub net_worth: Price,       // money + locked + margin + portfolio_value
    pub items: Vec<PortfolioItemUI>,
}

// --- Full State Sync Message Payload ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullStateSyncPayload {
    // Market state
    pub market_open: bool,
    pub halted_symbols: Vec<(String, i64)>, // (symbol, halted_until_timestamp)

    // Companies with current prices
    pub companies: Vec<CompanyUI>,

    // User state (None if not authenticated)
    pub portfolio: Option<PortfolioStateUI>,
    pub open_orders: Vec<OpenOrderUI>,

    // Market data
    pub indices: Vec<MarketIndexUI>,
    pub leaderboard: Vec<LeaderboardEntryUI>,
    pub news: Vec<NewsItemUI>,
    pub chat_history: Vec<ChatMessage>,

    // Currently viewed symbol data
    pub active_symbol: Option<String>,
    pub orderbook: Option<OrderbookUI>,
    pub candles: Option<Vec<CandleUI>>,
    pub recent_trades: Vec<TradeHistoryItem>, // Recent trades for active symbol

    // Sync metadata
    pub sync_id: u64,
    pub timestamp: i64,
}

// --- UI-Ready Candle ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleUI {
    pub timestamp: i64,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
}

// --- Trade History Response ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistoryResponse {
    pub trades: Vec<TradeHistoryItem>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

// --- Admin Open Orders Response ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOpenOrdersResponse {
    pub orders: Vec<OpenOrderUI>,
    pub total_count: usize,
}

// --- Admin Open Order (with user info) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOpenOrderUI {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub user_name: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub qty: Quantity,
    pub filled_qty: Quantity,
    pub remaining_qty: Quantity,
    pub price: Price,
    pub status: OrderStatus,
    pub timestamp: i64,
    pub time_in_force: TimeInForce,
}

// --- Admin Trade History Item (with both parties visible) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminTradeHistoryItem {
    pub trade_id: TradeId,
    pub symbol: String,
    pub buyer_id: UserId,
    pub buyer_name: String,
    pub buyer_side: String, // "Buy" or "Cover"
    pub seller_id: UserId,
    pub seller_name: String,
    pub seller_side: String, // "Sell" or "Short"
    pub qty: Quantity,
    pub price: Price,
    pub total_value: Price,
    pub timestamp: i64,
}

// --- Admin Dashboard Metrics ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDashboardMetrics {
    pub total_traders: usize,
    pub active_traders: usize, // Currently connected
    pub total_trades: u64,
    pub total_volume: Price,
    pub recent_volume: Price, // Last 5 minutes
    pub total_market_cap: Price,
    pub halted_symbols_count: usize,
    pub open_orders_count: usize,
    pub market_open: bool,
    pub timestamp: i64,
    // Server/system metrics
    pub server_uptime_secs: u64,
    pub active_sessions: Vec<ActiveSessionInfo>,
}

// --- Active Session Info ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSessionInfo {
    pub session_id: u64,
    pub user_id: UserId,
    pub user_name: String,
    pub connected_at: i64,  // Unix timestamp
    pub last_activity: i64, // Unix timestamp
    pub messages_sent: u64,
}

// --- Component Sync Responses ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSyncResponse {
    pub sync_id: u64,
    pub money: Price,
    pub locked_money: Price,
    pub margin_locked: Price,
    pub portfolio_value: Price,
    pub net_worth: Price,
    pub items: Vec<PortfolioItemUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrdersSyncResponse {
    pub sync_id: u64,
    pub orders: Vec<OpenOrderUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardSyncResponse {
    pub sync_id: u64,
    pub entries: Vec<LeaderboardEntryUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicesSyncResponse {
    pub sync_id: u64,
    pub indices: Vec<MarketIndexUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSyncResponse {
    pub sync_id: u64,
    pub symbol: String,
    pub orderbook: OrderbookUI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlesSyncResponse {
    pub sync_id: u64,
    pub symbol: String,
    pub candles: Vec<CandleUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSyncResponse {
    pub sync_id: u64,
    pub news: Vec<NewsItemUI>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSyncResponse {
    pub sync_id: u64,
    pub messages: Vec<ChatMessage>,
}

// Note: Sync ID generation has been consolidated into infrastructure/id_generator.rs
// Use IdGenerators::global().next_sync_id() instead
