//! Infrastructure layer - external concerns and implementations.
//!
//! This module contains:
//! - ID generation utilities
//! - Server setup and graceful shutdown
//! - Repository implementations

pub mod id_generator;
pub mod persistence;
pub mod shutdown;
