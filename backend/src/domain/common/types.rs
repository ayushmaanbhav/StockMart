//! Core type aliases for the domain layer.
//!
//! These types provide semantic clarity and type safety across all bounded contexts.

/// Unique identifier for users
pub type UserId = u64;

/// Unique identifier for companies/securities
pub type CompanyId = u64;

/// Unique identifier for orders
pub type OrderId = u64;

/// Unique identifier for trades
pub type TradeId = u64;

/// Session identifier for WebSocket connections
#[allow(dead_code)] // Type alias for future use
pub type SessionId = u64;

/// Price in scaled integer format (e.g., 150.25 -> 1_502_500 with PRICE_SCALE=10_000)
pub type Price = i64;

/// Quantity of shares/units
pub type Quantity = u64;

/// Unix timestamp in seconds
#[allow(dead_code)] // Type alias for future use
pub type Timestamp = i64;
