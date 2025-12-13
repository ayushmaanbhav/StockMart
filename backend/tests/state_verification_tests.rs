//! State Verification & Data Integrity Tests
//!
//! Tests for: STATE-*, DATA-*, INTEGRITY-*
//! Focuses on comprehensive state verification after operations

mod common;

use common::*;

// =============================================================================
// PORTFOLIO STATE VERIFICATION (STATE-PORT-*)
// =============================================================================

/// STATE-PORT-001: Full portfolio state after buy trade
#[tokio::test]
async fn test_portfolio_state_after_buy() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BUYSTATE", "Buy State Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "BUYSTATEST",
        "Buy State Seller",
        dollars(10_000),
        vec![("BUYSTATE".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "BUYSTATEBUY",
        "Buy State Buyer",
        dollars(10_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Record initial state
    let buyer_initial = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let seller_initial = state.user_repo.find_by_id(seller).await.unwrap().unwrap();

    // Execute trade: 10 shares at $100
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify buyer state
    let buyer_final = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(
        buyer_final.money,
        buyer_initial.money - dollars(1_000),
        "Buyer money should decrease by trade value"
    );
    assert_eq!(
        buyer_final.locked_money, 0,
        "No locked money after filled order"
    );

    let buyer_position = buyer_final.portfolio.iter().find(|p| p.symbol == symbol);
    assert!(buyer_position.is_some(), "Buyer should have position");
    assert_eq!(
        buyer_position.unwrap().qty,
        10,
        "Buyer should have 10 shares"
    );
    assert_eq!(buyer_position.unwrap().locked_qty, 0, "No locked shares");

    // Verify seller state
    let seller_final = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    assert_eq!(
        seller_final.money,
        seller_initial.money + dollars(1_000),
        "Seller money should increase by trade value"
    );

    let seller_position = seller_final.portfolio.iter().find(|p| p.symbol == symbol);
    assert!(
        seller_position.is_some(),
        "Seller should still have position"
    );
    assert_eq!(
        seller_position.unwrap().qty,
        90,
        "Seller should have 90 shares left"
    );
    assert_eq!(
        seller_position.unwrap().locked_qty,
        0,
        "No locked shares after fill"
    );
}

/// STATE-PORT-002: Portfolio state after partial fill
#[tokio::test]
async fn test_portfolio_state_after_partial_fill() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PARTSTATE", "Partial State Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "PARTSTATEST",
        "Partial State Seller",
        dollars(10_000),
        vec![("PARTSTATE".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "PARTSTATEBUY",
        "Partial State Buyer",
        dollars(10_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Seller offers 100 shares at $100
    place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Buyer only buys 30 - partial fill
    place_limit_buy(&state, buyer, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Verify seller state after partial fill
    let seller_user = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let seller_position = seller_user
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .unwrap();

    // After partial fill: 30 shares sold, 70 remain
    // The qty is the remaining shares (100 - 30 sold = 70)
    // And 70 are still locked for the remaining sell order
    assert_eq!(
        seller_position.qty, 70,
        "Seller has 70 shares after selling 30"
    );
    assert_eq!(
        seller_position.locked_qty, 70,
        "70 shares still locked for order"
    );

    // Seller should have received money for 30 shares
    // Initial: $10,000, received: $3,000 for 30 shares
    assert_eq!(seller_user.money, dollars(10_000) + dollars(3_000));

    // Buyer should have 30 shares and paid $3,000
    let buyer_user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(buyer_user.money, dollars(10_000) - dollars(3_000));
    let buyer_position = buyer_user
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .unwrap();
    assert_eq!(buyer_position.qty, 30);
}

/// STATE-PORT-003: Portfolio state after cancel
#[tokio::test]
async fn test_portfolio_state_after_cancel() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CANCELST", "Cancel State Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "CANCELSTBUY",
        "Cancel State Buyer",
        dollars(10_000),
        vec![],
    )
    .await;

    open_market(&state);

    let buyer_initial = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    // Place order
    let order_id = place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify money locked
    let buyer_locked = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(buyer_locked.locked_money, dollars(1_000));
    assert_eq!(buyer_locked.money, buyer_initial.money - dollars(1_000));

    // Cancel order
    state
        .engine
        .cancel_order(buyer, &symbol, order_id)
        .await
        .unwrap();

    // Verify money released
    let buyer_final = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(
        buyer_final.locked_money, 0,
        "Locked money should be released"
    );
    assert_eq!(
        buyer_final.money, buyer_initial.money,
        "Money should be restored"
    );
}

/// STATE-PORT-004: Multiple position portfolio state
#[tokio::test]
async fn test_multiple_position_portfolio_state() {
    let state = create_test_state().await;
    let symbol1 = create_test_company(&state, "MULTI1", "Multi 1 Co").await;
    let symbol2 = create_test_company(&state, "MULTI2", "Multi 2 Co").await;
    let symbol3 = create_test_company(&state, "MULTI3", "Multi 3 Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "MULTISELL",
        "Multi Seller",
        dollars(10_000),
        vec![
            ("MULTI1".to_string(), 100),
            ("MULTI2".to_string(), 200),
            ("MULTI3".to_string(), 300),
        ],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "MULTIBUY",
        "Multi Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Trade all three symbols
    place_limit_sell(&state, seller, &symbol1, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol1, 10, dollars(100))
        .await
        .unwrap();

    place_limit_sell(&state, seller, &symbol2, 20, dollars(50))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol2, 20, dollars(50))
        .await
        .unwrap();

    place_limit_sell(&state, seller, &symbol3, 30, dollars(25))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol3, 30, dollars(25))
        .await
        .unwrap();

    // Verify buyer has all three positions
    let buyer_user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(
        buyer_user.portfolio.len(),
        3,
        "Buyer should have 3 positions"
    );

    // Verify correct quantities
    assert_eq!(
        buyer_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol1)
            .unwrap()
            .qty,
        10
    );
    assert_eq!(
        buyer_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol2)
            .unwrap()
            .qty,
        20
    );
    assert_eq!(
        buyer_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol3)
            .unwrap()
            .qty,
        30
    );

    // Verify seller positions reduced
    let seller_user = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    assert_eq!(
        seller_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol1)
            .unwrap()
            .qty,
        90
    );
    assert_eq!(
        seller_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol2)
            .unwrap()
            .qty,
        180
    );
    assert_eq!(
        seller_user
            .portfolio
            .iter()
            .find(|p| p.symbol == symbol3)
            .unwrap()
            .qty,
        270
    );
}

// =============================================================================
// ORDER BOOK STATE VERIFICATION (STATE-BOOK-*)
// =============================================================================

/// STATE-BOOK-001: Order book state after multiple orders
#[tokio::test]
async fn test_orderbook_state_multiple_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BOOKST", "Book State Co").await;

    let buyer1 =
        create_test_user_with_portfolio(&state, "BOOKST1", "Book 1", dollars(100_000), vec![])
            .await;
    let buyer2 =
        create_test_user_with_portfolio(&state, "BOOKST2", "Book 2", dollars(100_000), vec![])
            .await;
    let buyer3 =
        create_test_user_with_portfolio(&state, "BOOKST3", "Book 3", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Place orders at different prices
    place_limit_buy(&state, buyer1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer2, &symbol, 20, dollars(99))
        .await
        .unwrap();
    place_limit_buy(&state, buyer3, &symbol, 30, dollars(98))
        .await
        .unwrap();

    // Verify order book state
    let (bids, asks) = state.engine.get_order_book_depth(&symbol, 10).unwrap();

    assert_eq!(bids.len(), 3, "Should have 3 bid levels");
    assert!(asks.is_empty(), "No asks");

    // Verify price-priority ordering (highest first)
    assert_eq!(bids[0].0, dollars(100));
    assert_eq!(bids[0].1, 10);
    assert_eq!(bids[1].0, dollars(99));
    assert_eq!(bids[1].1, 20);
    assert_eq!(bids[2].0, dollars(98));
    assert_eq!(bids[2].1, 30);
}

/// STATE-BOOK-002: Order book state after trade removes liquidity
#[tokio::test]
async fn test_orderbook_state_after_trade() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BOOKTR", "Book Trade Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "BOOKTRSELL",
        "Book Trade Seller",
        dollars(10_000),
        vec![("BOOKTR".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "BOOKTRBUY",
        "Book Trade Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place buy order
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify bid exists
    let (bids1, _) = state.engine.get_order_book_depth(&symbol, 10).unwrap();
    assert_eq!(bids1.len(), 1);
    assert_eq!(bids1[0].1, 50);

    // Seller takes liquidity (sells into bid)
    place_limit_sell(&state, seller, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Verify bid partially consumed
    let (bids2, _) = state.engine.get_order_book_depth(&symbol, 10).unwrap();
    assert_eq!(bids2.len(), 1);
    assert_eq!(bids2[0].1, 20, "20 shares should remain in bid");
}

/// STATE-BOOK-003: Order book state after cancel
#[tokio::test]
async fn test_orderbook_state_after_cancel() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BOOKCANC", "Book Cancel Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "BOOKCANCBUY",
        "Book Cancel Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place order
    let order_id = place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify in book
    let (bids1, _) = state.engine.get_order_book_depth(&symbol, 10).unwrap();
    assert_eq!(bids1.len(), 1);

    // Cancel
    state
        .engine
        .cancel_order(buyer, &symbol, order_id)
        .await
        .unwrap();

    // Verify removed from book
    let (bids2, _) = state.engine.get_order_book_depth(&symbol, 10).unwrap();
    assert!(bids2.is_empty(), "Order should be removed from book");
}

/// STATE-BOOK-004: Ask side of order book
#[tokio::test]
async fn test_orderbook_ask_state() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BOOKASK", "Book Ask Co").await;

    let seller1 = create_test_user_with_portfolio(
        &state,
        "BOOKASK1",
        "Book Ask 1",
        dollars(10_000),
        vec![("BOOKASK".to_string(), 100)],
    )
    .await;
    let seller2 = create_test_user_with_portfolio(
        &state,
        "BOOKASK2",
        "Book Ask 2",
        dollars(10_000),
        vec![("BOOKASK".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Place sell orders at different prices
    place_limit_sell(&state, seller1, &symbol, 10, dollars(102))
        .await
        .unwrap();
    place_limit_sell(&state, seller2, &symbol, 20, dollars(101))
        .await
        .unwrap();

    // Verify ask side (lowest first)
    let (bids, asks) = state.engine.get_order_book_depth(&symbol, 10).unwrap();

    assert!(bids.is_empty());
    assert_eq!(asks.len(), 2);

    // Best ask should be lowest price
    assert_eq!(asks[0].0, dollars(101));
    assert_eq!(asks[0].1, 20);
    assert_eq!(asks[1].0, dollars(102));
    assert_eq!(asks[1].1, 10);
}

// =============================================================================
// TRADE HISTORY STATE VERIFICATION (STATE-TRADE-*)
// =============================================================================

/// STATE-TRADE-001: Trade history after single trade
#[tokio::test]
async fn test_trade_history_state_single() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "HISTSING", "History Single Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "HISTSINGSELL",
        "History Single Seller",
        dollars(10_000),
        vec![("HISTSING".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "HISTSINGBUY",
        "History Single Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Record time before trade
    let before_trade = chrono::Utc::now().timestamp();

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 15, dollars(123))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 15, dollars(123))
        .await
        .unwrap();

    // Verify trade history
    let history = state.trade_history.get_symbol_trades(&symbol, 10);
    assert_eq!(history.len(), 1);

    let trade = &history[0];
    assert_eq!(trade.symbol, symbol);
    assert_eq!(trade.qty, 15);
    assert_eq!(trade.price, dollars(123));
    assert_eq!(trade.total_value, dollars(123) * 15);
    assert!(trade.timestamp >= before_trade);
}

/// STATE-TRADE-002: Trade history ordering
#[tokio::test]
async fn test_trade_history_ordering() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "HISTORD", "History Order Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "HISTORDSELL",
        "History Order Seller",
        dollars(10_000),
        vec![("HISTORD".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "HISTORDBUY",
        "History Order Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Execute multiple trades at different prices
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    place_limit_sell(&state, seller, &symbol, 10, dollars(101))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(101))
        .await
        .unwrap();

    place_limit_sell(&state, seller, &symbol, 10, dollars(102))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(102))
        .await
        .unwrap();

    // Get history (should be newest first)
    let history = state.trade_history.get_symbol_trades(&symbol, 10);
    assert_eq!(history.len(), 3);

    // Most recent trade should be at $102
    assert_eq!(history[0].price, dollars(102));
    // Oldest trade should be at $100
    assert_eq!(history[2].price, dollars(100));
}

/// STATE-TRADE-003: Trade history indexed by user
#[tokio::test]
async fn test_trade_history_user_indexing() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "HISTUSR", "History User Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "HISTUSRSELL",
        "History User Seller",
        dollars(10_000),
        vec![("HISTUSR".to_string(), 100)],
    )
    .await;
    let buyer1 = create_test_user_with_portfolio(
        &state,
        "HISTUSRBUY1",
        "History User Buyer 1",
        dollars(100_000),
        vec![],
    )
    .await;
    let buyer2 = create_test_user_with_portfolio(
        &state,
        "HISTUSRBUY2",
        "History User Buyer 2",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Buyer1 does 2 trades
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer1, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Buyer2 does 1 trade
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer2, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify user trade counts
    assert_eq!(state.trade_history.get_user_trade_count(buyer1), 2);
    assert_eq!(state.trade_history.get_user_trade_count(buyer2), 1);
    assert_eq!(state.trade_history.get_user_trade_count(seller), 3); // Seller in all 3
}

// =============================================================================
// OPEN ORDERS STATE VERIFICATION (STATE-ORDER-*)
// =============================================================================

/// STATE-ORDER-001: Open orders tracking
#[tokio::test]
async fn test_open_orders_tracking() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "OPENORD", "Open Order Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "OPENORDUSER",
        "Open Order User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place multiple orders
    let order1 = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let order2 = place_limit_buy(&state, user, &symbol, 20, dollars(99))
        .await
        .unwrap();
    let order3 = place_limit_buy(&state, user, &symbol, 30, dollars(98))
        .await
        .unwrap();

    // Verify all tracked
    let user_orders = state.orders.get_user_orders(user);
    assert_eq!(user_orders.len(), 3);

    // Verify order details
    assert!(user_orders
        .iter()
        .any(|o| o.order_id == order1 && o.qty == 10));
    assert!(user_orders
        .iter()
        .any(|o| o.order_id == order2 && o.qty == 20));
    assert!(user_orders
        .iter()
        .any(|o| o.order_id == order3 && o.qty == 30));
}

/// STATE-ORDER-002: Order status transitions
#[tokio::test]
async fn test_order_status_transitions() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ORDSTAT", "Order Status Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "ORDSTATSELL",
        "Order Status Seller",
        dollars(10_000),
        vec![("ORDSTAT".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "ORDSTATBUY",
        "Order Status Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place large sell order
    let sell_id = place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Initial: Open
    let order = state.orders.get_order(sell_id).unwrap();
    assert_eq!(
        order.status,
        stockmart_backend::domain::models::OrderStatus::Open
    );
    assert_eq!(order.filled_qty, 0);

    // Partial fill
    place_limit_buy(&state, buyer, &symbol, 30, dollars(100))
        .await
        .unwrap();

    let order = state.orders.get_order(sell_id).unwrap();
    assert_eq!(
        order.status,
        stockmart_backend::domain::models::OrderStatus::Partial
    );
    assert_eq!(order.filled_qty, 30);

    // Full fill
    place_limit_buy(&state, buyer, &symbol, 70, dollars(100))
        .await
        .unwrap();

    // After full fill, order removed from tracking
    let order = state.orders.get_order(sell_id);
    assert!(
        order.is_none(),
        "Filled order should be removed from tracking"
    );
}

// =============================================================================
// INVARIANT VERIFICATION (INV-*)
// =============================================================================

/// INV-001: Money conservation across trade
#[tokio::test]
async fn test_money_conservation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MONEYCONS", "Money Conservation Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "MONEYSELL",
        "Money Seller",
        dollars(10_000),
        vec![("MONEYCONS".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "MONEYBUY", "Money Buyer", dollars(50_000), vec![])
            .await;

    // Calculate total money before
    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_before = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let total_before = seller_before.money + buyer_before.money;

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 25, dollars(150))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 25, dollars(150))
        .await
        .unwrap();

    // Calculate total money after
    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let total_after = seller_after.money + buyer_after.money;

    // Money should be conserved
    assert_eq!(total_before, total_after, "Total money should be conserved");
}

/// INV-002: Share conservation across trade
#[tokio::test]
async fn test_share_conservation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SHARECONS", "Share Conservation Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SHARESELL",
        "Share Seller",
        dollars(10_000),
        vec![("SHARECONS".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "SHAREBUY", "Share Buyer", dollars(50_000), vec![])
            .await;

    // Total shares before
    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_before = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let seller_shares_before = seller_before
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer_shares_before = buyer_before
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let total_before = seller_shares_before + buyer_shares_before;

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 40, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 40, dollars(100))
        .await
        .unwrap();

    // Total shares after
    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let seller_shares_after = seller_after
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer_shares_after = buyer_after
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let total_after = seller_shares_after + buyer_shares_after;

    assert_eq!(
        total_before, total_after,
        "Total shares should be conserved"
    );
}

/// INV-003: Locked money equals open buy order value
#[tokio::test]
async fn test_locked_money_equals_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "LOCKMONEY", "Lock Money Co").await;

    let user =
        create_test_user_with_portfolio(&state, "LOCKUSER", "Lock User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Place multiple buy orders
    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap(); // $1,000
    place_limit_buy(&state, user, &symbol, 20, dollars(50))
        .await
        .unwrap(); // $1,000
    place_limit_buy(&state, user, &symbol, 5, dollars(200))
        .await
        .unwrap(); // $1,000

    // Total locked should equal total order value
    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();
    assert_eq!(user_data.locked_money, dollars(3_000));
}

/// INV-004: Locked shares equal open sell order quantity
#[tokio::test]
async fn test_locked_shares_equals_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "LOCKSHARE", "Lock Share Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "LOCKSHAREUSER",
        "Lock Share User",
        dollars(10_000),
        vec![("LOCKSHARE".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Place multiple sell orders
    place_limit_sell(&state, user, &symbol, 20, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, user, &symbol, 30, dollars(110))
        .await
        .unwrap();

    // Total locked shares should equal total sell order quantity
    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();
    let position = user_data
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .unwrap();
    assert_eq!(position.locked_qty, 50);
}

/// INV-005: Available balance = money - locked
#[tokio::test]
async fn test_available_balance_calculation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AVAIL", "Available Co").await;

    let user = create_test_user_with_portfolio(
        &state,
        "AVAILUSER",
        "Available User",
        dollars(10_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place order locking some funds
    place_limit_buy(&state, user, &symbol, 10, dollars(500))
        .await
        .unwrap(); // Lock $5,000

    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();

    // Available should be total - locked
    let available = user_data.money; // money field is already available (total - locked)
    let locked = user_data.locked_money;

    // In the actual system, money = available, and total = money + locked_money
    // But the user struct stores: money (available) and locked_money separately
    assert_eq!(available, dollars(5_000), "Available should be $5,000");
    assert_eq!(locked, dollars(5_000), "Locked should be $5,000");
}

// =============================================================================
// SESSION STATE VERIFICATION (STATE-SESS-*)
// =============================================================================

/// STATE-SESS-001: Session tracking after create
#[tokio::test]
async fn test_session_state_after_create() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "SESSSTATE", "Session State User", "pass").await;

    let (session_id, _) = state.sessions.create_session(user_id);

    // Verify session info
    let session_info = state.sessions.get_session(session_id);
    assert!(session_info.is_some());
    let info = session_info.unwrap();
    assert_eq!(info.user_id, user_id);
    assert!(info.connected_at > 0);
}

/// STATE-SESS-002: User sessions list
#[tokio::test]
async fn test_user_sessions_list() {
    let config = TestConfig {
        max_sessions_per_user: 5, // Allow multiple
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "SESSLIST", "Session List User", "pass").await;

    // Create multiple sessions
    let (s1, _) = state.sessions.create_session(user_id);
    let (s2, _) = state.sessions.create_session(user_id);
    let (s3, _) = state.sessions.create_session(user_id);

    // Get user sessions
    let sessions = state.sessions.get_user_sessions(user_id);
    assert_eq!(sessions.len(), 3);
    assert!(sessions.contains(&s1));
    assert!(sessions.contains(&s2));
    assert!(sessions.contains(&s3));
}

// =============================================================================
// TOKEN STATE VERIFICATION (STATE-TOKEN-*)
// =============================================================================

/// STATE-TOKEN-001: Token state after multiple creates
#[tokio::test]
async fn test_token_state_multiple_creates() {
    let config = TestConfig {
        max_sessions_per_user: 3, // Allow multiple tokens
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "TOKSTATE", "Token State User", "pass").await;

    // Create multiple tokens
    let (token1, _) = state.tokens.create_token(user_id);
    let (token2, _) = state.tokens.create_token(user_id);
    let (token3, _) = state.tokens.create_token(user_id);

    // All should be valid
    assert_eq!(state.tokens.validate_token(&token1), Some(user_id));
    assert_eq!(state.tokens.validate_token(&token2), Some(user_id));
    assert_eq!(state.tokens.validate_token(&token3), Some(user_id));
}

/// STATE-TOKEN-002: Token revocation state
#[tokio::test]
async fn test_token_revocation_state() {
    let config = TestConfig {
        max_sessions_per_user: 2, // Allow 2 tokens max
        ..Default::default()
    };
    let state = create_test_state_with_config(config).await;

    let user_id = create_test_user(&state, "TOKREV", "Token Revoke User", "pass").await;

    // Create 3 tokens (third should revoke first)
    let (token1, revoked1) = state.tokens.create_token(user_id);
    assert!(revoked1.is_empty());

    let (token2, revoked2) = state.tokens.create_token(user_id);
    assert!(revoked2.is_empty());

    // Third token should revoke first
    let (token3, revoked3) = state.tokens.create_token(user_id);
    assert!(!revoked3.is_empty());
    assert!(revoked3.contains(&token1));

    // Token1 should be invalid, token2 and token3 valid
    assert_eq!(state.tokens.validate_token(&token1), None);
    assert_eq!(state.tokens.validate_token(&token2), Some(user_id));
    assert_eq!(state.tokens.validate_token(&token3), Some(user_id));
}
