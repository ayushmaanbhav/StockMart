use crate::service::market::MarketService;
use crate::repository::CompanyRepository;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexValue {
    pub name: String,
    pub value: i64, // Scaled
    pub timestamp: i64,
}

pub struct IndicesService {
    market: Arc<MarketService>,
    company_repo: Arc<dyn CompanyRepository>,
    index_tx: broadcast::Sender<IndexValue>,
}

impl IndicesService {
    pub fn new(market: Arc<MarketService>, company_repo: Arc<dyn CompanyRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            market,
            company_repo,
            index_tx: tx,
        }
    }

    pub fn subscribe_indices(&self) -> broadcast::Receiver<IndexValue> {
        self.index_tx.subscribe()
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
            let mut total_market_price = 0;
            let mut total_companies = 0;

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

            // Broadcast Sector Indices
            for (sector, (sum, count)) in sector_sums {
                let avg = sum / count;
                let _ = self.index_tx.send(IndexValue {
                    name: format!("SECTOR:{}", sector),
                    value: avg,
                    timestamp: chrono::Utc::now().timestamp(),
                });
            }

            // Calculate VIX (Simulated)
            // In a real app, this would be complex. Here, we'll just use a random walk or based on volatility.
            // Let's use a base value + some noise.
            let vix = 15 * 10000 + (rand::random::<i64>() % 50000 - 25000); // 15.00 +/- 2.50
            let _ = self.index_tx.send(IndexValue {
                name: "VIX".to_string(),
                value: vix,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
    }
}
