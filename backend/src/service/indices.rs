use crate::domain::ui_models::MarketIndexUI;
use crate::service::market::MarketService;
use crate::repository::CompanyRepository;
use dashmap::DashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};

/// Legacy IndexValue for backward compatibility
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
            let mut sector_sums: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();
            let mut total_market_price = 0i64;
            let mut total_companies = 0i64;
            let mut updated_indices: Vec<MarketIndexUI> = Vec::new();

            for company in &companies {
                // Get last price from market service
                let candles = self.market.get_candles(&company.symbol);
                let price = if let Some(last) = candles.last() {
                    last.close
                } else {
                    // Fallback to some base price or skip
                    100 * 10000 // 100.00 default
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
            let base_vix = 10 + (avg_volatility * 20 / 100); // Maps 0-100 volatility to 10-30 VIX
            let noise = rand::random::<i64>() % 10000 - 5000; // ±0.50 random noise
            let vix = base_vix * 10000 + noise;

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
        let previous_value = self.previous_values
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
}
