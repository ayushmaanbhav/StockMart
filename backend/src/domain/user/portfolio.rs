//! Portfolio entity for the user bounded context.
//!
//! A Portfolio represents a user's position in a specific security.

use crate::domain::common::{Price, Quantity, UserId};
use serde::{Deserialize, Serialize};

/// A user's position in a specific security.
///
/// Tracks both long positions (qty), short positions (short_qty),
/// and shares locked in open orders (locked_qty).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// ID of the user who owns this position
    pub user_id: UserId,
    /// Trading symbol (e.g., "AAPL")
    pub symbol: String,
    /// Number of shares owned (long position)
    pub qty: Quantity,
    /// Number of shares borrowed and sold (short position)
    pub short_qty: Quantity,
    /// Number of shares locked in pending sell orders
    pub locked_qty: Quantity,
    /// Average price paid per share
    pub average_buy_price: Price,
}
