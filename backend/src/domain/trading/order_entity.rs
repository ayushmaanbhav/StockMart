//! Order entity for the trading bounded context.
//!
//! An Order represents a user's intent to buy or sell a security.

use super::order::{OrderSide, OrderStatus, OrderType, TimeInForce};
use crate::domain::common::{OrderId, Price, Quantity, UserId};
use serde::{Deserialize, Serialize};

/// A trading order placed by a user.
///
/// Orders can be market or limit orders, and can be for buying, selling,
/// or short-selling securities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: OrderId,
    /// ID of the user who placed the order
    pub user_id: UserId,
    /// Trading symbol (e.g., "AAPL")
    pub symbol: String,
    /// Market or Limit order
    pub order_type: OrderType,
    /// Buy, Sell, or Short
    pub side: OrderSide,
    /// Total quantity requested
    pub qty: Quantity,
    /// Quantity already filled
    pub filled_qty: Quantity,
    /// Limit price (for limit orders) or execution price
    pub price: Price,
    /// Current order status
    pub status: OrderStatus,
    /// Unix timestamp when order was placed
    pub timestamp: i64,
    /// Time-in-force policy
    pub time_in_force: TimeInForce,
}

impl Order {
    /// Get the remaining unfilled quantity.
    pub fn remaining_qty(&self) -> Quantity {
        self.qty.saturating_sub(self.filled_qty)
    }

    /// Check if the order is active (can receive fills).
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::Open | OrderStatus::Partial)
    }
}
