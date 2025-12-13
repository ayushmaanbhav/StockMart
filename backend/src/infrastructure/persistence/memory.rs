//! In-memory repository implementations.
//!
//! These implementations use DashMap for concurrent, thread-safe access
//! to data stored in memory. Ideal for development and testing.

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

use crate::domain::error::{RepositoryError, RepositoryResult};
use crate::domain::models::{Company, CompanyId, User, UserId};
use crate::domain::repositories::{CompanyRepository, UserRepository};

// =============================================================================
// IN-MEMORY USER REPOSITORY
// =============================================================================

/// In-memory implementation of UserRepository.
///
/// Uses DashMap for thread-safe concurrent access. Data is lost on restart.
#[derive(Clone)]
pub struct InMemoryUserRepository {
    users: Arc<DashMap<UserId, User>>,
    regno_index: Arc<DashMap<String, UserId>>,
}

impl InMemoryUserRepository {
    /// Create a new empty in-memory user repository.
    pub fn new() -> Self {
        Self {
            users: Arc::new(DashMap::new()),
            regno_index: Arc::new(DashMap::new()),
        }
    }

    /// Create a repository pre-populated with users.
    #[allow(dead_code)] // Helper for testing and initialization
    pub fn with_users(users: Vec<User>) -> Self {
        let repo = Self::new();
        for user in users {
            repo.regno_index.insert(user.regno.clone(), user.id);
            repo.users.insert(user.id, user);
        }
        repo
    }

    /// Get the count of users (sync version for metrics).
    #[allow(dead_code)] // Helper for metrics
    pub fn len(&self) -> usize {
        self.users.len()
    }

    /// Check if empty.
    #[allow(dead_code)] // Helper for testing
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>> {
        Ok(self.users.get(&id).map(|u| u.clone()))
    }

    async fn find_by_regno(&self, regno: &str) -> RepositoryResult<Option<User>> {
        if let Some(id) = self.regno_index.get(regno) {
            self.find_by_id(*id).await
        } else {
            Ok(None)
        }
    }

    async fn save(&self, user: User) -> RepositoryResult<()> {
        // Update regno index
        self.regno_index.insert(user.regno.clone(), user.id);
        // Update user
        self.users.insert(user.id, user);
        Ok(())
    }

    async fn delete(&self, id: UserId) -> RepositoryResult<bool> {
        if let Some((_, user)) = self.users.remove(&id) {
            self.regno_index.remove(&user.regno);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn all(&self) -> RepositoryResult<Vec<User>> {
        Ok(self
            .users
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn count(&self) -> RepositoryResult<usize> {
        Ok(self.users.len())
    }
}

// =============================================================================
// IN-MEMORY COMPANY REPOSITORY
// =============================================================================

/// In-memory implementation of CompanyRepository.
///
/// Uses DashMap for thread-safe concurrent access. Data is lost on restart.
#[derive(Clone)]
pub struct InMemoryCompanyRepository {
    companies: Arc<DashMap<CompanyId, Company>>,
    symbol_index: Arc<DashMap<String, CompanyId>>,
}

impl InMemoryCompanyRepository {
    /// Create a new empty in-memory company repository.
    pub fn new() -> Self {
        Self {
            companies: Arc::new(DashMap::new()),
            symbol_index: Arc::new(DashMap::new()),
        }
    }

    /// Create a repository pre-populated with companies.
    #[allow(dead_code)] // Helper for testing and initialization
    pub fn with_companies(companies: Vec<Company>) -> Self {
        let repo = Self::new();
        for company in companies {
            repo.symbol_index.insert(company.symbol.clone(), company.id);
            repo.companies.insert(company.id, company);
        }
        repo
    }

    /// Get the count of companies (sync version for metrics).
    #[allow(dead_code)] // Helper for metrics
    pub fn len(&self) -> usize {
        self.companies.len()
    }

    /// Check if empty.
    #[allow(dead_code)] // Helper for testing
    pub fn is_empty(&self) -> bool {
        self.companies.is_empty()
    }
}

impl Default for InMemoryCompanyRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CompanyRepository for InMemoryCompanyRepository {
    async fn find_by_id(&self, id: CompanyId) -> RepositoryResult<Option<Company>> {
        Ok(self.companies.get(&id).map(|c| c.clone()))
    }

    async fn find_by_symbol(&self, symbol: &str) -> RepositoryResult<Option<Company>> {
        if let Some(id) = self.symbol_index.get(symbol) {
            self.find_by_id(*id).await
        } else {
            Ok(None)
        }
    }

    async fn save(&self, company: Company) -> RepositoryResult<()> {
        self.symbol_index.insert(company.symbol.clone(), company.id);
        self.companies.insert(company.id, company);
        Ok(())
    }

    async fn create(&self, company: Company) -> RepositoryResult<CompanyId> {
        // Check if symbol already exists
        if self.symbol_index.contains_key(&company.symbol) {
            return Err(RepositoryError::SaveFailed {
                reason: format!("Company with symbol '{}' already exists", company.symbol),
            });
        }

        let id = company.id;
        self.symbol_index.insert(company.symbol.clone(), id);
        self.companies.insert(id, company);
        Ok(id)
    }

    async fn delete(&self, id: CompanyId) -> RepositoryResult<bool> {
        if let Some((_, company)) = self.companies.remove(&id) {
            self.symbol_index.remove(&company.symbol);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn all(&self) -> RepositoryResult<Vec<Company>> {
        Ok(self.companies.iter().map(|r| r.value().clone()).collect())
    }

    async fn count(&self) -> RepositoryResult<usize> {
        Ok(self.companies.len())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::User;

    fn create_test_user(regno: &str) -> User {
        // Use User::new to create a properly initialized user
        User::new(
            regno.to_string(),
            format!("Test User {}", regno),
            "test_password".to_string(),
        )
    }

    #[tokio::test]
    async fn test_user_repo_save_and_find() {
        let repo = InMemoryUserRepository::new();
        let user = create_test_user("TEST001");
        let user_id = user.id;

        repo.save(user.clone()).await.unwrap();

        let found = repo.find_by_id(user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().regno, "TEST001");
    }

    #[tokio::test]
    async fn test_user_repo_find_by_regno() {
        let repo = InMemoryUserRepository::new();
        let user = create_test_user("TEST001");
        let user_id = user.id;

        repo.save(user.clone()).await.unwrap();

        let found = repo.find_by_regno("TEST001").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user_id);
    }

    #[tokio::test]
    async fn test_user_repo_delete() {
        let repo = InMemoryUserRepository::new();
        let user = create_test_user("TEST001");
        let user_id = user.id;

        repo.save(user).await.unwrap();
        assert_eq!(repo.count().await.unwrap(), 1);

        let deleted = repo.delete(user_id).await.unwrap();
        assert!(deleted);
        assert_eq!(repo.count().await.unwrap(), 0);

        let found = repo.find_by_id(user_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_company_repo_create_duplicate_fails() {
        let repo = InMemoryCompanyRepository::new();
        let company = Company {
            id: 1,
            symbol: "TEST".to_string(),
            name: "Test Company".to_string(),
            sector: "Tech".to_string(),
            total_shares: 1_000_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 100,
        };

        repo.create(company.clone()).await.unwrap();

        let result = repo.create(company).await;
        assert!(result.is_err());
    }
}
