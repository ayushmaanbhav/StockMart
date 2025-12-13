//! Trade entity for the trading bounded context.
//!
//! A Trade represents a completed transaction between two parties.

use crate::domain::common::{OrderId, Price, Quantity, TradeId, UserId};
use serde::{Deserialize, Serialize};

/// A completed trade between two parties.
///
/// Created when an incoming order matches against a resting order in the orderbook.
/// Both maker (resting order) and taker (incoming order) information is recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade identifier
    pub id: TradeId,
    /// ID of the resting order that was matched
    pub maker_order_id: OrderId,
    /// ID of the incoming order that matched
    pub taker_order_id: OrderId,
    /// User ID of the maker (resting order owner)
    pub maker_user_id: UserId,
    /// User ID of the taker (incoming order owner)
    pub taker_user_id: UserId,
    /// Trading symbol (e.g., "AAPL")
    pub symbol: String,
    /// Quantity traded
    pub qty: Quantity,
    /// Execution price
    pub price: Price,
    /// Unix timestamp of trade execution
    pub timestamp: i64,
}
