//! Shared kernel for the domain layer.
//!
//! This module contains types and traits shared across all bounded contexts.

pub mod types;

pub use types::{
    UserId, CompanyId, OrderId, TradeId,
    Price, Quantity,
};
