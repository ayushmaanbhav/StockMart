use crate::service::engine::MatchingEngine;
use crate::repository::CompanyRepository;
use std::sync::Arc;

pub struct AdminService {
    engine: Arc<MatchingEngine>,
    company_repo: Arc<dyn CompanyRepository>,
}

impl AdminService {
    pub fn new(engine: Arc<MatchingEngine>, company_repo: Arc<dyn CompanyRepository>) -> Self {
        Self {
            engine,
            company_repo,
        }
    }

    pub fn toggle_market(&self, open: bool) {
        self.engine.set_market_open(open);
    }

    pub async fn set_company_volatility(&self, symbol: &str, volatility: i64) -> Result<(), String> {
        if let Some(mut company) = self.company_repo.find_by_symbol(symbol).await.map_err(|e| e.to_string())? {
            company.volatility = volatility;
            self.company_repo.save(company).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Company not found".to_string())
        }
    }
    
    pub async fn set_company_bankrupt(&self, symbol: &str, bankrupt: bool) -> Result<(), String> {
        if let Some(mut company) = self.company_repo.find_by_symbol(symbol).await.map_err(|e| e.to_string())? {
            company.bankrupt = bankrupt;
            self.company_repo.save(company).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Company not found".to_string())
        }
    }

    pub async fn create_company(&self, symbol: String, name: String, sector: String, volatility: i64) -> Result<(), String> {
        let company = crate::domain::models::Company {
            id: crate::domain::models::next_company_id(),
            symbol: symbol.clone(),
            name,
            sector,
            total_shares: 1_000_000, // Default IPO shares
            bankrupt: false,
            price_precision: 2,
            volatility,
        };

        self.company_repo.create(company).await.map_err(|e| e.to_string())?;
        self.engine.create_orderbook(symbol);
        Ok(())
    }
}
