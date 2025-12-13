//! User bounded context.
//!
//! This module contains user-related domain logic including:
//! - User entity
//! - Portfolio entity
//! - Role-based access control (RBAC)

pub mod portfolio;
pub mod role;
pub mod user_entity;

// Re-export role types (used by handlers)
pub use role::AdminAction;

// Re-export entities
pub use portfolio::Portfolio;
pub use user_entity::User;
