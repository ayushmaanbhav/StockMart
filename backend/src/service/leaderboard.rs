use crate::domain::models::UserId;
use crate::domain::ui_models::LeaderboardEntryUI;
use crate::domain::UserRepository;
use crate::service::market::MarketService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

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
                let net_worth =
                    user.money + user.locked_money + user.margin_locked + portfolio_value;

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

    /// Exposed for testing - update leaderboard once
    #[cfg(test)]
    pub async fn test_update_leaderboard(&self) {
        self.update_leaderboard().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Portfolio, User};
    use crate::infrastructure::persistence::InMemoryUserRepository;

    async fn create_test_service() -> LeaderboardService {
        let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
        let market = Arc::new(MarketService::new());
        LeaderboardService::new(repo, market)
    }

    async fn create_test_service_with_users() -> LeaderboardService {
        let repo = Arc::new(InMemoryUserRepository::new());
        let market = Arc::new(MarketService::new());

        // Create users with different net worths
        let mut user1 = User::new(
            "REG001".to_string(),
            "Rich User".to_string(),
            "pass".to_string(),
        );
        user1.id = 1;
        user1.money = 500_000_000_000; // $50M

        let mut user2 = User::new(
            "REG002".to_string(),
            "Medium User".to_string(),
            "pass".to_string(),
        );
        user2.id = 2;
        user2.money = 100_000_000_000; // $10M

        let mut user3 = User::new(
            "REG003".to_string(),
            "Poor User".to_string(),
            "pass".to_string(),
        );
        user3.id = 3;
        user3.money = 10_000_000_000; // $1M

        repo.save(user1).await.unwrap();
        repo.save(user2).await.unwrap();
        repo.save(user3).await.unwrap();

        LeaderboardService::new(repo, market)
    }

    #[tokio::test]
    async fn test_leaderboard_service_new() {
        let svc = create_test_service().await;
        // Should be able to subscribe
        let _rx = svc.subscribe();
    }

    #[tokio::test]
    async fn test_get_current_empty() {
        let svc = create_test_service().await;
        assert!(svc.get_current().is_empty());
    }

    #[tokio::test]
    async fn test_update_leaderboard_with_users() {
        let svc = create_test_service_with_users().await;
        svc.test_update_leaderboard().await;

        let lb = svc.get_current();
        assert_eq!(lb.len(), 3);

        // Should be sorted by net worth descending
        assert_eq!(lb[0].name, "Rich User");
        assert_eq!(lb[0].rank, 1);
        assert_eq!(lb[1].name, "Medium User");
        assert_eq!(lb[1].rank, 2);
        assert_eq!(lb[2].name, "Poor User");
        assert_eq!(lb[2].rank, 3);
    }

    #[tokio::test]
    async fn test_rank_change_calculation() {
        let repo = Arc::new(InMemoryUserRepository::new());
        let market = Arc::new(MarketService::new());

        // Create users
        let mut user1 = User::new(
            "REG001".to_string(),
            "User A".to_string(),
            "pass".to_string(),
        );
        user1.id = 1;
        user1.money = 200_000_000_000;

        let mut user2 = User::new(
            "REG002".to_string(),
            "User B".to_string(),
            "pass".to_string(),
        );
        user2.id = 2;
        user2.money = 100_000_000_000;

        repo.save(user1).await.unwrap();
        repo.save(user2.clone()).await.unwrap();

        let svc = LeaderboardService::new(repo.clone(), market);

        // First update - establishes initial rankings
        svc.test_update_leaderboard().await;
        let lb1 = svc.get_current();
        assert_eq!(lb1[0].name, "User A"); // Rank 1
        assert_eq!(lb1[1].name, "User B"); // Rank 2

        // Change money so User B becomes richer
        user2.money = 300_000_000_000;
        repo.save(user2).await.unwrap();

        // Second update - should show rank changes
        svc.test_update_leaderboard().await;
        let lb2 = svc.get_current();

        // User B should now be rank 1 with positive change
        assert_eq!(lb2[0].name, "User B");
        assert_eq!(lb2[0].rank, 1);
        assert_eq!(lb2[0].change_rank, 1); // Moved up from rank 2 to rank 1

        // User A should now be rank 2 with negative change
        assert_eq!(lb2[1].name, "User A");
        assert_eq!(lb2[1].rank, 2);
        assert_eq!(lb2[1].change_rank, -1); // Moved down from rank 1 to rank 2
    }

    #[tokio::test]
    async fn test_leaderboard_top_10_limit() {
        let repo = Arc::new(InMemoryUserRepository::new());
        let market = Arc::new(MarketService::new());

        // Create 15 users
        for i in 0..15 {
            let mut user = User::new(
                format!("REG{:03}", i),
                format!("User {}", i),
                "pass".to_string(),
            );
            user.id = i as u64;
            user.money = (100 - i as i64) * 1_000_000_000; // Decreasing money
            repo.save(user).await.unwrap();
        }

        let svc = LeaderboardService::new(repo, market);
        svc.test_update_leaderboard().await;

        let lb = svc.get_current();
        assert_eq!(lb.len(), 10); // Only top 10

        // Should be in order
        for (i, entry) in lb.iter().enumerate() {
            assert_eq!(entry.rank, i + 1);
        }
    }

    #[tokio::test]
    async fn test_net_worth_includes_portfolio() {
        let repo = Arc::new(InMemoryUserRepository::new());
        let market = Arc::new(MarketService::new());

        // Create a trade to establish market price
        use crate::domain::models::Trade;
        use chrono::Utc;
        market.test_process_trade(Trade {
            id: 1,
            symbol: "AAPL".to_string(),
            price: 1500000, // $150.00
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        });

        // Create user with portfolio
        let mut user = User::new(
            "REG001".to_string(),
            "Stock Holder".to_string(),
            "pass".to_string(),
        );
        user.id = 1;
        user.money = 10_000_000_000; // $1M cash
        user.portfolio = vec![Portfolio {
            user_id: 1,
            symbol: "AAPL".to_string(),
            qty: 1000, // 1000 shares at $150 = $150,000
            short_qty: 0,
            locked_qty: 0,
            average_buy_price: 1000000,
        }];

        repo.save(user).await.unwrap();

        let svc = LeaderboardService::new(repo, market);
        svc.test_update_leaderboard().await;

        let lb = svc.get_current();
        assert_eq!(lb.len(), 1);

        // Net worth should be cash + portfolio value
        // $1,000,000 cash + 1000 * $150 = $1,150,000 = 11,500,000,000 scaled
        let expected = 10_000_000_000 + (1000 * 1500000);
        assert_eq!(lb[0].net_worth, expected);
    }

    #[test]
    fn test_leaderboard_entry_struct() {
        let entry = LeaderboardEntry {
            rank: 1,
            name: "Test User".to_string(),
            net_worth: 1000000,
        };
        assert_eq!(entry.rank, 1);
        assert_eq!(entry.name, "Test User");
        assert_eq!(entry.net_worth, 1000000);
    }
}
