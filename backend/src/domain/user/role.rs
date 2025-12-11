//! User role definitions for Role-Based Access Control (RBAC).
//!
//! This module replaces the hardcoded `user_id == 1` admin check with
//! a proper role-based system.

use serde::{Deserialize, Serialize};

/// User roles for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Administrator with full system access
    Admin,

    /// Regular trader with standard permissions
    #[default]
    Trader,
}

impl Role {
    /// Check if the role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can perform trading operations
    #[allow(dead_code)] // RBAC API for future trading restrictions
    pub fn can_trade(&self) -> bool {
        // Both admin and trader can trade
        matches!(self, Role::Admin | Role::Trader)
    }

    /// Check if the role can view market data
    #[allow(dead_code)] // RBAC API for future visibility restrictions
    pub fn can_view_market_data(&self) -> bool {
        // All roles can view market data
        true
    }

    /// Check if the role can manage users (ban, mute, etc.)
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can manage companies (create, bankrupt, etc.)
    pub fn can_manage_companies(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can control market status (open/close)
    pub fn can_control_market(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can initialize or reset the game
    pub fn can_init_game(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can view admin dashboard metrics
    pub fn can_view_admin_dashboard(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can view all trades (admin trade history)
    pub fn can_view_all_trades(&self) -> bool {
        matches!(self, Role::Admin)
    }

    /// Check if the role can view all open orders (admin view)
    pub fn can_view_all_orders(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Trader => write!(f, "trader"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = RoleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "trader" => Ok(Role::Trader),
            _ => Err(RoleParseError(s.to_string())),
        }
    }
}

/// Error when parsing a role from string
#[derive(Debug, Clone)]
pub struct RoleParseError(pub String);

impl std::fmt::Display for RoleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid role: {}. Expected 'admin' or 'trader'", self.0)
    }
}

impl std::error::Error for RoleParseError {}

/// Admin actions that require authorization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAction {
    ToggleMarket,
    SetVolatility,
    CreateCompany,
    InitGame,
    SetBankrupt,
    BanTrader,
    MuteTrader,
    GetAllTrades,
    GetAllOpenOrders,
    GetOrderbook,
    GetDashboardMetrics,
}

impl AdminAction {
    /// Parse admin action from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ToggleMarket" => Some(AdminAction::ToggleMarket),
            "SetVolatility" => Some(AdminAction::SetVolatility),
            "CreateCompany" => Some(AdminAction::CreateCompany),
            "InitGame" => Some(AdminAction::InitGame),
            "SetBankrupt" => Some(AdminAction::SetBankrupt),
            "BanTrader" => Some(AdminAction::BanTrader),
            "MuteTrader" => Some(AdminAction::MuteTrader),
            "GetAllTrades" => Some(AdminAction::GetAllTrades),
            "GetAllOpenOrders" => Some(AdminAction::GetAllOpenOrders),
            "GetOrderbook" => Some(AdminAction::GetOrderbook),
            "GetDashboardMetrics" => Some(AdminAction::GetDashboardMetrics),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_is_admin() {
        assert!(Role::Admin.is_admin());
        assert!(!Role::Trader.is_admin());
    }

    #[test]
    fn test_role_can_trade() {
        assert!(Role::Admin.can_trade());
        assert!(Role::Trader.can_trade());
    }

    #[test]
    fn test_role_admin_actions() {
        // Test granular permission methods
        assert!(Role::Admin.can_control_market());
        assert!(!Role::Trader.can_control_market());
        assert!(Role::Admin.can_manage_users());
        assert!(!Role::Trader.can_manage_users());
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!("admin".parse::<Role>().unwrap(), Role::Admin);
        assert_eq!("trader".parse::<Role>().unwrap(), Role::Trader);
        assert_eq!("ADMIN".parse::<Role>().unwrap(), Role::Admin);
        assert!("unknown".parse::<Role>().is_err());
    }

    #[test]
    fn test_role_display() {
        assert_eq!(Role::Admin.to_string(), "admin");
        assert_eq!(Role::Trader.to_string(), "trader");
    }

    #[test]
    fn test_default_role() {
        let role: Role = Default::default();
        assert_eq!(role, Role::Trader);
    }

    #[test]
    fn test_granular_permissions() {
        // Admin can do everything
        assert!(Role::Admin.can_init_game());
        assert!(Role::Admin.can_view_admin_dashboard());
        assert!(Role::Admin.can_view_all_trades());
        assert!(Role::Admin.can_view_all_orders());
        assert!(Role::Admin.can_manage_companies());

        // Trader cannot do admin actions
        assert!(!Role::Trader.can_init_game());
        assert!(!Role::Trader.can_view_admin_dashboard());
        assert!(!Role::Trader.can_view_all_trades());
        assert!(!Role::Trader.can_view_all_orders());
        assert!(!Role::Trader.can_manage_companies());
    }
}
