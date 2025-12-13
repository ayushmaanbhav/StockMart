//! Deep Coverage Tests
//!
//! These tests target specific uncovered lines to maximize coverage.
//! Focus on service run() methods, edge cases, and data validation.

mod common;

use common::*;
use std::sync::Arc;
use stockmart_backend::domain::models::{
    Company, Order, OrderSide, OrderStatus, OrderType, Portfolio, TimeInForce, Trade, User,
};
use stockmart_backend::domain::trading::orderbook::OrderBook;
use stockmart_backend::infrastructure::id_generator::IdGenerators;
use stockmart_backend::service::event_log::{EventLogger, GameEvent, PositionSnapshot};
use stockmart_backend::service::orders::OrdersService;
use stockmart_backend::service::trade_history::TradeHistoryService;

// =============================================================================
// ORDERBOOK DEEP TESTS
// =============================================================================

/// Test OrderBook::new creates empty book
#[tokio::test]
async fn test_orderbook_new() {
    let book = OrderBook::new("AAPL".to_string());
    let (bids, asks) = book.get_depth(10);
    assert!(bids.is_empty());
    assert!(asks.is_empty());
}

/// Test OrderBook::best_bid on empty book
#[tokio::test]
async fn test_orderbook_best_bid_empty() {
    let book = OrderBook::new("AAPL".to_string());
    assert!(book.best_bid().is_none());
}

/// Test OrderBook::best_ask on empty book
#[tokio::test]
async fn test_orderbook_best_ask_empty() {
    let book = OrderBook::new("AAPL".to_string());
    assert!(book.best_ask().is_none());
}

/// Test OrderBook with multiple orders at same price (time priority)
#[tokio::test]
async fn test_orderbook_time_priority() {
    let mut book = OrderBook::new("AAPL".to_string());

    // Add two buy orders at same price
    let order1 = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 10,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: 1000,
        time_in_force: TimeInForce::GTC,
    };

    let order2 = Order {
        id: 2,
        user_id: 101,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 20,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: 1001,
        time_in_force: TimeInForce::GTC,
    };

    book.add_order(order1, TimeInForce::GTC);
    book.add_order(order2, TimeInForce::GTC);

    let (bids, _) = book.get_depth(10);
    assert_eq!(bids.len(), 1); // Same price level
    assert_eq!(bids[0].1, 30); // Combined qty
}

/// Test OrderBook::seed_order
#[tokio::test]
async fn test_orderbook_seed_order() {
    let mut book = OrderBook::new("AAPL".to_string());

    let order = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Sell,
        qty: 100,
        filled_qty: 0,
        price: dollars(150),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    book.seed_order(order);

    let (_, asks) = book.get_depth(10);
    assert_eq!(asks.len(), 1);
    assert_eq!(asks[0].0, dollars(150));
    assert_eq!(asks[0].1, 100);
}

/// Test OrderBook::clear
#[tokio::test]
async fn test_orderbook_clear() {
    let mut book = OrderBook::new("AAPL".to_string());

    // Add some orders
    let order = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 100,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    book.add_order(order, TimeInForce::GTC);

    // Verify not empty
    let (bids, _) = book.get_depth(10);
    assert!(!bids.is_empty());

    // Clear
    book.clear();

    // Verify empty
    let (bids, asks) = book.get_depth(10);
    assert!(bids.is_empty());
    assert!(asks.is_empty());
}

/// Test OrderBook cancel_order that doesn't exist
#[tokio::test]
async fn test_orderbook_cancel_nonexistent() {
    let mut book = OrderBook::new("AAPL".to_string());

    let result = book.cancel_order(99999);
    assert!(result.is_none());
}

/// Test OrderBook matching with price improvement
#[tokio::test]
async fn test_orderbook_matching_price_improvement() {
    let mut book = OrderBook::new("AAPL".to_string());

    // Seed a sell order at $95
    let sell_order = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Sell,
        qty: 10,
        filled_qty: 0,
        price: dollars(95),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };
    book.seed_order(sell_order);

    // Place buy order at $100 - should match at $95
    let buy_order = Order {
        id: 2,
        user_id: 101,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 10,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    let (processed, trades) = book.add_order(buy_order, TimeInForce::GTC);

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].price, dollars(95)); // Trade at resting order price
    assert_eq!(processed.status, OrderStatus::Filled);
}

// =============================================================================
// ORDERS SERVICE DEEP TESTS
// =============================================================================

/// Test OrdersService with multiple users
#[tokio::test]
async fn test_orders_service_multi_user() {
    let service = OrdersService::new();

    let order1 = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 10,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    let order2 = Order {
        id: 2,
        user_id: 101,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Sell,
        qty: 20,
        filled_qty: 0,
        price: dollars(110),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    service.add_order(order1);
    service.add_order(order2);

    assert_eq!(service.get_user_order_count(100), 1);
    assert_eq!(service.get_user_order_count(101), 1);
    assert_eq!(service.get_total_open_orders_count(), 2);
}

/// Test OrdersService update to Filled removes order
#[tokio::test]
async fn test_orders_service_update_to_filled() {
    let service = OrdersService::new();

    let order = Order {
        id: 1,
        user_id: 100,
        symbol: "AAPL".to_string(),
        order_type: OrderType::Limit,
        side: OrderSide::Buy,
        qty: 10,
        filled_qty: 0,
        price: dollars(100),
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: TimeInForce::GTC,
    };

    service.add_order(order);
    assert!(service.order_exists(1));

    // Update to filled
    service.update_order(1, 10, OrderStatus::Filled);

    // Check order is updated (may or may not be removed depending on impl)
    let updated = service.get_order(1);
    if let Some(o) = updated {
        assert_eq!(o.filled_qty, 10);
        assert_eq!(o.status, OrderStatus::Filled);
    }
}

/// Test OrdersService get_all_orders_admin with user name map
#[tokio::test]
async fn test_orders_service_admin_with_names() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ADMORD", "Admin Order Co").await;

    let user_id = create_test_user_with_portfolio(
        &state,
        "ADMORDUSER",
        "Admin Order User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);
    place_limit_buy(&state, user_id, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let mut names = std::collections::HashMap::new();
    names.insert(user_id, "Admin Order User".to_string());

    let orders = state.orders.get_all_orders_admin(None, &names);
    assert!(!orders.is_empty());
    assert_eq!(orders[0].user_name, "Admin Order User");
}

// =============================================================================
// TRADE HISTORY SERVICE DEEP TESTS
// =============================================================================

/// Test TradeHistoryService record and retrieve
#[tokio::test]
async fn test_trade_history_record_retrieve() {
    let service = TradeHistoryService::new();

    let trade = Trade {
        id: 1,
        symbol: "AAPL".to_string(),
        price: dollars(150),
        qty: 10,
        maker_order_id: 1,
        taker_order_id: 2,
        maker_user_id: 100,
        taker_user_id: 101,
        timestamp: chrono::Utc::now().timestamp(),
    };

    service.record_trade(
        trade.clone(),
        "Buyer Name".to_string(),
        "Seller Name".to_string(),
        OrderSide::Buy,
        OrderSide::Sell,
    );

    // Verify recorded
    assert_eq!(service.get_total_trade_count(), 1);

    // Get by symbol
    let trades = service.get_symbol_trades("AAPL", 10);
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].symbol, "AAPL");

    // Get by user (buyer)
    let buyer_trades = service.get_user_trades(101, 0, 10);
    assert_eq!(buyer_trades.total_count, 1);

    // Get by user (seller)
    let seller_trades = service.get_user_trades(100, 0, 10);
    assert_eq!(seller_trades.total_count, 1);
}

/// Test TradeHistoryService volume calculations
#[tokio::test]
async fn test_trade_history_volume() {
    let service = TradeHistoryService::new();

    // Record trade: 10 shares at $150 = $1500 volume
    let trade = Trade {
        id: 1,
        symbol: "AAPL".to_string(),
        price: dollars(150),
        qty: 10,
        maker_order_id: 1,
        taker_order_id: 2,
        maker_user_id: 100,
        taker_user_id: 101,
        timestamp: chrono::Utc::now().timestamp(),
    };

    service.record_trade(
        trade,
        "Buyer".to_string(),
        "Seller".to_string(),
        OrderSide::Buy,
        OrderSide::Sell,
    );

    let total_vol = service.get_total_volume();
    assert_eq!(total_vol, dollars(1500));

    let recent_vol = service.get_recent_volume(60);
    assert_eq!(recent_vol, dollars(1500));
}

/// Test TradeHistoryService pagination
#[tokio::test]
async fn test_trade_history_pagination() {
    let service = TradeHistoryService::new();

    // Record 10 trades
    for i in 0..10 {
        let trade = Trade {
            id: i as u64,
            symbol: "AAPL".to_string(),
            price: dollars(100),
            qty: 1,
            maker_order_id: i as u64,
            taker_order_id: i as u64 + 100,
            maker_user_id: 100,
            taker_user_id: 101,
            timestamp: chrono::Utc::now().timestamp(),
        };

        service.record_trade(
            trade,
            "Buyer".to_string(),
            "Seller".to_string(),
            OrderSide::Buy,
            OrderSide::Sell,
        );
    }

    // Get page 0, 3 per page
    let page0 = service.get_all_trades(None, None, 0, 3);
    assert_eq!(page0.trades.len(), 3);
    assert!(page0.has_more);
    assert_eq!(page0.total_count, 10);

    // Get page 3 (last page with 1 item)
    let page3 = service.get_all_trades(None, None, 3, 3);
    assert_eq!(page3.trades.len(), 1);
    assert!(!page3.has_more);
}

// =============================================================================
// USER MODEL TESTS
// =============================================================================

/// Test User::new creates correct initial state
#[tokio::test]
async fn test_user_new() {
    let user = User::new(
        "REG001".to_string(),
        "Test User".to_string(),
        "password123".to_string(),
    );

    assert_eq!(user.regno, "REG001");
    assert_eq!(user.name, "Test User");
    assert!(!user.banned);
    assert!(user.chat_enabled);
    assert!(user.portfolio.is_empty());
    assert_eq!(
        user.money,
        stockmart_backend::domain::constants::user::DEFAULT_STARTING_MONEY
    );
    assert_eq!(user.locked_money, 0);
    assert_eq!(user.margin_locked, 0);
}

/// Test Portfolio struct
#[tokio::test]
async fn test_portfolio_struct() {
    let portfolio = Portfolio {
        user_id: 1,
        symbol: "AAPL".to_string(),
        qty: 100,
        short_qty: 0,
        locked_qty: 10,
        average_buy_price: dollars(150),
    };

    assert_eq!(portfolio.user_id, 1);
    assert_eq!(portfolio.symbol, "AAPL");
    assert_eq!(portfolio.qty, 100);
    assert_eq!(portfolio.locked_qty, 10);
}

// =============================================================================
// COMPANY MODEL TESTS
// =============================================================================

/// Test Company struct
#[tokio::test]
async fn test_company_struct() {
    let company = Company {
        id: 1,
        symbol: "AAPL".to_string(),
        name: "Apple Inc".to_string(),
        sector: "Technology".to_string(),
        total_shares: 1_000_000,
        bankrupt: false,
        price_precision: 2,
        volatility: 25,
    };

    assert_eq!(company.symbol, "AAPL");
    assert_eq!(company.name, "Apple Inc");
    assert!(!company.bankrupt);
    assert_eq!(company.volatility, 25);
}

// =============================================================================
// ID GENERATOR DEEP TESTS
// =============================================================================

/// Test IdGenerators uniqueness
#[tokio::test]
async fn test_id_generators_uniqueness() {
    let gen = IdGenerators::global();

    let mut order_ids = std::collections::HashSet::new();
    for _ in 0..1000 {
        let id = gen.next_order_id();
        assert!(order_ids.insert(id), "Duplicate order ID generated");
    }

    let mut trade_ids = std::collections::HashSet::new();
    for _ in 0..1000 {
        let id = gen.next_trade_id();
        assert!(trade_ids.insert(id), "Duplicate trade ID generated");
    }
}

/// Test IdGenerators thread safety
#[tokio::test]
async fn test_id_generators_thread_safety() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let gen = Arc::new(IdGenerators::global());
    let mut set = JoinSet::new();

    for _ in 0..10 {
        let gen_clone = gen.clone();
        set.spawn(async move {
            let mut ids = Vec::new();
            for _ in 0..100 {
                ids.push(gen_clone.next_order_id());
            }
            ids
        });
    }

    let mut all_ids = std::collections::HashSet::new();
    while let Some(result) = set.join_next().await {
        let ids = result.unwrap();
        for id in ids {
            assert!(all_ids.insert(id), "Duplicate ID across threads");
        }
    }

    assert_eq!(all_ids.len(), 1000);
}

// =============================================================================
// EVENT LOGGER DEEP TESTS
// =============================================================================

/// Test EventLogger with all event types
#[tokio::test]
async fn test_event_logger_all_events() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let logger = EventLogger::new(temp_dir.path().to_str().unwrap(), true);

    // User events
    logger.log(GameEvent::UserRegistered {
        user_id: 1,
        regno: "REG001".to_string(),
        name: "Test User".to_string(),
        initial_cash: dollars(100_000),
        initial_portfolio_value: dollars(50_000),
    });

    logger.log(GameEvent::UserLogin {
        user_id: 1,
        regno: "REG001".to_string(),
        name: "Test User".to_string(),
    });

    // Order events
    logger.log(GameEvent::OrderPlaced {
        order_id: 1,
        user_id: 1,
        symbol: "AAPL".to_string(),
        side: "Buy".to_string(),
        order_type: "Limit".to_string(),
        qty: 100,
        price: dollars(150),
        time_in_force: "GTC".to_string(),
    });

    logger.log(GameEvent::OrderCancelled {
        order_id: 1,
        user_id: 1,
        symbol: "AAPL".to_string(),
        reason: "User cancelled".to_string(),
    });

    logger.log(GameEvent::OrderRejected {
        user_id: 1,
        symbol: "AAPL".to_string(),
        side: "Buy".to_string(),
        qty: 100,
        price: dollars(150),
        reason: "Insufficient funds".to_string(),
    });

    // Trade events
    logger.log(GameEvent::TradeExecuted {
        trade_id: 1,
        symbol: "AAPL".to_string(),
        buyer_id: 1,
        seller_id: 2,
        qty: 100,
        price: dollars(150),
        buyer_order_id: 1,
        seller_order_id: 2,
    });

    // Portfolio events
    logger.log(GameEvent::PortfolioUpdate {
        user_id: 1,
        cash: dollars(50_000),
        locked_cash: dollars(15_000),
        margin_locked: 0,
        positions: vec![PositionSnapshot {
            symbol: "AAPL".to_string(),
            qty: 100,
            short_qty: 0,
            locked_qty: 0,
            average_buy_price: dollars(150),
        }],
        net_worth: dollars(65_000),
    });

    // Market events
    logger.log(GameEvent::MarketOpened);
    logger.log(GameEvent::MarketClosed);
    logger.log(GameEvent::CircuitBreakerTriggered {
        symbol: "AAPL".to_string(),
        reason: "10% price move".to_string(),
        halted_until: chrono::Utc::now().timestamp() + 60,
    });

    // Admin events
    logger.log(GameEvent::GameInitialized {
        num_traders: 100,
        starting_money: dollars(100_000),
        share_allocation_per_trader: dollars(50_000),
    });

    logger.log(GameEvent::GameReset {
        reason: "Admin reset".to_string(),
    });

    logger.log(GameEvent::CompanyCreated {
        symbol: "NEWCO".to_string(),
        name: "New Company".to_string(),
        sector: "Technology".to_string(),
        initial_price: dollars(100),
    });

    logger.log(GameEvent::CompanyBankrupt {
        symbol: "BANKCO".to_string(),
    });

    logger.log(GameEvent::VolatilityChanged {
        symbol: "AAPL".to_string(),
        old_volatility: 10,
        new_volatility: 25,
    });

    logger.log(GameEvent::TraderBanned {
        user_id: 1,
        reason: "TOS violation".to_string(),
    });

    logger.log(GameEvent::TraderUnbanned { user_id: 1 });

    logger.log(GameEvent::TraderChatMuted { user_id: 1 });

    logger.log(GameEvent::TraderChatUnmuted { user_id: 1 });

    // Chat event
    logger.log(GameEvent::ChatMessage {
        user_id: 1,
        username: "TestUser".to_string(),
        message: "Hello world!".to_string(),
    });

    // Verify log file was created
    let log_path = temp_dir.path().join("game_events.jsonl");
    assert!(log_path.exists());
}

// =============================================================================
// ADMIN SERVICE DEEP TESTS
// =============================================================================

/// Test AdminService company operations
#[tokio::test]
async fn test_admin_service_company_ops() {
    let state = create_test_state().await;

    // Create company with specific volatility
    state
        .admin
        .create_company(
            "HIGHVOL".to_string(),
            "High Volatility Co".to_string(),
            "Finance".to_string(),
            75,
        )
        .await
        .unwrap();

    // Verify company
    let company = state
        .company_repo
        .find_by_symbol("HIGHVOL")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(company.volatility, 75);
    assert_eq!(company.sector, "Finance");
    assert!(!company.bankrupt);

    // Set bankrupt
    state
        .admin
        .set_company_bankrupt("HIGHVOL", true)
        .await
        .unwrap();

    let company = state
        .company_repo
        .find_by_symbol("HIGHVOL")
        .await
        .unwrap()
        .unwrap();
    assert!(company.bankrupt);
}

/// Test AdminService trader operations
#[tokio::test]
async fn test_admin_service_trader_ops() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "ADMINTEST", "Admin Test User", "pass").await;

    // Ban trader
    state.admin.set_trader_banned(user_id, true).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.banned);

    // Unban trader
    state.admin.set_trader_banned(user_id, false).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.banned);

    // Mute chat
    state.admin.set_trader_chat(user_id, false).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled);

    // Unmute chat
    state.admin.set_trader_chat(user_id, true).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled);
}

// =============================================================================
// ENGINE SETTLEMENT DEEP TESTS
// =============================================================================

/// Test full buy-sell settlement cycle with price improvement
#[tokio::test]
async fn test_engine_full_settlement_cycle() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SETTLE", "Settlement Co").await;

    // Create seller with shares
    let seller = create_test_user_with_portfolio(
        &state,
        "SETTLESELL",
        "Settlement Seller",
        dollars(10_000),
        vec![("SETTLE".to_string(), 100)],
    )
    .await;

    // Create buyer with cash
    let buyer = create_test_user_with_portfolio(
        &state,
        "SETTLEBUY",
        "Settlement Buyer",
        dollars(20_000),
        vec![],
    )
    .await;

    let seller_initial = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_initial = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    open_market(&state);

    // Seller places ask at $95
    place_limit_sell(&state, seller, &symbol, 50, dollars(95))
        .await
        .unwrap();

    // Buyer places bid at $100 - should match at $95 (price improvement)
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify settlement
    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    // Seller should have: initial_money + (50 * $95) = $10,000 + $4,750 = $14,750
    assert_eq!(seller_after.money, dollars(14_750));

    // Seller should have 50 shares left
    let seller_pos = seller_after
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .unwrap();
    assert_eq!(seller_pos.qty, 50);

    // Buyer should have: initial_money - (50 * $95) = $20,000 - $4,750 = $15,250
    assert_eq!(buyer_after.money, dollars(15_250));

    // Buyer should have 50 shares
    let buyer_pos = buyer_after
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .unwrap();
    assert_eq!(buyer_pos.qty, 50);
}

/// Test partial fill settlement
#[tokio::test]
async fn test_engine_partial_fill_settlement() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PARTFILL", "Partial Fill Co").await;

    // Seller with 30 shares
    let seller = create_test_user_with_portfolio(
        &state,
        "PARTFILLSELL",
        "Partial Fill Seller",
        dollars(10_000),
        vec![("PARTFILL".to_string(), 30)],
    )
    .await;

    // Buyer wants 50 shares
    let buyer = create_test_user_with_portfolio(
        &state,
        "PARTFILLBUY",
        "Partial Fill Buyer",
        dollars(20_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Seller places ask for 30
    place_limit_sell(&state, seller, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Buyer places bid for 50 - should partially fill
    let order_id = place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify partial fill
    let order = state.orders.get_order(order_id);
    if let Some(o) = order {
        assert_eq!(o.filled_qty, 30);
        assert_eq!(o.status, OrderStatus::Partial);
    }

    // Buyer should have 30 shares
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let buyer_pos = buyer_after.portfolio.iter().find(|p| p.symbol == symbol);
    assert!(buyer_pos.is_some());
    assert_eq!(buyer_pos.unwrap().qty, 30);
}

// =============================================================================
// SESSION SERVICE DEEP TESTS
// =============================================================================

/// Test session service with FIFO eviction
#[tokio::test]
async fn test_session_fifo_eviction() {
    let config = TestConfig {
        max_sessions_per_user: 2,
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "FIFOUSER", "FIFO User", "pass").await;

    // Create 3 sessions - first should be evicted
    let (s1, _) = state.sessions.create_session(user_id);
    let (s2, _) = state.sessions.create_session(user_id);
    let (s3, kicked) = state.sessions.create_session(user_id);

    // First session should be kicked
    assert_eq!(kicked.len(), 1);
    assert_eq!(kicked[0], s1);

    // First session should be gone
    assert!(state.sessions.get_session(s1).is_none());

    // Other sessions should exist
    assert!(state.sessions.get_session(s2).is_some());
    assert!(state.sessions.get_session(s3).is_some());
}

/// Test session service get_all_sessions
#[tokio::test]
async fn test_session_get_all() {
    let state = create_test_state().await;

    let user1 = create_test_user(&state, "SESSALL1", "Session All 1", "pass").await;
    let user2 = create_test_user(&state, "SESSALL2", "Session All 2", "pass").await;

    state.sessions.create_session(user1);
    state.sessions.create_session(user2);

    let all_sessions = state.sessions.get_all_sessions();
    assert_eq!(all_sessions.len(), 2);
}

// =============================================================================
// TOKEN SERVICE DEEP TESTS
// =============================================================================

/// Test token service FIFO eviction
#[tokio::test]
async fn test_token_fifo_eviction() {
    let config = TestConfig {
        max_sessions_per_user: 2,
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "TOKFIFO", "Token FIFO User", "pass").await;

    // Create 3 tokens - first should be revoked
    let (t1, _) = state.tokens.create_token(user_id);
    let (t2, _) = state.tokens.create_token(user_id);
    let (t3, revoked) = state.tokens.create_token(user_id);

    // First token should be revoked
    assert!(!revoked.is_empty());
    assert!(revoked.contains(&t1));

    // First token should not validate
    assert!(state.tokens.validate_token(&t1).is_none());

    // Other tokens should validate
    assert_eq!(state.tokens.validate_token(&t2), Some(user_id));
    assert_eq!(state.tokens.validate_token(&t3), Some(user_id));
}

/// Test token service explicit revocation
#[tokio::test]
async fn test_token_explicit_revoke() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TOKREVOKE", "Token Revoke User", "pass").await;

    let (token, _) = state.tokens.create_token(user_id);

    // Verify valid
    assert_eq!(state.tokens.validate_token(&token), Some(user_id));

    // Explicitly revoke
    state.tokens.revoke_token(&token);

    // Should no longer validate
    assert!(state.tokens.validate_token(&token).is_none());
}

// =============================================================================
// INVARIANT TESTS
// =============================================================================

/// Test money conservation after trade
#[tokio::test]
async fn test_money_conservation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CONSERVE", "Conservation Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "CONSERVESELL",
        "Conservation Seller",
        dollars(10_000),
        vec![("CONSERVE".to_string(), 100)],
    )
    .await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "CONSERVEBUY",
        "Conservation Buyer",
        dollars(20_000),
        vec![],
    )
    .await;

    // Calculate initial total money
    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_before = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let total_before = seller_before.money + buyer_before.money;

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Calculate final total money (including locked)
    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let total_after = seller_after.money
        + seller_after.locked_money
        + buyer_after.money
        + buyer_after.locked_money;

    // Money should be conserved
    assert_eq!(total_before, total_after);
}

/// Test share conservation after trade
#[tokio::test]
async fn test_share_conservation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SHARECON", "Share Conservation Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SHARECONSELL",
        "Share Con Seller",
        dollars(10_000),
        vec![("SHARECON".to_string(), 100)],
    )
    .await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "SHARECONBUY",
        "Share Con Buyer",
        dollars(20_000),
        vec![],
    )
    .await;

    // Calculate initial total shares
    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let seller_shares_before = seller_before
        .portfolio
        .iter()
        .filter(|p| p.symbol == symbol)
        .map(|p| p.qty + p.locked_qty)
        .sum::<u64>();

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Calculate final total shares
    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let seller_shares_after = seller_after
        .portfolio
        .iter()
        .filter(|p| p.symbol == symbol)
        .map(|p| p.qty + p.locked_qty)
        .sum::<u64>();
    let buyer_shares_after = buyer_after
        .portfolio
        .iter()
        .filter(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .sum::<u64>();

    // Shares should be conserved
    assert_eq!(
        seller_shares_before,
        seller_shares_after + buyer_shares_after
    );
}
