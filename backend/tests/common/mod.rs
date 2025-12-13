//! Common test utilities and infrastructure for integration tests.
//!
//! This module provides reusable test helpers including:
//! - Test state creation
//! - Mock WebSocket client
//! - Test data generators
//! - Assertion helpers

use std::sync::Arc;

use stockmart_backend::api::ws::AppState;
use stockmart_backend::config::ConfigService;
use stockmart_backend::domain::models::{Company, User};
use stockmart_backend::domain::trading::order::{OrderSide, OrderStatus, OrderType, TimeInForce};
use stockmart_backend::domain::trading::order_entity::Order;
use stockmart_backend::domain::{CompanyRepository, UserRepository};
use stockmart_backend::infrastructure::id_generator::IdGenerators;
use stockmart_backend::infrastructure::persistence::{
    InMemoryCompanyRepository, InMemoryUserRepository,
};
use stockmart_backend::service::admin::AdminService;
use stockmart_backend::service::chat::ChatService;
use stockmart_backend::service::engine::MatchingEngine;
use stockmart_backend::service::event_log::EventLogger;
use stockmart_backend::service::indices::IndicesService;
use stockmart_backend::service::leaderboard::LeaderboardService;
use stockmart_backend::service::market::MarketService;
use stockmart_backend::service::news::NewsService;
use stockmart_backend::service::orders::OrdersService;
use stockmart_backend::service::session::SessionManager;
use stockmart_backend::service::token::TokenService;
use stockmart_backend::service::trade_history::TradeHistoryService;

// =============================================================================
// TEST STATE CREATION
// =============================================================================

/// Create a fully initialized test AppState with all services.
///
/// This creates an isolated test environment with:
/// - In-memory repositories
/// - All services initialized
/// - No background tasks running (for deterministic testing)
pub async fn create_test_state() -> Arc<AppState> {
    create_test_state_with_config(TestConfig::default()).await
}

/// Configuration options for test state creation
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub max_sessions_per_user: u32,
    pub data_dir: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_user: 1,
            data_dir: "/tmp/stockmart_test".to_string(),
        }
    }
}

/// Create test state with custom configuration
pub async fn create_test_state_with_config(config: TestConfig) -> Arc<AppState> {
    // Initialize Config Service
    let config_service = Arc::new(ConfigService::new(config.data_dir.clone()));

    // Initialize Session Manager
    let session_manager = Arc::new(SessionManager::new(config.max_sessions_per_user));

    // Initialize Token Service
    let token_service = Arc::new(TokenService::new(config.max_sessions_per_user));

    // Initialize Repositories
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
    let company_repo: Arc<dyn CompanyRepository> = Arc::new(InMemoryCompanyRepository::new());

    // Initialize Orders Service
    let orders_service = Arc::new(OrdersService::new());

    // Initialize Trade History Service
    let trade_history_service = Arc::new(TradeHistoryService::new());

    // Initialize Engine
    let engine = Arc::new(MatchingEngine::new(
        user_repo.clone(),
        orders_service.clone(),
        trade_history_service.clone(),
    ));

    // Initialize Market Service (without background task)
    let market_service = Arc::new(MarketService::new());

    // Initialize Indices Service (without background task)
    let indices_service = Arc::new(IndicesService::new(
        market_service.clone(),
        company_repo.clone(),
    ));

    // Initialize News Service (without background task)
    let news_service = Arc::new(NewsService::new(company_repo.clone()));

    // Initialize Leaderboard Service (without background task)
    let leaderboard_service = Arc::new(LeaderboardService::new(
        user_repo.clone(),
        market_service.clone(),
    ));

    // Initialize Admin Service
    let admin_service = Arc::new(AdminService::new(
        engine.clone(),
        company_repo.clone(),
        user_repo.clone(),
    ));

    // Initialize Chat Service
    let chat_service = Arc::new(ChatService::new());

    // Initialize Event Logger (disabled for tests)
    let event_logger = Arc::new(EventLogger::new(&config.data_dir, true));

    let server_start_time = chrono::Utc::now().timestamp();

    Arc::new(AppState {
        engine,
        market: market_service,
        admin: admin_service,
        indices: indices_service,
        news: news_service,
        leaderboard: leaderboard_service,
        chat: chat_service,
        user_repo,
        company_repo,
        config: config_service,
        sessions: session_manager,
        event_log: event_logger,
        orders: orders_service,
        trade_history: trade_history_service,
        tokens: token_service,
        server_start_time,
    })
}

// =============================================================================
// TEST USER HELPERS
// =============================================================================

/// Create a test user and return their user_id
pub async fn create_test_user(state: &AppState, regno: &str, name: &str, password: &str) -> u64 {
    let user = User::new(regno.to_string(), name.to_string(), password.to_string());
    let user_id = user.id;
    state
        .user_repo
        .save(user)
        .await
        .expect("Failed to save test user");
    user_id
}

/// Create a test user with initial portfolio
pub async fn create_test_user_with_portfolio(
    state: &AppState,
    regno: &str,
    name: &str,
    money: i64,
    portfolio: Vec<(String, u64)>, // (symbol, qty)
) -> u64 {
    let mut user = User::new(regno.to_string(), name.to_string(), "password".to_string());
    user.money = money;

    for (symbol, qty) in portfolio {
        user.portfolio
            .push(stockmart_backend::domain::models::Portfolio {
                user_id: user.id,
                symbol,
                qty,
                short_qty: 0,
                locked_qty: 0,
                average_buy_price: 100 * 10_000, // $100
            });
    }

    let user_id = user.id;
    state
        .user_repo
        .save(user)
        .await
        .expect("Failed to save test user");
    user_id
}

/// Create an admin user
pub async fn create_admin_user(state: &AppState, regno: &str, name: &str) -> u64 {
    let mut user = User::new(regno.to_string(), name.to_string(), "admin".to_string());
    user.role = stockmart_backend::domain::user::role::Role::Admin;
    let user_id = user.id;
    state
        .user_repo
        .save(user)
        .await
        .expect("Failed to save admin user");
    user_id
}

// =============================================================================
// TEST COMPANY HELPERS
// =============================================================================

/// Create a test company and return its symbol
pub async fn create_test_company(state: &AppState, symbol: &str, name: &str) -> String {
    let company = Company {
        id: IdGenerators::global().next_company_id(),
        symbol: symbol.to_string(),
        name: name.to_string(),
        sector: "Test".to_string(),
        total_shares: 1_000_000,
        bankrupt: false,
        price_precision: 2,
        volatility: 10,
    };

    state
        .company_repo
        .save(company)
        .await
        .expect("Failed to save test company");
    state.engine.create_orderbook(symbol.to_string());

    symbol.to_string()
}

/// Create multiple test companies
pub async fn create_test_companies(state: &AppState, count: usize) -> Vec<String> {
    let mut symbols = Vec::new();
    let names = [
        "AAPL", "GOOGL", "MSFT", "AMZN", "META", "TSLA", "NVDA", "JPM", "V", "JNJ",
    ];
    let full_names = [
        "Apple Inc.",
        "Alphabet Inc.",
        "Microsoft Corp.",
        "Amazon.com Inc.",
        "Meta Platforms",
        "Tesla Inc.",
        "NVIDIA Corp.",
        "JPMorgan Chase",
        "Visa Inc.",
        "Johnson & Johnson",
    ];

    for i in 0..count.min(names.len()) {
        let symbol = create_test_company(state, names[i], full_names[i]).await;
        symbols.push(symbol);
    }

    symbols
}

// =============================================================================
// PRICE HELPERS
// =============================================================================

/// Convert dollars to scaled price (multiply by PRICE_SCALE)
pub fn dollars(amount: i64) -> i64 {
    amount * 10_000
}

/// Convert scaled price to dollars
pub fn to_dollars(scaled: i64) -> f64 {
    scaled as f64 / 10_000.0
}

// =============================================================================
// ASSERTION HELPERS
// =============================================================================

/// Assert that a user has specific money amount
pub async fn assert_user_money(state: &AppState, user_id: u64, expected: i64) {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User not found");
    assert_eq!(
        user.money,
        expected,
        "User {} money mismatch: expected ${}, got ${}",
        user_id,
        to_dollars(expected),
        to_dollars(user.money)
    );
}

/// Assert that a user has specific locked money amount
pub async fn assert_user_locked_money(state: &AppState, user_id: u64, expected: i64) {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User not found");
    assert_eq!(
        user.locked_money,
        expected,
        "User {} locked_money mismatch: expected ${}, got ${}",
        user_id,
        to_dollars(expected),
        to_dollars(user.locked_money)
    );
}

/// Assert that a user has specific portfolio position
pub async fn assert_user_position(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    expected_qty: u64,
    expected_locked: u64,
) {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User not found");

    let position = user.portfolio.iter().find(|p| p.symbol == symbol);

    match position {
        Some(p) => {
            assert_eq!(
                p.qty, expected_qty,
                "User {} position {} qty mismatch: expected {}, got {}",
                user_id, symbol, expected_qty, p.qty
            );
            assert_eq!(
                p.locked_qty, expected_locked,
                "User {} position {} locked_qty mismatch: expected {}, got {}",
                user_id, symbol, expected_locked, p.locked_qty
            );
        }
        None if expected_qty == 0 && expected_locked == 0 => {
            // No position is fine if we expect 0
        }
        None => {
            panic!(
                "User {} has no position in {}, expected qty={}",
                user_id, symbol, expected_qty
            );
        }
    }
}

/// Assert order book depth at specific price level
pub fn assert_book_level(
    state: &AppState,
    symbol: &str,
    side: &str, // "bid" or "ask"
    level: usize,
    expected_price: i64,
    expected_qty: u64,
) {
    let depth = state.engine.get_order_book_depth(symbol, level + 1);
    match depth {
        Some((bids, asks)) => {
            let levels = if side == "bid" { &bids } else { &asks };
            if level < levels.len() {
                let (price, qty) = levels[level];
                assert_eq!(
                    price,
                    expected_price,
                    "Book {} {} level {} price mismatch: expected ${}, got ${}",
                    symbol,
                    side,
                    level,
                    to_dollars(expected_price),
                    to_dollars(price)
                );
                assert_eq!(
                    qty, expected_qty,
                    "Book {} {} level {} qty mismatch: expected {}, got {}",
                    symbol, side, level, expected_qty, qty
                );
            } else {
                panic!(
                    "Book {} has fewer than {} {} levels",
                    symbol,
                    level + 1,
                    side
                );
            }
        }
        None => panic!("No order book found for {}", symbol),
    }
}

/// Assert that order book is empty for a symbol
pub fn assert_book_empty(state: &AppState, symbol: &str) {
    let depth = state.engine.get_order_book_depth(symbol, 1);
    match depth {
        Some((bids, asks)) => {
            assert!(bids.is_empty(), "Expected empty bids for {}", symbol);
            assert!(asks.is_empty(), "Expected empty asks for {}", symbol);
        }
        None => panic!("No order book found for {}", symbol),
    }
}

// =============================================================================
// ORDER HELPERS
// =============================================================================

/// Create an Order object with the given parameters
fn create_order(
    user_id: u64,
    symbol: &str,
    order_type: OrderType,
    side: OrderSide,
    qty: u64,
    price: i64,
) -> Order {
    Order {
        id: IdGenerators::global().next_order_id(),
        user_id,
        symbol: symbol.to_string(),
        order_type,
        side,
        qty,
        filled_qty: 0,
        price,
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    }
}

/// Place a limit buy order and return the order ID
pub async fn place_limit_buy(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    qty: u64,
    price: i64,
) -> Result<u64, String> {
    let order = create_order(
        user_id,
        symbol,
        OrderType::Limit,
        OrderSide::Buy,
        qty,
        price,
    );
    state
        .engine
        .place_order(order)
        .await
        .map(|o| o.id)
        .map_err(|e| e.to_string())
}

/// Place a limit sell order and return the order ID
pub async fn place_limit_sell(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    qty: u64,
    price: i64,
) -> Result<u64, String> {
    let order = create_order(
        user_id,
        symbol,
        OrderType::Limit,
        OrderSide::Sell,
        qty,
        price,
    );
    state
        .engine
        .place_order(order)
        .await
        .map(|o| o.id)
        .map_err(|e| e.to_string())
}

/// Place a market buy order
pub async fn place_market_buy(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    qty: u64,
) -> Result<u64, String> {
    let order = create_order(
        user_id,
        symbol,
        OrderType::Market,
        OrderSide::Buy,
        qty,
        i64::MAX / 2,
    );
    state
        .engine
        .place_order(order)
        .await
        .map(|o| o.id)
        .map_err(|e| e.to_string())
}

/// Place a market sell order
pub async fn place_market_sell(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    qty: u64,
) -> Result<u64, String> {
    let order = create_order(user_id, symbol, OrderType::Market, OrderSide::Sell, qty, 1);
    state
        .engine
        .place_order(order)
        .await
        .map(|o| o.id)
        .map_err(|e| e.to_string())
}

/// Place a short sell order
#[allow(dead_code)]
pub async fn place_short(
    state: &AppState,
    user_id: u64,
    symbol: &str,
    qty: u64,
    price: i64,
) -> Result<u64, String> {
    let order = create_order(
        user_id,
        symbol,
        OrderType::Limit,
        OrderSide::Short,
        qty,
        price,
    );
    state
        .engine
        .place_order(order)
        .await
        .map(|o| o.id)
        .map_err(|e| e.to_string())
}

// =============================================================================
// MARKET HELPERS
// =============================================================================

/// Open the market for trading
pub fn open_market(state: &AppState) {
    state.engine.set_market_open(true);
}

/// Close the market
pub fn close_market(state: &AppState) {
    state.engine.set_market_open(false);
}

/// Check if market is open
pub fn is_market_open(state: &AppState) -> bool {
    state.engine.is_market_open()
}

// =============================================================================
// TRADE SUBSCRIPTION HELPERS
// =============================================================================

/// Subscribe to trades and collect them
pub struct TradeCollector {
    receiver: tokio::sync::broadcast::Receiver<stockmart_backend::domain::models::Trade>,
    trades: Vec<stockmart_backend::domain::models::Trade>,
}

impl TradeCollector {
    pub fn new(state: &AppState) -> Self {
        Self {
            receiver: state.engine.subscribe_trades(),
            trades: Vec::new(),
        }
    }

    /// Collect all pending trades (non-blocking)
    pub fn collect(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(trade) => self.trades.push(trade),
                Err(_) => break,
            }
        }
    }

    /// Get collected trades
    pub fn trades(&self) -> &[stockmart_backend::domain::models::Trade] {
        &self.trades
    }

    /// Get trade count
    pub fn count(&self) -> usize {
        self.trades.len()
    }

    /// Clear collected trades
    pub fn clear(&mut self) {
        self.trades.clear();
    }
}

// =============================================================================
// INVARIANT CHECKERS
// =============================================================================

/// Check that user money is non-negative
pub async fn check_money_invariant(state: &AppState, user_id: u64) -> Result<(), String> {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    if user.money < 0 {
        return Err(format!(
            "Invariant violation: user {} has negative money: {}",
            user_id, user.money
        ));
    }
    if user.locked_money < 0 {
        return Err(format!(
            "Invariant violation: user {} has negative locked_money: {}",
            user_id, user.locked_money
        ));
    }

    Ok(())
}

/// Check that locked_qty <= qty for all positions
pub async fn check_position_invariant(state: &AppState, user_id: u64) -> Result<(), String> {
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    for position in &user.portfolio {
        if position.locked_qty > position.qty {
            return Err(format!(
                "Invariant violation: user {} position {} has locked_qty ({}) > qty ({})",
                user_id, position.symbol, position.locked_qty, position.qty
            ));
        }
    }

    Ok(())
}

/// Check order book invariant (bids sorted desc, asks sorted asc, no crossing)
pub fn check_book_invariant(state: &AppState, symbol: &str) -> Result<(), String> {
    let depth = state.engine.get_order_book_depth(symbol, 100);

    match depth {
        Some((bids, asks)) => {
            // Check bids sorted descending
            for i in 1..bids.len() {
                if bids[i].0 > bids[i - 1].0 {
                    return Err(format!(
                        "Invariant violation: {} bids not sorted descending at level {}",
                        symbol, i
                    ));
                }
            }

            // Check asks sorted ascending
            for i in 1..asks.len() {
                if asks[i].0 < asks[i - 1].0 {
                    return Err(format!(
                        "Invariant violation: {} asks not sorted ascending at level {}",
                        symbol, i
                    ));
                }
            }

            // Check no crossing (best bid < best ask)
            if let (Some((best_bid, _)), Some((best_ask, _))) = (bids.first(), asks.first()) {
                if best_bid >= best_ask {
                    return Err(format!(
                        "Invariant violation: {} has crossed book (bid {} >= ask {})",
                        symbol, best_bid, best_ask
                    ));
                }
            }

            Ok(())
        }
        None => Err(format!("No order book found for {}", symbol)),
    }
}
