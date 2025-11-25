use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use crate::repository::UserRepository;
use crate::service::market::MarketService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub name: String,
    pub net_worth: i64,
}

pub struct LeaderboardService {
    user_repo: Arc<dyn UserRepository>,
    market_service: Arc<MarketService>,
    lb_tx: broadcast::Sender<Vec<LeaderboardEntry>>,
}

impl LeaderboardService {
    pub fn new(user_repo: Arc<dyn UserRepository>, market_service: Arc<MarketService>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            user_repo,
            market_service,
            lb_tx: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<LeaderboardEntry>> {
        self.lb_tx.subscribe()
    }

    pub async fn run(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;
            self.update_leaderboard().await;
        }
    }

    async fn update_leaderboard(&self) {
        if let Ok(users) = self.user_repo.all().await {
            let mut entries: Vec<LeaderboardEntry> = Vec::new();

            for user in users {
                let mut portfolio_value = 0;
                for item in &user.portfolio {
                    if let Some(price) = self.market_service.get_last_price(&item.symbol) {
                        portfolio_value += (item.qty as i64) * price;
                    }
                }

                let net_worth = user.money + portfolio_value;
                entries.push(LeaderboardEntry {
                    rank: 0, // Will assign later
                    name: user.name,
                    net_worth,
                });
            }

            // Sort by net worth descending
            entries.sort_by(|a, b| b.net_worth.cmp(&a.net_worth));

            // Assign ranks and take top 10
            let top_10: Vec<LeaderboardEntry> = entries.into_iter().enumerate().take(10).map(|(i, mut entry)| {
                entry.rank = i + 1;
                entry
            }).collect();

            let _ = self.lb_tx.send(top_10);
        }
    }
}
