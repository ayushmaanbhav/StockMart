use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

// --- Type Aliases for Clarity ---
pub type UserId = u64;
pub type CompanyId = u64;
pub type OrderId = u64;
pub type TradeId = u64;
pub type Price = i64; // Scaled by 10,000 (e.g., 150.25 -> 1502500)
pub type Quantity = u64;

// --- Constants ---
pub const PRICE_SCALE: i64 = 10_000;

// --- Atomic Counters for ID Generation ---
static NEXT_USER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_COMPANY_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_ORDER_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_TRADE_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_user_id() -> UserId {
    NEXT_USER_ID.fetch_add(1, Ordering::Relaxed)
}
pub fn next_company_id() -> CompanyId {
    NEXT_COMPANY_ID.fetch_add(1, Ordering::Relaxed)
}
pub fn next_order_id() -> OrderId {
    NEXT_ORDER_ID.fetch_add(1, Ordering::Relaxed)
}
pub fn next_trade_id() -> TradeId {
    NEXT_TRADE_ID.fetch_add(1, Ordering::Relaxed)
}

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    Partial,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate or Cancel
}

// --- Structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub regno: String,
    pub name: String,
    // In a real app, this would be a hash. For this simulation, we might keep it simple or mock it.
    pub password_hash: String, 
    pub money: Price,        // Available balance
    pub locked_money: Price, // Locked in active buy orders
    pub margin_locked: Price, // Locked for short positions (150% of short value)
    pub chat_enabled: bool,
    pub banned: bool,
    pub created_at: i64, // Unix timestamp
    pub portfolio: Vec<Portfolio>,
}

impl User {
    pub fn new(regno: String, name: String, password: String) -> Self {
        Self {
            id: next_user_id(),
            regno,
            name,
            password_hash: password, // TODO: Hash this
            money: 100_000 * PRICE_SCALE, // Default starting money: 100k
            locked_money: 0,
            margin_locked: 0,
            chat_enabled: true,
            banned: false,
            created_at: chrono::Utc::now().timestamp(),
            portfolio: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: CompanyId,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub total_shares: Quantity,
    pub bankrupt: bool,
    pub price_precision: u8, // e.g., 2 for 0.01, 4 for 0.0001
    pub volatility: i64, // Scaled volatility factor
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub user_id: UserId,
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub qty: Quantity,
    pub filled_qty: Quantity,
    pub price: Price, // Limit price (0 for Market orders, or handled differently)
    pub status: OrderStatus,
    pub timestamp: i64,
    pub time_in_force: TimeInForce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: TradeId,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub maker_user_id: UserId,
    pub taker_user_id: UserId,
    pub symbol: String,
    pub qty: Quantity,
    pub price: Price,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub user_id: UserId,
    pub symbol: String,
    pub qty: Quantity,
    pub short_qty: Quantity,  // Short position quantity
    pub locked_qty: Quantity, // Locked in active sell orders
    pub average_buy_price: Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub resolution: String, // "1m", "5m", "1h"
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub user_id: u64,
    pub username: String,
    pub message: String,
    pub timestamp: i64,
}
