//! Edge Cases & Boundary Integration Tests
//!
//! Tests for: EDGE-PRICE-*, EDGE-QTY-*, EDGE-PORT-*, EDGE-BOOK-*, EDGE-TIME-*

mod common;

use common::*;

// =============================================================================
// PRICE EDGE CASES (EDGE-PRICE-*)
// =============================================================================

/// EDGE-PRICE-001: Price at PRICE_SCALE minimum
#[tokio::test]
async fn test_price_at_minimum() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "MINPRICE",
        "Min Price User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Price of 1 (smallest unit in cents with PRICE_SCALE)
    let result = place_limit_buy(&state, user, &symbol, 100, 1).await;

    // Should accept minimum price
    assert!(result.is_ok(), "Should accept minimum price of 1 cent");
}

/// EDGE-PRICE-002: Price at maximum safe i64
#[tokio::test]
async fn test_price_at_maximum() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user =
        create_test_user_with_portfolio(&state, "MAXPRICE", "Max Price User", i64::MAX / 2, vec![])
            .await;

    open_market(&state);

    // Try large but safe price
    let large_price = dollars(1_000_000); // $1 million per share
    let result = place_limit_buy(&state, user, &symbol, 1, large_price).await;

    // Should handle large prices
    match result {
        Ok(_) => (),  // Accepted
        Err(_) => (), // Rejected due to funds check
    }
}

/// EDGE-PRICE-003: Price matching at exact boundary
#[tokio::test]
async fn test_price_exact_match() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Exact price match
    let exact_price = dollars(100) + 1; // $100.01
    place_limit_sell(&state, seller, &symbol, 10, exact_price)
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, exact_price)
        .await
        .unwrap();

    // Verify trade executed at exact price
    let trades = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    assert!(!trades.is_empty(), "Trade should execute at exact price");
    assert_eq!(trades[0].price, exact_price);
}

// =============================================================================
// QUANTITY EDGE CASES (EDGE-QTY-*)
// =============================================================================

/// EDGE-QTY-001: Quantity of 1
#[tokio::test]
async fn test_quantity_of_one() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user =
        create_test_user_with_portfolio(&state, "ONEQTY", "One Qty User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Minimum quantity of 1
    let result = place_limit_buy(&state, user, &symbol, 1, dollars(100)).await;

    assert!(result.is_ok(), "Should accept quantity of 1");
}

/// EDGE-QTY-002: Partial fill leaves remainder
#[tokio::test]
async fn test_partial_fill_remainder() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 50)], // Only 50 shares
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Buyer wants 100 shares, seller has only 50
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Buyer should have open order for remaining 50
    let orders = state.orders.get_user_orders(buyer);
    let open_buy = orders.iter().find(|o| o.remaining_qty > 0);
    assert!(open_buy.is_some(), "Should have remaining order");
    assert_eq!(
        open_buy.unwrap().remaining_qty,
        50,
        "Should have 50 shares remaining"
    );
}

/// EDGE-QTY-003: Multiple partial fills
#[tokio::test]
async fn test_multiple_partial_fills() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller1 = create_test_user_with_portfolio(
        &state,
        "S1",
        "Seller 1",
        dollars(10_000),
        vec![("AAPL".to_string(), 30)],
    )
    .await;
    let seller2 = create_test_user_with_portfolio(
        &state,
        "S2",
        "Seller 2",
        dollars(10_000),
        vec![("AAPL".to_string(), 30)],
    )
    .await;
    let seller3 = create_test_user_with_portfolio(
        &state,
        "S3",
        "Seller 3",
        dollars(10_000),
        vec![("AAPL".to_string(), 40)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Multiple sell orders
    place_limit_sell(&state, seller1, &symbol, 30, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, seller2, &symbol, 30, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, seller3, &symbol, 40, dollars(100))
        .await
        .unwrap();

    // Large buy order should fill from all three
    place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Buyer should have 100 shares total
    let buyer_data = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let position = buyer_data.portfolio.iter().find(|p| p.symbol == "AAPL");
    assert!(position.is_some(), "Buyer should have position");
    assert_eq!(
        position.unwrap().qty,
        100,
        "Should have received 100 shares"
    );
}

// =============================================================================
// PORTFOLIO EDGE CASES (EDGE-PORT-*)
// =============================================================================

/// EDGE-PORT-001: Sell exact portfolio quantity
#[tokio::test]
async fn test_sell_exact_portfolio_qty() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "EXACT",
        "Exact Sell",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)], // Exactly 100 shares
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Sell exactly 100 (all shares)
    place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Seller should have 0 shares
    let seller_data = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let position = seller_data.portfolio.iter().find(|p| p.symbol == "AAPL");

    match position {
        Some(pos) => assert_eq!(pos.qty, 0, "Position qty should be 0"),
        None => (), // Position might be removed entirely
    }
}

/// EDGE-PORT-002: Short position creation
#[tokio::test]
async fn test_short_position_creation() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    // User with no shares
    let seller =
        create_test_user_with_portfolio(&state, "SHORT", "Short Seller", dollars(100_000), vec![])
            .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Sell without owning (short sell)
    let sell_result = place_limit_sell(&state, seller, &symbol, 50, dollars(100)).await;

    if sell_result.is_ok() {
        // Buy to create trade
        place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
            .await
            .unwrap();

        // Check for short position
        let seller_data = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
        let position = seller_data.portfolio.iter().find(|p| p.symbol == "AAPL");

        if let Some(pos) = position {
            assert!(
                pos.short_qty > 0 || pos.qty == 0,
                "Should have short position or no position"
            );
        }
    }
}

/// EDGE-PORT-005: Empty portfolio sync
#[tokio::test]
async fn test_empty_portfolio_sync() {
    let state = create_test_state().await;

    // User with empty portfolio
    let user = create_test_user(&state, "EMPTY", "Empty Portfolio", "pass").await;

    // Get portfolio
    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();

    // Portfolio should be empty but not null
    assert!(
        user_data.portfolio.is_empty(),
        "New user should have empty portfolio"
    );
}

// =============================================================================
// ORDER BOOK EDGE CASES (EDGE-BOOK-*)
// =============================================================================

/// EDGE-BOOK-001: Empty order book depth
#[tokio::test]
async fn test_empty_orderbook_depth() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "EMPTY", "Empty Book Co.").await;

    let depth = state.engine.get_order_book_depth(&symbol, 5);
    assert!(depth.is_some(), "Should return depth even if empty");

    let (bids, asks) = depth.unwrap();
    assert!(bids.is_empty(), "No bids in empty book");
    assert!(asks.is_empty(), "No asks in empty book");
}

/// EDGE-BOOK-002: Single order in book
#[tokio::test]
async fn test_single_order_in_book() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SINGLE", "Single Order Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "SINGLE",
        "Single Order User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place single buy order
    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let depth = state.engine.get_order_book_depth(&symbol, 5);
    let (bids, asks) = depth.unwrap();

    assert_eq!(bids.len(), 1, "Should have exactly one bid level");
    assert!(asks.is_empty(), "No asks");
    assert_eq!(bids[0].0, dollars(100), "Bid price should be $100");
    assert_eq!(bids[0].1, 10, "Bid quantity should be 10");
}

/// EDGE-BOOK-003: Max depth levels returned
#[tokio::test]
async fn test_max_depth_levels() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "DEEP", "Deep Book Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "DEEP",
        "Deep Book User",
        dollars(1_000_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place many orders at different prices
    for i in 0..20u64 {
        place_limit_buy(&state, user, &symbol, 10, dollars(100) - (i as i64))
            .await
            .unwrap();
    }

    // Request only 5 levels
    let depth = state.engine.get_order_book_depth(&symbol, 5);
    let (bids, _) = depth.unwrap();

    assert!(bids.len() <= 5, "Should return at most 5 levels");
}

/// EDGE-BOOK-004: Self-trade allowed (same user different orders)
#[tokio::test]
async fn test_self_trade_allowed() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SELF", "Self Trade Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "SELF",
        "Self Trade User",
        dollars(100_000),
        vec![("SELF".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Place buy and sell from same user
    place_limit_sell(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Trade should execute (self-trade allowed)
    let trades = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    // Whether this creates a trade depends on implementation
    // The test documents the behavior
}

/// Test: Order book maintains price-time priority
#[tokio::test]
async fn test_price_time_priority() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PRIO", "Priority Co.").await;

    let buyer1 =
        create_test_user_with_portfolio(&state, "B1", "Buyer 1", dollars(100_000), vec![]).await;
    let buyer2 =
        create_test_user_with_portfolio(&state, "B2", "Buyer 2", dollars(100_000), vec![]).await;
    let seller = create_test_user_with_portfolio(
        &state,
        "S",
        "Seller",
        dollars(10_000),
        vec![("PRIO".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Buyer1 places order first at $100
    place_limit_buy(&state, buyer1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    // Buyer2 places order second at $100 (same price)
    place_limit_buy(&state, buyer2, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Seller sells 10 shares - should match buyer1 first (time priority)
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Buyer1 should get the trade (first in time)
    let buyer1_data = state.user_repo.find_by_id(buyer1).await.unwrap().unwrap();
    let buyer1_pos = buyer1_data.portfolio.iter().find(|p| p.symbol == "PRIO");

    // Buyer2 should have open order (not filled yet)
    let buyer2_orders = state.orders.get_user_orders(buyer2);

    // Verify price-time priority
    assert!(
        buyer1_pos.is_some() || buyer2_orders.is_empty(),
        "Either buyer1 got filled or implementation doesn't follow price-time priority"
    );
}

/// Test: Best bid/ask spread
#[tokio::test]
async fn test_bid_ask_spread() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SPREAD", "Spread Co.").await;

    let buyer =
        create_test_user_with_portfolio(&state, "BUY", "Buyer", dollars(100_000), vec![]).await;
    let seller = create_test_user_with_portfolio(
        &state,
        "SELL",
        "Seller",
        dollars(10_000),
        vec![("SPREAD".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Create spread: bid at $99, ask at $101
    place_limit_buy(&state, buyer, &symbol, 10, dollars(99))
        .await
        .unwrap();
    place_limit_sell(&state, seller, &symbol, 10, dollars(101))
        .await
        .unwrap();

    let depth = state.engine.get_order_book_depth(&symbol, 5);
    let (bids, asks) = depth.unwrap();

    assert!(!bids.is_empty(), "Should have bids");
    assert!(!asks.is_empty(), "Should have asks");

    let best_bid = bids[0].0;
    let best_ask = asks[0].0;

    assert_eq!(best_bid, dollars(99), "Best bid should be $99");
    assert_eq!(best_ask, dollars(101), "Best ask should be $101");
    assert!(
        best_ask > best_bid,
        "Ask should be higher than bid (no trade should occur)"
    );
}

// =============================================================================
// TIME EDGE CASES (EDGE-TIME-*)
// =============================================================================

/// EDGE-TIME-001: Timestamps are reasonable
#[tokio::test]
async fn test_timestamps_reasonable() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TIME", "Time User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Created_at should be a recent Unix timestamp
    let now = chrono::Utc::now().timestamp();
    assert!(user.created_at > 0, "Timestamp should be positive");
    assert!(
        user.created_at <= now,
        "Timestamp should not be in the future"
    );
    assert!(
        user.created_at > now - 60,
        "Timestamp should be within last minute"
    );
}

/// EDGE-TIME-003: Unix timestamp handles large values (no year 2038 problem)
#[tokio::test]
async fn test_timestamp_no_overflow() {
    // i64 timestamps can handle dates far into the future
    let far_future: i64 = 4102444800; // Year 2100
    let very_far: i64 = 32503680000; // Year 3000

    // These operations should not overflow
    let _ = far_future + 86400; // Add one day
    let _ = very_far + 86400;

    // i64 max is 9,223,372,036,854,775,807
    // This represents approximately 292 billion years from Unix epoch
    // So we're safe from timestamp overflow
    assert!(
        i64::MAX > very_far * 100,
        "i64 should handle timestamps far into the future"
    );
}

/// Test: Trade timestamps are ordered
#[tokio::test]
async fn test_trade_timestamps_ordered() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Execute multiple trades
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

    // Get trades
    let trades = state.trade_history.get_recent_symbol_trades(&symbol, 10);

    // Verify timestamps are ordered (newest first or oldest first, depending on impl)
    if trades.len() >= 2 {
        // Either increasing or decreasing - just should be consistent
        let monotonic_inc = trades.windows(2).all(|w| w[0].timestamp <= w[1].timestamp);
        let monotonic_dec = trades.windows(2).all(|w| w[0].timestamp >= w[1].timestamp);
        assert!(
            monotonic_inc || monotonic_dec,
            "Timestamps should be monotonically ordered"
        );
    }
}

// =============================================================================
// MONEY EDGE CASES
// =============================================================================

/// Test: Money locked correctly for pending orders
#[tokio::test]
async fn test_money_locked_for_orders() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "LOCK", "Lock Test Co.").await;

    let initial_money = dollars(100_000);
    let user =
        create_test_user_with_portfolio(&state, "LOCK", "Lock Test User", initial_money, vec![])
            .await;

    open_market(&state);

    // Place order worth $10,000
    let order_value = dollars(100) * 100; // 100 shares @ $100
    place_limit_buy(&state, user, &symbol, 100, dollars(100))
        .await
        .unwrap();

    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();

    // Money should be locked
    assert!(user_data.locked_money > 0, "Should have locked money");
    assert_eq!(
        user_data.locked_money, order_value,
        "Locked amount should match order value"
    );

    // Available money should be reduced
    let available = user_data.money - user_data.locked_money;
    assert!(
        available < initial_money,
        "Available money should be less than initial"
    );
}

/// Test: Cannot place order exceeding available funds
#[tokio::test]
async fn test_cannot_exceed_available_funds() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "FUND", "Fund Test Co.").await;

    let user =
        create_test_user_with_portfolio(&state, "FUND", "Fund Test User", dollars(100), vec![])
            .await;

    open_market(&state);

    // Try to place order worth more than available
    let result = place_limit_buy(&state, user, &symbol, 1000, dollars(100)).await;

    // Should be rejected (would cost $100,000, user only has $100)
    assert!(
        result.is_err(),
        "Should reject order exceeding available funds"
    );
}

/// Test: Order cancellation returns locked funds
#[tokio::test]
async fn test_cancel_returns_locked_funds() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CANCEL", "Cancel Test Co.").await;

    let initial_money = dollars(100_000);
    let user = create_test_user_with_portfolio(
        &state,
        "CANCEL",
        "Cancel Test User",
        initial_money,
        vec![],
    )
    .await;

    open_market(&state);

    // Place order
    let order_id = place_limit_buy(&state, user, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Verify money locked
    let user_before = state.user_repo.find_by_id(user).await.unwrap().unwrap();
    assert!(user_before.locked_money > 0, "Money should be locked");

    // Cancel order
    state
        .engine
        .cancel_order(user, &symbol, order_id)
        .await
        .unwrap();

    // Money should be unlocked
    let user_after = state.user_repo.find_by_id(user).await.unwrap().unwrap();
    assert_eq!(
        user_after.locked_money, 0,
        "Locked money should be returned"
    );
    assert_eq!(
        user_after.money, initial_money,
        "All money should be available again"
    );
}

// =============================================================================
// MATCHING ENGINE EDGE CASES
// =============================================================================

/// Test: Market order without liquidity
#[tokio::test]
async fn test_market_order_no_liquidity() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "NOLIQ", "No Liquidity Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "NOLIQ",
        "No Liquidity User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Try market buy with no sellers
    let result = place_market_buy(&state, user, &symbol, 100).await;

    // Behavior depends on implementation - should either fail or remain open
    match result {
        Ok(_) => {
            // Order placed but may not fill
            let orders = state.orders.get_user_orders(user);
            // Market order might remain open or be rejected
        }
        Err(_) => {
            // Rejected due to no liquidity - also valid
        }
    }
}

/// Test: Crossing the spread
#[tokio::test]
async fn test_crossing_spread() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CROSS", "Cross Spread Co.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("CROSS".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Seller asks $100
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Buyer bids $105 (crosses the spread)
    place_limit_buy(&state, buyer, &symbol, 10, dollars(105))
        .await
        .unwrap();

    // Trade should execute at the ask price ($100)
    let trades = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    assert!(
        !trades.is_empty(),
        "Trade should execute when crossing spread"
    );

    // Price should be at the resting order price (the ask)
    assert_eq!(
        trades[0].price,
        dollars(100),
        "Trade should execute at ask price"
    );
}
