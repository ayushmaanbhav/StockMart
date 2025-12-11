//! Order types for the trading bounded context.
//!
//! This module contains order-related enums used throughout the trading system.
//! The Order struct is defined in domain/models.rs for backward compatibility.

use serde::{Deserialize, Serialize};

/// Type of order execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Execute immediately at best available price
    Market,
    /// Execute only at specified price or better
    Limit,
}

/// Direction of the order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buy to open or increase long position
    Buy,
    /// Sell to close or reduce long position
    Sell,
    /// Short sell (sell borrowed shares)
    Short,
}

/// Current status of the order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order is active and waiting for fills
    Open,
    /// Order has been partially filled
    Partial,
    /// Order has been completely filled
    Filled,
    /// Order was cancelled by user
    Cancelled,
    /// Order was rejected by the system
    Rejected,
}

/// Time-in-force policy for the order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancelled - remains active until filled or cancelled
    GTC,
    /// Immediate or Cancel - fill immediately or cancel unfilled portion
    IOC,
}
