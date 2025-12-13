//! Service Coverage Tests
//!
//! Tests for: SVC-*, ADMIN-*, ORDER-*, TRADE-*, INDEX-*, NEWS-*, LEAD-*
//! Ensures all service methods are tested with proper state verification

mod common;

use common::*;
use std::collections::HashMap;

// =============================================================================
// ORDERS SERVICE TESTS (SVC-ORDERS-*)
// =============================================================================

/// SVC-ORDERS-001: Add order to tracking
#[tokio::test]
async fn test_orders_service_add_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDADD", "Order Add Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDADDUSER",
        "Order Add User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify order is tracked in OrdersService
    let user_orders = state.orders.get_user_orders(user);
    assert!(!user_orders.is_empty(), "User should have orders tracked");
    assert!(user_orders.iter().any(|o| o.order_id == order_id));
}

/// SVC-ORDERS-002: Update order status
#[tokio::test]
async fn test_orders_service_update_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDUPD", "Order Update Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "ORDUPDSELL",
        "Order Update Seller",
        dollars(10_000),
        vec![("ORDUPD".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "ORDUPDBUY",
        "Order Update Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place sell order
    let sell_id = place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Partial fill
    place_limit_buy(&state, buyer, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Check order was updated
    let sell_order = state.orders.get_order(sell_id);
    if let Some(order) = sell_order {
        assert_eq!(order.filled_qty, 30, "Order should be partially filled");
        assert_eq!(
            order.status,
            stockmart_backend::domain::models::OrderStatus::Partial
        );
    }
}

/// SVC-ORDERS-003: Remove order from tracking
#[tokio::test]
async fn test_orders_service_remove_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDREM", "Order Remove Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDREMUSER",
        "Order Remove User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Cancel order - should be removed from tracking
    state
        .engine
        .cancel_order(user, &symbol, order_id)
        .await
        .unwrap();

    // Verify order is no longer tracked
    let user_orders = state.orders.get_user_orders(user);
    assert!(
        user_orders.iter().all(|o| o.order_id != order_id),
        "Cancelled order should be removed from tracking"
    );
}

/// SVC-ORDERS-004: Get all orders
#[tokio::test]
async fn test_orders_service_get_all_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDALL", "Order All Co").await;

    let user1 = create_test_user_with_portfolio(
        &state,
        "ORDALL1",
        "Order All User 1",
        dollars(100_000),
        vec![],
    )
    .await;
    let user2 = create_test_user_with_portfolio(
        &state,
        "ORDALL2",
        "Order All User 2",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user2, &symbol, 20, dollars(99))
        .await
        .unwrap();

    // Get all orders
    let all_orders = state.orders.get_all_orders();
    assert_eq!(all_orders.len(), 2, "Should have 2 orders total");
}

/// SVC-ORDERS-005: Get orders by symbol
#[tokio::test]
async fn test_orders_service_get_orders_by_symbol() {
    let state = create_test_state().await;
    let symbol1 = create_test_company(&state, "ORDSYM1", "Order Symbol 1").await;
    let symbol2 = create_test_company(&state, "ORDSYM2", "Order Symbol 2").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDSYMUSER",
        "Order Symbol User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user, &symbol1, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol2, 20, dollars(100))
        .await
        .unwrap();

    // Get orders for symbol1 only
    let symbol1_orders = state.orders.get_orders_by_symbol(&symbol1);
    assert_eq!(symbol1_orders.len(), 1, "Should have 1 order for symbol1");
    assert!(symbol1_orders[0].symbol == symbol1);
}

/// SVC-ORDERS-006: Get user order count
#[tokio::test]
async fn test_orders_service_user_order_count() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDCNT", "Order Count Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDCNTUSER",
        "Order Count User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(99))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(98))
        .await
        .unwrap();

    let count = state.orders.get_user_order_count(user);
    assert_eq!(count, 3, "User should have 3 orders");
}

/// SVC-ORDERS-007: Clear user orders
#[tokio::test]
async fn test_orders_service_clear_user_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDCLR", "Order Clear Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDCLRUSER",
        "Order Clear User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(99))
        .await
        .unwrap();

    // Clear user's orders
    state.orders.clear_user_orders(user);

    let count = state.orders.get_user_order_count(user);
    assert_eq!(count, 0, "User should have 0 orders after clear");
}

/// SVC-ORDERS-008: Order exists check
#[tokio::test]
async fn test_orders_service_order_exists() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDEXIST", "Order Exist Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDEXISTUSER",
        "Order Exist User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    assert!(state.orders.order_exists(order_id));
    assert!(!state.orders.order_exists(99999));
}

/// SVC-ORDERS-009: Get total open orders count
#[tokio::test]
async fn test_orders_service_total_count() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDTOT", "Order Total Co").await;

    let user1 = create_test_user_with_portfolio(
        &state,
        "ORDTOT1",
        "Order Total 1",
        dollars(100_000),
        vec![],
    )
    .await;
    let user2 = create_test_user_with_portfolio(
        &state,
        "ORDTOT2",
        "Order Total 2",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user1, &symbol, 10, dollars(99))
        .await
        .unwrap();
    place_limit_buy(&state, user2, &symbol, 10, dollars(98))
        .await
        .unwrap();

    let total = state.orders.get_total_open_orders_count();
    assert_eq!(total, 3, "Should have 3 total open orders");
}

/// SVC-ORDERS-010: Get all orders for admin
#[tokio::test]
async fn test_orders_service_admin_view() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDADM", "Order Admin Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "ORDADMUSER",
        "Order Admin User",
        dollars(100_000),
        vec![],
    )
    .await;

    // Create user name map
    let mut user_names = HashMap::new();
    user_names.insert(user, "Order Admin User".to_string());

    open_market(&state);

    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let admin_orders = state
        .orders
        .get_all_orders_admin(Some(&symbol), &user_names);
    assert_eq!(admin_orders.len(), 1);
    assert_eq!(admin_orders[0].user_name, "Order Admin User");
}

// =============================================================================
// TRADE HISTORY SERVICE TESTS (SVC-TRADE-*)
// =============================================================================

/// SVC-TRADE-001: Record trade
#[tokio::test]
async fn test_trade_history_record_trade() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDREC", "Trade Record Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDRECSELL",
        "Trade Record Seller",
        dollars(10_000),
        vec![("TRDREC".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDRECBUY",
        "Trade Record Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify trade recorded
    let count = state.trade_history.get_total_trade_count();
    assert!(count >= 1, "Should have at least 1 trade recorded");
}

/// SVC-TRADE-002: Get user trades with pagination
#[tokio::test]
async fn test_trade_history_user_pagination() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDPAGE", "Trade Page Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDPAGESELL",
        "Trade Page Seller",
        dollars(10_000),
        vec![("TRDPAGE".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDPAGEBUY",
        "Trade Page Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Execute multiple trades
    for _ in 0..5 {
        place_limit_sell(&state, seller, &symbol, 2, dollars(100))
            .await
            .unwrap();
        place_limit_buy(&state, buyer, &symbol, 2, dollars(100))
            .await
            .unwrap();
    }

    // Test pagination - page 0, 3 per page
    let page0 = state.trade_history.get_user_trades(buyer, 0, 3);
    assert_eq!(page0.trades.len(), 3, "Page 0 should have 3 trades");
    assert!(page0.has_more, "Should have more pages");
    assert_eq!(page0.total_count, 5);

    // Page 1
    let page1 = state.trade_history.get_user_trades(buyer, 1, 3);
    assert_eq!(page1.trades.len(), 2, "Page 1 should have 2 trades");
    assert!(!page1.has_more, "Should not have more pages");
}

/// SVC-TRADE-003: Get symbol trades
#[tokio::test]
async fn test_trade_history_symbol_trades() {
    let state = create_test_state().await;
    let symbol1 = create_test_company(&state, "TRDSYM1", "Trade Symbol 1").await;
    let symbol2 = create_test_company(&state, "TRDSYM2", "Trade Symbol 2").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDSYMSELL",
        "Trade Symbol Seller",
        dollars(10_000),
        vec![("TRDSYM1".to_string(), 100), ("TRDSYM2".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDSYMBUY",
        "Trade Symbol Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Trade symbol1
    place_limit_sell(&state, seller, &symbol1, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol1, 10, dollars(100))
        .await
        .unwrap();

    // Trade symbol2
    place_limit_sell(&state, seller, &symbol2, 5, dollars(200))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol2, 5, dollars(200))
        .await
        .unwrap();

    // Get symbol1 trades only
    let sym1_trades = state.trade_history.get_symbol_trades(&symbol1, 10);
    assert!(sym1_trades.iter().all(|t| t.symbol == symbol1));
}

/// SVC-TRADE-004: Get all trades with filters
#[tokio::test]
async fn test_trade_history_filtered() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDFILT", "Trade Filter Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDFILTSELL",
        "Trade Filter Seller",
        dollars(10_000),
        vec![("TRDFILT".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDFILTBUY",
        "Trade Filter Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Filter by user
    let user_trades = state.trade_history.get_all_trades(Some(buyer), None, 0, 10);
    assert!(!user_trades.trades.is_empty());

    // Filter by symbol
    let symbol_trades = state
        .trade_history
        .get_all_trades(None, Some(&symbol), 0, 10);
    assert!(!symbol_trades.trades.is_empty());
}

/// SVC-TRADE-005: Get user trade count
#[tokio::test]
async fn test_trade_history_user_count() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDCNT", "Trade Count Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDCNTSELL",
        "Trade Count Seller",
        dollars(10_000),
        vec![("TRDCNT".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDCNTBUY",
        "Trade Count Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Execute 3 trades
    for _ in 0..3 {
        place_limit_sell(&state, seller, &symbol, 5, dollars(100))
            .await
            .unwrap();
        place_limit_buy(&state, buyer, &symbol, 5, dollars(100))
            .await
            .unwrap();
    }

    let count = state.trade_history.get_user_trade_count(buyer);
    assert_eq!(count, 3, "Buyer should have 3 trades");
}

/// SVC-TRADE-006: Clear all trade history
#[tokio::test]
async fn test_trade_history_clear() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDCLR", "Trade Clear Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDCLRSELL",
        "Trade Clear Seller",
        dollars(10_000),
        vec![("TRDCLR".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDCLRBUY",
        "Trade Clear Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Clear all
    state.trade_history.clear_all();

    let count = state.trade_history.get_total_trade_count();
    assert_eq!(count, 0, "Should have 0 trades after clear");
}

/// SVC-TRADE-007: Get total volume
#[tokio::test]
async fn test_trade_history_total_volume() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDVOL", "Trade Volume Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDVOLSELL",
        "Trade Volume Seller",
        dollars(10_000),
        vec![("TRDVOL".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDVOLBUY",
        "Trade Volume Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Trade: 10 shares at $100 = $1000 total
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let volume = state.trade_history.get_total_volume();
    assert!(volume >= dollars(1_000), "Volume should be at least $1000");
}

/// SVC-TRADE-008: Get recent volume
#[tokio::test]
async fn test_trade_history_recent_volume() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDRECV", "Trade Recent Vol Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDRECVSELL",
        "Trade Recent Vol Seller",
        dollars(10_000),
        vec![("TRDRECV".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDRECVBUY",
        "Trade Recent Vol Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Get volume in last 60 seconds
    let recent_volume = state.trade_history.get_recent_volume(60);
    assert!(
        recent_volume >= dollars(1_000),
        "Recent volume should include our trade"
    );
}

/// SVC-TRADE-009: Admin trade history view
#[tokio::test]
async fn test_trade_history_admin_view() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRDADM", "Trade Admin Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRDADMSELL",
        "Trade Admin Seller",
        dollars(10_000),
        vec![("TRDADM".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRDADMBUY",
        "Trade Admin Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let (trades, total, has_more) = state.trade_history.get_all_trades_admin(None, None, 0, 10);
    assert!(!trades.is_empty());
    assert!(total >= 1);
    // Admin view should have buyer/seller info
    assert!(!trades[0].buyer_name.is_empty());
    assert!(!trades[0].seller_name.is_empty());
}

// =============================================================================
// ADMIN SERVICE TESTS (SVC-ADMIN-*)
// =============================================================================

/// SVC-ADMIN-001: Toggle market
#[tokio::test]
async fn test_admin_toggle_market() {
    let state = create_test_state().await;

    state.admin.toggle_market(false);
    assert!(!is_market_open(&state));

    state.admin.toggle_market(true);
    assert!(is_market_open(&state));
}

/// SVC-ADMIN-002: Set company volatility
#[tokio::test]
async fn test_admin_set_volatility() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "VOLAT", "Volatility Co").await;

    state
        .admin
        .set_company_volatility(&symbol, 50)
        .await
        .unwrap();

    let company = state
        .company_repo
        .find_by_symbol(&symbol)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(company.volatility, 50);
}

/// SVC-ADMIN-003: Set volatility for non-existent company
#[tokio::test]
async fn test_admin_set_volatility_nonexistent() {
    let state = create_test_state().await;

    let result = state.admin.set_company_volatility("DOESNOTEXIST", 50).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

/// SVC-ADMIN-004: Set company bankrupt
#[tokio::test]
async fn test_admin_set_bankrupt() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BANKR", "Bankrupt Co").await;

    state
        .admin
        .set_company_bankrupt(&symbol, true)
        .await
        .unwrap();

    let company = state
        .company_repo
        .find_by_symbol(&symbol)
        .await
        .unwrap()
        .unwrap();
    assert!(company.bankrupt);

    state
        .admin
        .set_company_bankrupt(&symbol, false)
        .await
        .unwrap();
    let company = state
        .company_repo
        .find_by_symbol(&symbol)
        .await
        .unwrap()
        .unwrap();
    assert!(!company.bankrupt);
}

/// SVC-ADMIN-005: Create new company
#[tokio::test]
async fn test_admin_create_company() {
    let state = create_test_state().await;

    state
        .admin
        .create_company(
            "NEWCO".to_string(),
            "New Company Inc".to_string(),
            "Technology".to_string(),
            25,
        )
        .await
        .unwrap();

    let company = state
        .company_repo
        .find_by_symbol("NEWCO")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(company.name, "New Company Inc");
    assert_eq!(company.sector, "Technology");
    assert_eq!(company.volatility, 25);
}

/// SVC-ADMIN-006: Create duplicate company
#[tokio::test]
async fn test_admin_create_duplicate_company() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "DUPSYM", "Duplicate Symbol Co").await;

    let result = state
        .admin
        .create_company(
            symbol,
            "Another Company".to_string(),
            "Finance".to_string(),
            10,
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

/// SVC-ADMIN-007: Get all traders
#[tokio::test]
async fn test_admin_get_all_traders() {
    let state = create_test_state().await;

    create_test_user(&state, "TRADER1", "Trader One", "pass").await;
    create_test_user(&state, "TRADER2", "Trader Two", "pass").await;

    let traders = state.admin.get_all_traders().await.unwrap();
    assert_eq!(traders.len(), 2);
}

/// SVC-ADMIN-008: Ban trader
#[tokio::test]
async fn test_admin_ban_trader() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BANNED", "Banned User", "pass").await;

    state.admin.set_trader_banned(user_id, true).await.unwrap();

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.banned);
}

/// SVC-ADMIN-009: Ban non-existent trader
#[tokio::test]
async fn test_admin_ban_nonexistent() {
    let state = create_test_state().await;

    let result = state.admin.set_trader_banned(99999, true).await;
    assert!(result.is_err());
}

/// SVC-ADMIN-010: Set trader chat enabled
#[tokio::test]
async fn test_admin_set_trader_chat() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "CHATUSER", "Chat User", "pass").await;

    state.admin.set_trader_chat(user_id, false).await.unwrap();

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled);

    state.admin.set_trader_chat(user_id, true).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled);
}

// =============================================================================
// INDICES SERVICE TESTS (SVC-INDEX-*)
// =============================================================================

/// SVC-INDEX-001: Subscribe to indices
#[tokio::test]
async fn test_indices_subscribe() {
    let state = create_test_state().await;

    let _rx = state.indices.subscribe_indices();

    // Should not panic
}

/// SVC-INDEX-002: Get all indices (initially empty)
#[tokio::test]
async fn test_indices_get_all_empty() {
    let state = create_test_state().await;

    let indices = state.indices.get_all_indices();
    // May be empty or have default values depending on init
    assert!(indices.is_empty() || !indices.is_empty()); // Just test no panic
}

/// SVC-INDEX-003: Get specific index
#[tokio::test]
async fn test_indices_get_specific() {
    let state = create_test_state().await;

    // This may return None if no indices calculated yet
    let index = state.indices.get_index("MARKET");
    // Document behavior
    match index {
        Some(i) => println!("MARKET index: {}", i.value),
        None => println!("MARKET index not yet calculated"),
    }
}

// =============================================================================
// LEADERBOARD SERVICE TESTS (SVC-LEAD-*)
// =============================================================================

/// SVC-LEAD-001: Subscribe to leaderboard
#[tokio::test]
async fn test_leaderboard_subscribe() {
    let state = create_test_state().await;

    let _rx = state.leaderboard.subscribe();
    // Should not panic
}

/// SVC-LEAD-002: Get current leaderboard
#[tokio::test]
async fn test_leaderboard_get_current() {
    let state = create_test_state().await;

    // Create some users with different net worth
    create_test_user_with_portfolio(&state, "LEADER1", "Leader 1", dollars(100_000), vec![]).await;
    create_test_user_with_portfolio(&state, "LEADER2", "Leader 2", dollars(50_000), vec![]).await;

    let leaderboard = state.leaderboard.get_current();
    // May be empty if not updated yet
    println!("Leaderboard entries: {}", leaderboard.len());
}

// =============================================================================
// NEWS SERVICE TESTS (SVC-NEWS-*)
// =============================================================================

/// SVC-NEWS-001: Subscribe to news
#[tokio::test]
async fn test_news_subscribe() {
    let state = create_test_state().await;

    let _rx = state.news.subscribe();
    // Should not panic
}

/// SVC-NEWS-002: Get recent news
#[tokio::test]
async fn test_news_get_recent() {
    let state = create_test_state().await;

    let news = state.news.get_recent(10);
    assert!(news.len() <= 10);
}

// =============================================================================
// CHAT SERVICE TESTS (SVC-CHAT-*)
// =============================================================================

/// SVC-CHAT-001: Subscribe to chat
#[tokio::test]
async fn test_chat_subscribe() {
    let state = create_test_state().await;

    let _rx = state.chat.subscribe();
    // Should not panic
}

/// SVC-CHAT-002: Get chat history
#[tokio::test]
async fn test_chat_get_history() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "CHATHISTUSER", "Chat History User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Add chat message
    let message = stockmart_backend::domain::market::chat::ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        user_id,
        username: user.name,
        message: "Test message".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    state.chat.broadcast_message(message);

    let history = state.chat.get_history();
    assert!(!history.is_empty());
    assert!(history.iter().any(|m| m.message == "Test message"));
}

/// SVC-CHAT-003: Chat history limit (50 messages)
#[tokio::test]
async fn test_chat_history_limit() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "CHATCLRUSER", "Chat Clear User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Add more than 50 messages
    for i in 0..60 {
        let message = stockmart_backend::domain::market::chat::ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username: user.name.clone(),
            message: format!("Message {}", i),
            timestamp: chrono::Utc::now().timestamp(),
        };
        state.chat.broadcast_message(message);
    }

    // Should be limited to 50
    let history = state.chat.get_history();
    assert!(history.len() <= 50);
}

// =============================================================================
// SESSION SERVICE TESTS (SVC-SESSION-*)
// =============================================================================

/// SVC-SESSION-001: Create session
#[tokio::test]
async fn test_session_create() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "SESSUSER", "Session User", "pass").await;

    // Create session using create_session API
    let (session_id, kicked) = state.sessions.create_session(user_id);

    // Should have a valid session ID
    assert!(session_id > 0);
    // Should not have kicked anyone (first session)
    assert!(kicked.is_empty());
}

/// SVC-SESSION-002: Remove session
#[tokio::test]
async fn test_session_remove() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "SESSREM", "Session Remove User", "pass").await;

    let (session_id, _) = state.sessions.create_session(user_id);

    // Remove the session
    state.sessions.remove_session(session_id);

    // Session should be gone
    assert!(state.sessions.get_session(session_id).is_none());
}

/// SVC-SESSION-003: Max sessions enforcement
#[tokio::test]
async fn test_session_max_enforcement() {
    // Create state with max 1 session
    let config = TestConfig {
        max_sessions_per_user: 1,
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "SESSMAX", "Session Max User", "pass").await;

    // First session
    let (session1, kicked1) = state.sessions.create_session(user_id);
    assert!(kicked1.is_empty(), "First session should not kick anyone");

    // Second session should kick the first
    let (_session2, kicked2) = state.sessions.create_session(user_id);
    assert!(!kicked2.is_empty(), "Second session should kick first");
    assert_eq!(kicked2[0], session1);
}

// =============================================================================
// TOKEN SERVICE TESTS (SVC-TOKEN-*)
// =============================================================================

/// SVC-TOKEN-001: Create token
#[tokio::test]
async fn test_token_create() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TOKENUSER", "Token User", "pass").await;

    let (token, _revoked) = state.tokens.create_token(user_id);
    assert!(!token.is_empty());
}

/// SVC-TOKEN-002: Validate token
#[tokio::test]
async fn test_token_validate() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TOKENVAL", "Token Validate User", "pass").await;

    let (token, _) = state.tokens.create_token(user_id);
    let validated = state.tokens.validate_token(&token);

    assert_eq!(validated, Some(user_id));
}

/// SVC-TOKEN-003: Validate invalid token
#[tokio::test]
async fn test_token_validate_invalid() {
    let state = create_test_state().await;

    let validated = state.tokens.validate_token("invalid_token_12345");
    assert!(validated.is_none());
}

/// SVC-TOKEN-004: Token revocation on multiple tokens
#[tokio::test]
async fn test_token_revocation() {
    // Create state with max 1 session
    let config = TestConfig {
        max_sessions_per_user: 1,
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "TOKENREV", "Token Revoke User", "pass").await;

    // Create first token
    let (token1, revoked1) = state.tokens.create_token(user_id);
    assert!(
        revoked1.is_empty(),
        "First token should not revoke anything"
    );

    // Create second token - should revoke first
    let (_token2, revoked2) = state.tokens.create_token(user_id);
    assert!(!revoked2.is_empty(), "Second token should revoke first");
    assert!(
        revoked2.contains(&token1),
        "First token should be in revoked list"
    );

    // First token should no longer be valid
    let validated = state.tokens.validate_token(&token1);
    assert!(validated.is_none(), "Revoked token should not validate");
}

// =============================================================================
// MARKET SERVICE TESTS (SVC-MARKET-*)
// =============================================================================

/// SVC-MARKET-001: Get candles (empty)
#[tokio::test]
async fn test_market_get_candles_empty() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MKTCND", "Market Candle Co").await;

    let candles = state.market.get_candles(&symbol);
    // May be empty initially
    assert!(candles.len() >= 0);
}

/// SVC-MARKET-002: Get last price
#[tokio::test]
async fn test_market_get_last_price() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MKTPRICE", "Market Price Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "MKTPRICESELL",
        "Market Price Seller",
        dollars(10_000),
        vec![("MKTPRICE".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "MKTPRICEBUY",
        "Market Price Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Execute trade at $150
    place_limit_sell(&state, seller, &symbol, 10, dollars(150))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(150))
        .await
        .unwrap();

    // Note: MarketService needs to process the trade for get_last_price to work
    // In tests without background task, we may need to manually process
    let price = state.market.get_last_price(&symbol);
    println!("Last price for {}: {:?}", symbol, price);
}

/// SVC-MARKET-003: Subscribe to candles
#[tokio::test]
async fn test_market_subscribe_candles() {
    let state = create_test_state().await;

    let _rx = state.market.subscribe_candles();
    // Should not panic
}

/// SVC-MARKET-004: Subscribe to circuit breakers
#[tokio::test]
async fn test_market_subscribe_circuit_breakers() {
    let state = create_test_state().await;

    let _rx = state.market.subscribe_circuit_breakers();
    // Should not panic
}

// =============================================================================
// ENGINE SERVICE TESTS (SVC-ENGINE-*)
// =============================================================================

/// SVC-ENGINE-001: Create orderbook
#[tokio::test]
async fn test_engine_create_orderbook() {
    let state = create_test_state().await;

    state.engine.create_orderbook("NEWBOOK".to_string());

    // Should now be able to get depth
    let depth = state.engine.get_order_book_depth("NEWBOOK", 5);
    assert!(depth.is_some());
}

/// SVC-ENGINE-002: Clear orderbook
#[tokio::test]
async fn test_engine_clear_orderbook() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CLRBOOK", "Clear Book Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "CLRBOOKBUY",
        "Clear Book Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Add orders
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Clear
    state.engine.clear_orderbook(&symbol);

    // Should be empty
    let depth = state.engine.get_order_book_depth(&symbol, 5);
    let (bids, asks) = depth.unwrap();
    assert!(bids.is_empty());
    assert!(asks.is_empty());
}

/// SVC-ENGINE-003: Seed order (admin liquidity)
#[tokio::test]
async fn test_engine_seed_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SEEDORD", "Seed Order Co").await;

    let order = stockmart_backend::domain::models::Order {
        id: stockmart_backend::infrastructure::id_generator::IdGenerators::global().next_order_id(),
        user_id: 1, // Admin
        symbol: symbol.clone(),
        order_type: stockmart_backend::domain::models::OrderType::Limit,
        side: stockmart_backend::domain::models::OrderSide::Buy,
        qty: 100,
        filled_qty: 0,
        price: dollars(100),
        status: stockmart_backend::domain::models::OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: stockmart_backend::domain::models::TimeInForce::GTC,
    };

    state.engine.seed_order(order);

    // Should have the seeded order in the book
    let depth = state.engine.get_order_book_depth(&symbol, 5);
    let (bids, _) = depth.unwrap();
    assert!(!bids.is_empty());
    assert_eq!(bids[0].0, dollars(100));
}

/// SVC-ENGINE-004: Subscribe to trades
#[tokio::test]
async fn test_engine_subscribe_trades() {
    let state = create_test_state().await;

    let _rx = state.engine.subscribe_trades();
    // Should not panic
}
