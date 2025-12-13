//! Shared kernel for the domain layer.
//!
//! This module contains types and traits shared across all bounded contexts.

pub mod types;

pub use types::{CompanyId, OrderId, Price, Quantity, TradeId, UserId};
