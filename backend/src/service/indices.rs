use crate::domain::ui_models::MarketIndexUI;
use crate::domain::CompanyRepository;
use crate::service::market::MarketService;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

/// IndexValue for individual index queries
#[allow(dead_code)] // API type for direct index queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexValue {
    pub name: String,
    pub value: i64, // Scaled
    pub timestamp: i64,
}

pub struct IndicesService {
    market: Arc<MarketService>,
    company_repo: Arc<dyn CompanyRepository>,
    /// Broadcast channel for UI-ready indices
    index_tx: broadcast::Sender<MarketIndexUI>,
    /// Previous values for calculating change
    previous_values: DashMap<String, i64>,
    /// Current indices for state sync
    current_indices: RwLock<Vec<MarketIndexUI>>,
}

impl IndicesService {
    pub fn new(market: Arc<MarketService>, company_repo: Arc<dyn CompanyRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            market,
            company_repo,
            index_tx: tx,
            previous_values: DashMap::new(),
            current_indices: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe_indices(&self) -> broadcast::Receiver<MarketIndexUI> {
        self.index_tx.subscribe()
    }

    /// Get all current indices for state sync
    pub fn get_all_indices(&self) -> Vec<MarketIndexUI> {
        self.current_indices.read().unwrap().clone()
    }

    /// Get a specific index by name
    #[allow(dead_code)] // API method for querying individual indices
    pub fn get_index(&self, name: &str) -> Option<MarketIndexUI> {
        self.current_indices
            .read()
            .unwrap()
            .iter()
            .find(|i| i.name == name)
            .cloned()
    }

    pub async fn run(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;
            self.calculate_indices().await;
        }
    }

    async fn calculate_indices(&self) {
        if let Ok(companies) = self.company_repo.all().await {
            let mut sector_sums: std::collections::HashMap<String, (i64, i64)> =
                std::collections::HashMap::new();
            let mut total_market_price = 0i64;
            let mut total_companies = 0i64;
            let mut updated_indices: Vec<MarketIndexUI> = Vec::new();

            for company in &companies {
                // Get last price from market service
                let candles = self.market.get_candles(&company.symbol);
                let price = if let Some(last) = candles.last() {
                    last.close
                } else {
                    // Fallback to base share price
                    crate::domain::constants::user::BASE_SHARE_PRICE
                };

                let entry = sector_sums.entry(company.sector.clone()).or_insert((0, 0));
                entry.0 += price;
                entry.1 += 1;

                total_market_price += price;
                total_companies += 1;
            }

            let timestamp = chrono::Utc::now().timestamp();

            // Broadcast Sector Indices
            for (sector, (sum, count)) in sector_sums {
                let avg = sum / count;
                let name = format!("SECTOR:{}", sector);

                let index_ui = self.create_index_ui(&name, avg, timestamp);
                updated_indices.push(index_ui.clone());
                let _ = self.index_tx.send(index_ui);
            }

            // Calculate VIX (Simulated) based on actual market volatility
            // Use average company volatility and some smoothed noise
            let avg_volatility: i64 = if !companies.is_empty() {
                companies.iter().map(|c| c.volatility).sum::<i64>() / companies.len() as i64
            } else {
                50 // Default medium volatility
            };

            // VIX is based on volatility factor (0-100) mapped to typical VIX range (10-30)
            // Add very small random noise (±0.5) for natural variation
            use crate::domain::constants::PRICE_SCALE;
            let base_vix = 10 + (avg_volatility * 20 / 100); // Maps 0-100 volatility to 10-30 VIX
            let noise = rand::random::<i64>() % PRICE_SCALE - (PRICE_SCALE / 2); // ±0.50 random noise
            let vix = base_vix * PRICE_SCALE + noise;

            let vix_ui = self.create_index_ui("VIX", vix, timestamp);
            updated_indices.push(vix_ui.clone());
            let _ = self.index_tx.send(vix_ui);

            // Calculate overall market index if we have companies
            if total_companies > 0 {
                let market_avg = total_market_price / total_companies;
                let market_ui = self.create_index_ui("MARKET", market_avg, timestamp);
                updated_indices.push(market_ui.clone());
                let _ = self.index_tx.send(market_ui);
            }

            // Store current indices for sync requests
            {
                let mut current = self.current_indices.write().unwrap();
                *current = updated_indices;
            }
        }
    }

    /// Create a UI-ready index with change calculation
    fn create_index_ui(&self, name: &str, value: i64, timestamp: i64) -> MarketIndexUI {
        // Get previous value and calculate change
        let previous_value = self
            .previous_values
            .insert(name.to_string(), value)
            .unwrap_or(value); // First time: use current value (no change)

        let change = value - previous_value;
        let change_percent = if previous_value != 0 {
            ((value - previous_value) as f64 / previous_value as f64) * 100.0
        } else {
            0.0
        };

        MarketIndexUI {
            name: name.to_string(),
            value,
            previous_value,
            change,
            change_percent,
            timestamp,
        }
    }

    /// Exposed for testing - calculate indices once
    #[cfg(test)]
    pub async fn test_calculate_indices(&self) {
        self.calculate_indices().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Company;
    use crate::infrastructure::persistence::InMemoryCompanyRepository;

    async fn create_test_service() -> IndicesService {
        let market = Arc::new(MarketService::new());
        let repo: Arc<dyn CompanyRepository> = Arc::new(InMemoryCompanyRepository::new());
        IndicesService::new(market, repo)
    }

    async fn create_test_service_with_companies() -> IndicesService {
        let market = Arc::new(MarketService::new());
        let repo = Arc::new(InMemoryCompanyRepository::new());

        // Add some companies
        let company1 = Company {
            id: 1,
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            sector: "Tech".to_string(),
            total_shares: 1_000_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 50,
        };
        let company2 = Company {
            id: 2,
            symbol: "GOOGL".to_string(),
            name: "Alphabet Inc.".to_string(),
            sector: "Tech".to_string(),
            total_shares: 500_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 40,
        };
        let company3 = Company {
            id: 3,
            symbol: "JPM".to_string(),
            name: "JPMorgan Chase".to_string(),
            sector: "Finance".to_string(),
            total_shares: 2_000_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 30,
        };

        repo.save(company1).await.unwrap();
        repo.save(company2).await.unwrap();
        repo.save(company3).await.unwrap();

        IndicesService::new(Arc::new(MarketService::new()), repo)
    }

    #[tokio::test]
    async fn test_indices_service_new() {
        let svc = create_test_service().await;
        // Should be able to subscribe
        let _rx = svc.subscribe_indices();
    }

    #[tokio::test]
    async fn test_get_all_indices_empty() {
        let svc = create_test_service().await;
        assert!(svc.get_all_indices().is_empty());
    }

    #[tokio::test]
    async fn test_get_index_not_found() {
        let svc = create_test_service().await;
        assert!(svc.get_index("MARKET").is_none());
    }

    #[tokio::test]
    async fn test_calculate_indices_with_companies() {
        let svc = create_test_service_with_companies().await;
        svc.test_calculate_indices().await;

        let indices = svc.get_all_indices();
        // Should have sector indices (Tech, Finance), VIX, and MARKET
        assert!(!indices.is_empty());

        // Check for specific indices
        let names: Vec<&str> = indices.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"VIX"));
        assert!(names.contains(&"MARKET"));
        assert!(names.contains(&"SECTOR:Tech"));
        assert!(names.contains(&"SECTOR:Finance"));
    }

    #[tokio::test]
    async fn test_get_index_after_calculation() {
        let svc = create_test_service_with_companies().await;
        svc.test_calculate_indices().await;

        let vix = svc.get_index("VIX");
        assert!(vix.is_some());
        let vix = vix.unwrap();
        assert_eq!(vix.name, "VIX");
        assert!(vix.value > 0);
    }

    #[tokio::test]
    async fn test_index_change_calculation() {
        let svc = create_test_service_with_companies().await;

        // First calculation
        svc.test_calculate_indices().await;
        let first_indices = svc.get_all_indices();

        // Second calculation
        svc.test_calculate_indices().await;
        let second_indices = svc.get_all_indices();

        // MARKET index should exist in both
        let first_market = first_indices.iter().find(|i| i.name == "MARKET");
        let second_market = second_indices.iter().find(|i| i.name == "MARKET");

        assert!(first_market.is_some());
        assert!(second_market.is_some());

        // Second should have previous_value set to first's value
        let first_market = first_market.unwrap();
        let second_market = second_market.unwrap();
        assert_eq!(second_market.previous_value, first_market.value);
    }

    #[test]
    fn test_index_value_struct() {
        let iv = IndexValue {
            name: "TEST".to_string(),
            value: 1000000,
            timestamp: 12345,
        };
        assert_eq!(iv.name, "TEST");
        assert_eq!(iv.value, 1000000);
        assert_eq!(iv.timestamp, 12345);
    }
}
