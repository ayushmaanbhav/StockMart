//! WebSocket message types.
//!
//! This module contains the message definitions for client-server communication.

pub mod client;
pub mod server;

pub use client::ClientMessage;
pub use server::{CompanyInfo, CurrencyConfigPayload, ServerMessage};
