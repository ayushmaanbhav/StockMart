//! Trading bounded context.
//!
//! This module contains trading-related domain logic including:
//! - Order types and entities
//! - Trade entity
//! - Orderbook for price-time priority matching

pub mod order;
pub mod order_entity;
pub mod orderbook;
pub mod trade;

// Re-export order enums
pub use order::{OrderSide, OrderStatus, OrderType, TimeInForce};

// Re-export entities
pub use order_entity::Order;
pub use orderbook::OrderBook;
pub use trade::Trade;
