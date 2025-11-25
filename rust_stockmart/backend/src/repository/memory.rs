use crate::domain::models::{Company, CompanyId, User, UserId};
use crate::repository::{CompanyRepository, UserRepository};
use async_trait::async_trait;
use dashmap::DashMap;
use std::error::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct InMemoryUserRepository {
    users: Arc<DashMap<UserId, User>>,
    regno_index: Arc<DashMap<String, UserId>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(DashMap::new()),
            regno_index: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        Ok(self.users.get(&id).map(|u| u.clone()))
    }

    async fn find_by_regno(&self, regno: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        if let Some(id) = self.regno_index.get(regno) {
            self.find_by_id(*id).await
        } else {
            Ok(None)
        }
    }

    async fn save(&self, user: User) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.regno_index.insert(user.regno.clone(), user.id);
        self.users.insert(user.id, user);
        Ok(())
    }

    async fn all(&self) -> Result<Vec<User>, Box<dyn Error + Send + Sync>> {
        Ok(self.users.iter().map(|entry| entry.value().clone()).collect())
    }
}

#[derive(Clone)]
pub struct InMemoryCompanyRepository {
    companies: Arc<DashMap<CompanyId, Company>>,
    symbol_index: Arc<DashMap<String, CompanyId>>,
}

impl InMemoryCompanyRepository {
    pub fn new() -> Self {
        Self {
            companies: Arc::new(DashMap::new()),
            symbol_index: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl CompanyRepository for InMemoryCompanyRepository {
    async fn find_by_id(&self, id: CompanyId) -> Result<Option<Company>, Box<dyn Error + Send + Sync>> {
        Ok(self.companies.get(&id).map(|c| c.clone()))
    }

    async fn find_by_symbol(&self, symbol: &str) -> Result<Option<Company>, Box<dyn Error + Send + Sync>> {
        if let Some(id) = self.symbol_index.get(symbol) {
            self.find_by_id(*id).await
        } else {
            Ok(None)
        }
    }

    async fn save(&self, company: Company) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.symbol_index.insert(company.symbol.clone(), company.id);
        self.companies.insert(company.id, company);
        Ok(())
    }

    async fn create(&self, company: Company) -> Result<CompanyId, Box<dyn Error + Send + Sync>> {
        let id = company.id;
        self.symbol_index.insert(company.symbol.clone(), id);
        self.companies.insert(id, company);
        Ok(id)
    }

    async fn all(&self) -> Result<Vec<Company>, Box<dyn Error + Send + Sync>> {
        Ok(self.companies.iter().map(|r| r.value().clone()).collect())
    }
}
