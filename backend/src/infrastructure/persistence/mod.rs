//! Persistence implementations for domain repositories.
//!
//! This module provides concrete repository implementations:
//! - In-memory repositories using DashMap for concurrent access
//! - JSON file persistence (future)

mod memory;

pub use memory::{InMemoryCompanyRepository, InMemoryUserRepository};
