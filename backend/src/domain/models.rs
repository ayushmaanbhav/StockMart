//! Domain models - Re-exports from bounded contexts for backward compatibility.
//!
//! This module maintains backward compatibility by re-exporting domain types
//! from their respective bounded contexts. New code should import directly
//! from the bounded context modules:
//!
//! - `domain::trading::{Order, Trade, OrderType, OrderSide, OrderStatus, TimeInForce}`
//! - `domain::user::{User, Portfolio, Role}`
//! - `domain::market::{Company, Candle, ChatMessage}`
//! - `domain::common::{UserId, CompanyId, OrderId, TradeId, Price, Quantity}`

// Re-export type aliases from common module
pub use crate::domain::common::{
    UserId, CompanyId, OrderId, TradeId, Price, Quantity,
};

// Re-export PRICE_SCALE from constants
pub use crate::domain::constants::PRICE_SCALE;

// Re-export trading types
pub use crate::domain::trading::{
    Order, Trade, OrderType, OrderSide, OrderStatus, TimeInForce,
};

// Re-export user types
pub use crate::domain::user::{User, Portfolio};

// Re-export market types
pub use crate::domain::market::{Company, Candle, ChatMessage};
