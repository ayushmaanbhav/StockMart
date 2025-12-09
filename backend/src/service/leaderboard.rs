use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use crate::domain::models::UserId;
use crate::domain::ui_models::LeaderboardEntryUI;
use crate::repository::UserRepository;
use crate::service::market::MarketService;

/// Legacy entry for backward compatibility during migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub name: String,
    pub net_worth: i64,
}

pub struct LeaderboardService {
    user_repo: Arc<dyn UserRepository>,
    market_service: Arc<MarketService>,
    /// Broadcast channel for UI-ready entries
    lb_tx: broadcast::Sender<Vec<LeaderboardEntryUI>>,
    /// Previous rankings for calculating rank changes
    previous_rankings: RwLock<HashMap<UserId, usize>>,
    /// Current leaderboard for sync requests
    current_leaderboard: RwLock<Vec<LeaderboardEntryUI>>,
}

impl LeaderboardService {
    pub fn new(user_repo: Arc<dyn UserRepository>, market_service: Arc<MarketService>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            user_repo,
            market_service,
            lb_tx: tx,
            previous_rankings: RwLock::new(HashMap::new()),
            current_leaderboard: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<LeaderboardEntryUI>> {
        self.lb_tx.subscribe()
    }

    /// Get current leaderboard for state sync
    pub fn get_current(&self) -> Vec<LeaderboardEntryUI> {
        self.current_leaderboard.read().unwrap().clone()
    }

    pub async fn run(&self) {
        loop {
            sleep(Duration::from_secs(5)).await;
            self.update_leaderboard().await;
        }
    }

    async fn update_leaderboard(&self) {
        if let Ok(users) = self.user_repo.all().await {
            let mut entries: Vec<LeaderboardEntryUI> = Vec::new();

            for user in users {
                // Calculate portfolio value correctly:
                // Long positions ADD value, short positions SUBTRACT value (liability)
                let mut portfolio_value: i64 = 0;
                for item in &user.portfolio {
                    if let Some(price) = self.market_service.get_last_price(&item.symbol) {
                        // Long positions add value
                        portfolio_value += (item.qty as i64) * price;
                        // Short positions are a liability (subtract value)
                        portfolio_value -= (item.short_qty as i64) * price;
                    }
                }

                // CORRECT NET WORTH CALCULATION:
                // money (available) + locked_money (in buy orders) + margin_locked (for shorts) + portfolio_value
                let net_worth = user.money + user.locked_money + user.margin_locked + portfolio_value;

                entries.push(LeaderboardEntryUI {
                    rank: 0, // Will assign later
                    user_id: user.id,
                    name: user.name.clone(),
                    net_worth,
                    change_rank: 0, // Will calculate after sorting
                });
            }

            // Sort by net worth descending
            entries.sort_by(|a, b| b.net_worth.cmp(&a.net_worth));

            // Get previous rankings for change calculation
            let prev_rankings = self.previous_rankings.read().unwrap().clone();

            // Assign ranks and calculate rank changes, take top 10
            let top_10: Vec<LeaderboardEntryUI> = entries
                .into_iter()
                .enumerate()
                .take(10)
                .map(|(i, mut entry)| {
                    let new_rank = i + 1;
                    entry.rank = new_rank;

                    // Calculate rank change (positive = moved up, negative = moved down)
                    entry.change_rank = prev_rankings
                        .get(&entry.user_id)
                        .map(|&prev_rank| prev_rank as i32 - new_rank as i32)
                        .unwrap_or(0);

                    entry
                })
                .collect();

            // Store current rankings for next update
            {
                let mut prev = self.previous_rankings.write().unwrap();
                prev.clear();
                for entry in &top_10 {
                    prev.insert(entry.user_id, entry.rank);
                }
            }

            // Store current leaderboard for sync requests
            {
                let mut current = self.current_leaderboard.write().unwrap();
                *current = top_10.clone();
            }

            let _ = self.lb_tx.send(top_10);
        }
    }
}
