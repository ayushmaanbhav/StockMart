use crate::domain::models::{Company, User};
use crate::domain::{CompanyRepository, UserRepository};
use serde_json;
use std::fs;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct PersistenceService {
    user_repo: Arc<dyn UserRepository>,
    company_repo: Arc<dyn CompanyRepository>,
    data_dir: String,
}

impl PersistenceService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        company_repo: Arc<dyn CompanyRepository>,
        data_dir: String,
    ) -> Self {
        // Ensure data directory exists
        let _ = fs::create_dir_all(&data_dir);
        Self {
            user_repo,
            company_repo,
            data_dir,
        }
    }

    pub async fn load_data(&self) {
        // Load Users
        let users_path = format!("{}/users.json", self.data_dir);
        if let Ok(content) = fs::read_to_string(&users_path) {
            if let Ok(users) = serde_json::from_str::<Vec<User>>(&content) {
                for user in users {
                    let _ = self.user_repo.save(user).await;
                }
                tracing::info!("Loaded users from disk");
            }
        }

        // Load Companies
        let companies_path = format!("{}/companies.json", self.data_dir);
        if let Ok(content) = fs::read_to_string(&companies_path) {
            if let Ok(companies) = serde_json::from_str::<Vec<Company>>(&content) {
                for company in companies {
                    let _ = self.company_repo.save(company).await;
                }
                tracing::info!("Loaded companies from disk");
            }
        }
    }

    pub async fn run(&self) {
        loop {
            sleep(Duration::from_secs(60)).await; // Save every minute
            self.save_data().await;
        }
    }

    /// Save all data to disk.
    ///
    /// This method is called periodically and during graceful shutdown
    /// to ensure data is persisted.
    pub async fn save_data(&self) {
        // Save Users
        if let Ok(users) = self.user_repo.all().await {
            let users_path = format!("{}/users.json", self.data_dir);
            let _ = fs::write(
                users_path,
                serde_json::to_string_pretty(&users).unwrap_or_default(),
            );
        }

        // Save Companies
        if let Ok(companies) = self.company_repo.all().await {
            let companies_path = format!("{}/companies.json", self.data_dir);
            let _ = fs::write(
                companies_path,
                serde_json::to_string_pretty(&companies).unwrap_or_default(),
            );
        }

        tracing::info!("Saved data to disk");
    }
}
