//! Service layer - Application services and business logic.
//!
//! This module contains all services that orchestrate domain logic
//! and coordinate between different bounded contexts.

pub mod admin;
pub mod chat;
pub mod engine;
pub mod event_log;
pub mod indices;
pub mod leaderboard;
pub mod market;
pub mod news;
pub mod orders;
pub mod persistence;
pub mod session;
pub mod token;
pub mod trade_history;
