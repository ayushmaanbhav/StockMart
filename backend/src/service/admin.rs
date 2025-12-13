//! Admin service for administrative operations.

#![allow(dead_code)] // Service API includes utility methods for future admin features

use crate::domain::models::{
    Order, OrderSide, OrderStatus, OrderType, Portfolio, TimeInForce, PRICE_SCALE,
};
use crate::domain::{CompanyRepository, UserRepository};
use crate::infrastructure::id_generator::IdGenerators;
use crate::service::engine::MatchingEngine;
use rand::Rng;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct AdminService {
    engine: Arc<MatchingEngine>,
    company_repo: Arc<dyn CompanyRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl AdminService {
    pub fn new(
        engine: Arc<MatchingEngine>,
        company_repo: Arc<dyn CompanyRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            engine,
            company_repo,
            user_repo,
        }
    }

    pub fn toggle_market(&self, open: bool) {
        self.engine.set_market_open(open);
    }

    pub async fn set_company_volatility(
        &self,
        symbol: &str,
        volatility: i64,
    ) -> Result<(), String> {
        if let Some(mut company) = self
            .company_repo
            .find_by_symbol(symbol)
            .await
            .map_err(|e| e.to_string())?
        {
            company.volatility = volatility;
            self.company_repo
                .save(company)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Company not found".to_string())
        }
    }

    pub async fn set_company_bankrupt(&self, symbol: &str, bankrupt: bool) -> Result<(), String> {
        if let Some(mut company) = self
            .company_repo
            .find_by_symbol(symbol)
            .await
            .map_err(|e| e.to_string())?
        {
            company.bankrupt = bankrupt;
            self.company_repo
                .save(company)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Company not found".to_string())
        }
    }

    pub async fn create_company(
        &self,
        symbol: String,
        name: String,
        sector: String,
        volatility: i64,
    ) -> Result<(), String> {
        // Check if symbol already exists
        if self
            .company_repo
            .symbol_exists(&symbol)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err(format!("Symbol {} already exists", symbol));
        }

        let company = crate::domain::models::Company {
            id: IdGenerators::global().next_company_id(),
            symbol: symbol.clone(),
            name,
            sector,
            total_shares: 1_000_000, // Default IPO shares
            bankrupt: false,
            price_precision: 2,
            volatility,
        };

        self.company_repo
            .create(company)
            .await
            .map_err(|e| e.to_string())?;
        self.engine.create_orderbook(symbol);
        Ok(())
    }

    /// Initialize or reset the game
    /// - Resets all traders to equal starting net worth
    /// - Starting cash is approximately half of net worth
    /// - Other half is randomly allocated shares from companies
    /// - All traders end up with the same total net worth
    pub async fn init_game(
        &self,
        target_networth: i64,
        shares_per_trader_per_company: u64,
    ) -> Result<String, String> {
        info!("=== GAME INITIALIZATION STARTED ===");
        info!(
            "Target net worth per trader: ${}",
            target_networth / PRICE_SCALE
        );
        info!(
            "Base shares per company per trader: {}",
            shares_per_trader_per_company
        );

        // Close market during initialization
        self.engine.set_market_open(false);
        debug!("Market closed for initialization");

        // Get all users and companies
        let mut users = self.user_repo.all().await.map_err(|e| e.to_string())?;
        let companies = self.company_repo.all().await.map_err(|e| e.to_string())?;

        if users.is_empty() {
            warn!("No traders registered - cannot initialize game");
            return Err("No traders registered".to_string());
        }

        if companies.is_empty() {
            warn!("No companies configured - cannot initialize game");
            return Err("No companies configured".to_string());
        }

        let num_users = users.len();
        let num_companies = companies.len();
        info!("Found {} users, {} companies", num_users, num_companies);

        // Base price for all stocks at game start
        let base_price: i64 = 100 * PRICE_SCALE; // $100.00 per share

        // Calculate target portfolio value (half of net worth in shares)
        let target_portfolio_value = target_networth / 2;
        let target_cash = target_networth - target_portfolio_value;

        debug!(
            "Target cash: ${}, Target portfolio value: ${}",
            target_cash / PRICE_SCALE,
            target_portfolio_value / PRICE_SCALE
        );

        // Pre-generate random variances for all users and companies (before async)
        // We'll adjust cash to ensure equal net worth despite random share allocation
        let variances: Vec<Vec<i64>> = {
            let mut rng = rand::thread_rng();
            users
                .iter()
                .map(|_| {
                    companies
                        .iter()
                        .map(|_| rng.gen_range(-20i64..=20i64))
                        .collect()
                })
                .collect()
        };

        let mut trader_count = 0;

        // For each user, reset their portfolio with equal net worth
        for (user_idx, user) in users.iter_mut().enumerate() {
            // Skip admin users (using RBAC role check)
            if user.is_admin() {
                debug!("Skipping admin user: {} (id={})", user.name, user.id);
                continue;
            }

            trader_count += 1;
            debug!("Processing trader: {} (id={})", user.name, user.id);

            // Clear existing portfolio and locked amounts
            user.portfolio.clear();
            user.locked_money = 0;
            user.margin_locked = 0;

            // Allocate random shares from each company
            let mut total_portfolio_value: i64 = 0;

            for (company_idx, company) in companies.iter().enumerate() {
                // Add some randomness to share allocation (+/- 20%)
                let variance = variances[user_idx][company_idx];
                let adjusted_shares =
                    ((shares_per_trader_per_company as i64 * (100 + variance)) / 100) as u64;
                let final_shares = adjusted_shares.max(1); // At least 1 share

                let share_value = (final_shares as i64) * base_price;
                total_portfolio_value += share_value;

                user.portfolio.push(Portfolio {
                    user_id: user.id,
                    symbol: company.symbol.clone(),
                    qty: final_shares,
                    short_qty: 0,
                    locked_qty: 0,
                    average_buy_price: base_price, // $100.00 per share at start
                });

                debug!(
                    "  {} allocated {} shares = ${}",
                    company.symbol,
                    final_shares,
                    share_value / PRICE_SCALE
                );
            }

            // Calculate cash needed to reach target net worth
            // Net worth = cash + portfolio_value
            // So cash = target_networth - portfolio_value
            let calculated_cash = target_networth - total_portfolio_value;
            user.money = calculated_cash.max(0); // Ensure non-negative cash

            let actual_networth = user.money + total_portfolio_value;
            info!(
                "Trader {} (id={}): cash=${}, portfolio=${}, networth=${}",
                user.name,
                user.id,
                user.money / PRICE_SCALE,
                total_portfolio_value / PRICE_SCALE,
                actual_networth / PRICE_SCALE
            );

            // Save updated user
            self.user_repo
                .save(user.clone())
                .await
                .map_err(|e| e.to_string())?;
        }

        // Clear all order books
        for company in &companies {
            self.engine.clear_orderbook(&company.symbol);
            debug!("Cleared orderbook for {}", company.symbol);
        }

        // Seed initial orders to create liquidity and establish market prices
        // This creates synthetic bid/ask spreads around a base price of $100
        let mut rng = rand::thread_rng();
        debug!("Seeding initial liquidity...");

        for company in &companies {
            // Create 5 bid levels (buy orders) below market price
            for i in 1..=5 {
                let price = base_price - (i as i64 * 50 * 100); // $0.50 decrements
                let qty = rng.gen_range(50..200); // Random qty 50-200

                let order = Order {
                    id: IdGenerators::global().next_order_id(),
                    user_id: 1, // Admin places these orders
                    symbol: company.symbol.clone(),
                    order_type: OrderType::Limit,
                    side: OrderSide::Buy,
                    qty,
                    filled_qty: 0,
                    price,
                    status: OrderStatus::Open,
                    timestamp: chrono::Utc::now().timestamp(),
                    time_in_force: TimeInForce::GTC,
                };
                self.engine.seed_order(order);
            }

            // Create 5 ask levels (sell orders) above market price
            for i in 1..=5 {
                let price = base_price + (i as i64 * 50 * 100); // $0.50 increments
                let qty = rng.gen_range(50..200); // Random qty 50-200

                let order = Order {
                    id: IdGenerators::global().next_order_id(),
                    user_id: 1, // Admin places these orders
                    symbol: company.symbol.clone(),
                    order_type: OrderType::Limit,
                    side: OrderSide::Sell,
                    qty,
                    filled_qty: 0,
                    price,
                    status: OrderStatus::Open,
                    timestamp: chrono::Utc::now().timestamp(),
                    time_in_force: TimeInForce::GTC,
                };
                self.engine.seed_order(order);
            }
            debug!("  {} seeded with 5 bid and 5 ask levels", company.symbol);
        }

        let summary = format!(
            "Game initialized: {} traders with ${} target net worth each (~${} cash + ~${} in {} companies). Order books seeded.",
            trader_count,
            target_networth / PRICE_SCALE,
            target_networth / PRICE_SCALE / 2,
            target_networth / PRICE_SCALE / 2,
            num_companies
        );

        info!("=== GAME INITIALIZATION COMPLETE ===");
        info!("{}", summary);

        Ok(summary)
    }

    /// Get all traders for admin view
    pub async fn get_all_traders(&self) -> Result<Vec<crate::domain::models::User>, String> {
        self.user_repo.all().await.map_err(|e| e.to_string())
    }

    /// Ban/unban a trader
    pub async fn set_trader_banned(&self, user_id: u64, banned: bool) -> Result<(), String> {
        if let Some(mut user) = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| e.to_string())?
        {
            user.banned = banned;
            self.user_repo.save(user).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    /// Enable/disable chat for a trader
    pub async fn set_trader_chat(&self, user_id: u64, enabled: bool) -> Result<(), String> {
        if let Some(mut user) = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| e.to_string())?
        {
            user.chat_enabled = enabled;
            self.user_repo.save(user).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
}
