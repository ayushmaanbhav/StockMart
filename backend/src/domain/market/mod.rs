//! Market bounded context.
//!
//! This module contains market-related domain logic including:
//! - Company entity
//! - Candle (OHLCV) entity
//! - Chat message entity

pub mod candle;
pub mod chat;
pub mod company;

// Re-export entities
pub use candle::Candle;
pub use chat::ChatMessage;
pub use company::Company;
