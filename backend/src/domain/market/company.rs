//! Company entity for the market bounded context.
//!
//! A Company represents a tradable security/stock.

use crate::domain::common::{CompanyId, Quantity};
use serde::{Deserialize, Serialize};

/// A tradable company/security.
///
/// Represents a stock that can be traded on the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// Unique company identifier
    pub id: CompanyId,
    /// Trading symbol (e.g., "AAPL")
    pub symbol: String,
    /// Full company name
    pub name: String,
    /// Industry sector
    pub sector: String,
    /// Total shares outstanding
    pub total_shares: Quantity,
    /// Whether company is bankrupt (halts trading)
    pub bankrupt: bool,
    /// Decimal places for price display
    pub price_precision: u8,
    /// Volatility factor for price simulation
    pub volatility: i64,
}

impl Company {
    /// Check if the company can be traded.
    #[allow(dead_code)] // Used in CompanyRepository::all_tradable
    pub fn is_tradable(&self) -> bool {
        !self.bankrupt
    }
}
