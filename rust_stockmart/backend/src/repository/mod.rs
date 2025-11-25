use crate::domain::models::{Company, CompanyId, User, UserId};
use async_trait::async_trait;
use std::error::Error;

pub mod memory;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Box<dyn Error + Send + Sync>>;
    async fn find_by_regno(&self, regno: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>>;
    async fn save(&self, user: User) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn all(&self) -> Result<Vec<User>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait CompanyRepository: Send + Sync {
    async fn find_by_id(&self, id: CompanyId) -> Result<Option<Company>, Box<dyn Error + Send + Sync>>;
    async fn find_by_symbol(&self, symbol: &str) -> Result<Option<Company>, Box<dyn Error + Send + Sync>>;
    async fn save(&self, company: Company) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn create(&self, company: Company) -> Result<CompanyId, Box<dyn Error + Send + Sync>>;
    async fn all(&self) -> Result<Vec<Company>, Box<dyn Error + Send + Sync>>;
}
