//! Domain layer - Core business logic organized by bounded contexts.
//!
//! This module follows Domain-Driven Design principles with:
//! - `common`: Shared kernel (type aliases, common traits)
//! - `trading`: Trading bounded context (orders, trades, orderbook)
//! - `market`: Market bounded context (companies, candles)
//! - `user`: User bounded context (users, portfolios, roles)

// Bounded contexts
pub mod common;
pub mod market;
pub mod trading;
pub mod user;

// Core domain infrastructure
pub mod constants;
pub mod error;
pub mod models;
pub mod repositories;
pub mod ui_models;

// Re-export repository traits (used throughout the codebase)
pub use repositories::{CompanyRepository, UserRepository};
