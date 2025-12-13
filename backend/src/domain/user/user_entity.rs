//! User entity for the user bounded context.
//!
//! A User represents a trader in the system.

use super::portfolio::Portfolio;
use super::role::Role;
use crate::domain::common::{Price, UserId};
use crate::domain::constants::user::DEFAULT_STARTING_MONEY;
use crate::infrastructure::id_generator::IdGenerators;
use serde::{Deserialize, Serialize};

/// A user/trader in the system.
///
/// Users can place orders, hold portfolios, and chat.
/// Admins have elevated privileges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: UserId,
    /// Registration number (login identifier)
    pub regno: String,
    /// Display name
    pub name: String,
    /// Password (plaintext for now - not production ready)
    pub password_hash: String,
    /// User's role for RBAC
    #[serde(default)]
    pub role: Role,
    /// Available cash balance
    pub money: Price,
    /// Cash locked in pending buy orders
    pub locked_money: Price,
    /// Cash locked as margin for short positions
    pub margin_locked: Price,
    /// Whether user can send chat messages
    pub chat_enabled: bool,
    /// Whether user is banned from trading
    pub banned: bool,
    /// Unix timestamp of account creation
    pub created_at: i64,
    /// User's portfolio positions
    pub portfolio: Vec<Portfolio>,
}

impl User {
    /// Create a new user with default settings.
    pub fn new(regno: String, name: String, password: String) -> Self {
        Self {
            id: IdGenerators::global().next_user_id(),
            regno,
            name,
            password_hash: password,
            role: Role::Trader,
            money: DEFAULT_STARTING_MONEY,
            locked_money: 0,
            margin_locked: 0,
            chat_enabled: true,
            banned: false,
            created_at: chrono::Utc::now().timestamp(),
            portfolio: Vec::new(),
        }
    }

    /// Check if the user is an admin.
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}
