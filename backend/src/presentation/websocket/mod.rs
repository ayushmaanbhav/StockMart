//! WebSocket API layer.
//!
//! This module handles WebSocket communication with clients including:
//! - Message types (client and server)
//! - Connection handling
//! - Message handlers
//! - Application state

pub mod connection;
pub mod handlers;
pub mod messages;
pub mod state;

// Types are accessed directly from submodules rather than via re-exports
// e.g., crate::presentation::websocket::messages::ServerMessage
