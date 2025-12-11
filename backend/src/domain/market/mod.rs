//! Market bounded context.
//!
//! This module contains market-related domain logic including:
//! - Company entity
//! - Candle (OHLCV) entity
//! - Chat message entity

pub mod company;
pub mod candle;
pub mod chat;

// Re-export entities
pub use company::Company;
pub use candle::Candle;
pub use chat::ChatMessage;
