//! Repository traits for domain entities.
//!
//! These traits define the contracts for data persistence. Implementations
//! live in the infrastructure layer, allowing the domain to remain independent
//! of persistence details.

use crate::domain::error::RepositoryResult;
use crate::domain::models::{Company, CompanyId, User, UserId};
use async_trait::async_trait;

// =============================================================================
// USER REPOSITORY
// =============================================================================

/// Repository trait for User aggregate persistence.
///
/// Implementations must be thread-safe (Send + Sync) to allow concurrent access.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their unique ID.
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>>;

    /// Find a user by their registration number.
    async fn find_by_regno(&self, regno: &str) -> RepositoryResult<Option<User>>;

    /// Save (insert or update) a user.
    async fn save(&self, user: User) -> RepositoryResult<()>;

    /// Delete a user by ID.
    #[allow(dead_code)] // API method for user management
    async fn delete(&self, id: UserId) -> RepositoryResult<bool>;

    /// Get all users.
    async fn all(&self) -> RepositoryResult<Vec<User>>;

    /// Count total users.
    async fn count(&self) -> RepositoryResult<usize>;

    /// Check if a registration number already exists.
    async fn regno_exists(&self, regno: &str) -> RepositoryResult<bool> {
        Ok(self.find_by_regno(regno).await?.is_some())
    }
}

// =============================================================================
// COMPANY REPOSITORY
// =============================================================================

/// Repository trait for Company aggregate persistence.
///
/// Implementations must be thread-safe (Send + Sync) to allow concurrent access.
#[async_trait]
pub trait CompanyRepository: Send + Sync {
    /// Find a company by its unique ID.
    async fn find_by_id(&self, id: CompanyId) -> RepositoryResult<Option<Company>>;

    /// Find a company by its ticker symbol.
    async fn find_by_symbol(&self, symbol: &str) -> RepositoryResult<Option<Company>>;

    /// Save (insert or update) a company.
    async fn save(&self, company: Company) -> RepositoryResult<()>;

    /// Create a new company and return its assigned ID.
    async fn create(&self, company: Company) -> RepositoryResult<CompanyId>;

    /// Delete a company by ID.
    #[allow(dead_code)] // API method for company management
    async fn delete(&self, id: CompanyId) -> RepositoryResult<bool>;

    /// Get all companies.
    async fn all(&self) -> RepositoryResult<Vec<Company>>;

    /// Get all tradable (non-bankrupt) companies.
    #[allow(dead_code)] // API method for filtering
    async fn all_tradable(&self) -> RepositoryResult<Vec<Company>> {
        Ok(self
            .all()
            .await?
            .into_iter()
            .filter(|c| c.is_tradable())
            .collect())
    }

    /// Count total companies.
    #[allow(dead_code)] // API method for stats
    async fn count(&self) -> RepositoryResult<usize>;

    /// Check if a symbol already exists.
    async fn symbol_exists(&self, symbol: &str) -> RepositoryResult<bool> {
        Ok(self.find_by_symbol(symbol).await?.is_some())
    }
}

// =============================================================================
// ORDER REPOSITORY (Future)
// =============================================================================

// Note: Orders are currently managed in-memory by the MatchingEngine.
// When persistence is needed, an OrderRepository trait can be added here.

// =============================================================================
// TRADE REPOSITORY (Future)
// =============================================================================

// Note: Trades are currently managed in-memory by TradeHistoryService.
// When persistence is needed, a TradeRepository trait can be added here.
