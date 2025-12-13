//! State Machine & Invariant Integration Tests
//!
//! Tests for: SM-ORDER-*, SM-MARKET-*, INV-FIN-*, INV-BOOK-*, INV-TRADE-*
//!
//! These tests verify state machine transitions and system invariants.

mod common;

use common::*;
use stockmart_backend::domain::models::OrderStatus;

// =============================================================================
// ORDER STATE MACHINE TESTS (SM-ORDER-*)
// =============================================================================

/// SM-ORDER-001: New order starts in Open state
#[tokio::test]
async fn test_order_starts_open() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "SM1",
        "State Machine User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify order is in Open state
    let order = state.orders.get_order(order_id);
    assert!(order.is_some(), "Order should exist");
    let order = order.unwrap();

    assert_eq!(order.status, OrderStatus::Open, "New order should be Open");
    assert_eq!(order.filled_qty, 0, "Filled qty should be 0");
}

/// SM-ORDER-002: Partial fill transitions to Partial state
#[tokio::test]
async fn test_order_partial_fill_state() {
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

    // Buyer wants 100 shares
    let buy_order_id = place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Seller only has 50
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Buy order should be partially filled
    let order = state.orders.get_order(buy_order_id);
    assert!(order.is_some(), "Order should still exist");
    let order = order.unwrap();

    assert_eq!(
        order.status,
        OrderStatus::Partial,
        "Order should be Partial after partial fill"
    );
    assert_eq!(order.filled_qty, 50, "Should have 50 filled");
    assert_eq!(order.qty - order.filled_qty, 50, "Should have 50 remaining");
}

/// SM-ORDER-003: Full fill transitions to Filled state
#[tokio::test]
async fn test_order_full_fill_state() {
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

    // Place matching orders
    place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();
    let buy_order_id = place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Buy order should be fully filled (may be removed from OrdersService)
    let order = state.orders.get_order(buy_order_id);

    // Filled orders may be removed from active orders
    match order {
        Some(o) => {
            assert_eq!(o.status, OrderStatus::Filled, "Order should be Filled");
            assert_eq!(o.filled_qty, o.qty, "Should be fully filled");
        }
        None => {
            // Order was removed after being filled - also valid
        }
    }
}

/// SM-ORDER-004: Cancel transitions to Cancelled state
#[tokio::test]
async fn test_order_cancel_state() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user =
        create_test_user_with_portfolio(&state, "CANCEL", "Cancel User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Cancel the order
    let cancelled = state
        .engine
        .cancel_order(user, &symbol, order_id)
        .await
        .unwrap();

    assert_eq!(
        cancelled.status,
        OrderStatus::Cancelled,
        "Cancelled order should have Cancelled status"
    );
}

/// SM-ORDER-005: Cannot transition from Filled to any other state
#[tokio::test]
async fn test_filled_order_immutable() {
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

    // Create and fill order
    let sell_order_id = place_limit_sell(&state, seller, &symbol, 100, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Try to cancel filled order - should fail or order not found
    let cancel_result = state
        .engine
        .cancel_order(seller, &symbol, sell_order_id)
        .await;

    // Either error (can't cancel filled) or order not found (already removed)
    assert!(cancel_result.is_err(), "Cannot cancel a filled order");
}

/// SM-ORDER-006: Cannot transition from Cancelled
#[tokio::test]
async fn test_cancelled_order_immutable() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "IMMUT",
        "Immutable User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Cancel the order
    state
        .engine
        .cancel_order(user, &symbol, order_id)
        .await
        .unwrap();

    // Try to cancel again - should fail
    let result = state.engine.cancel_order(user, &symbol, order_id).await;
    assert!(result.is_err(), "Cannot cancel an already cancelled order");
}

// =============================================================================
// MARKET STATE MACHINE TESTS (SM-MARKET-*)
// =============================================================================

/// SM-MARKET-001: Market starts open (engine default)
/// NOTE: The engine initializes with market open. This is the current design choice.
#[tokio::test]
async fn test_market_starts_open() {
    let state = create_test_state().await;

    // Market starts open by default in the engine (is_open: AtomicBool::new(true))
    assert!(
        state.engine.is_market_open(),
        "Market should start open by default"
    );
}

/// SM-MARKET-002: Market can be opened
#[tokio::test]
async fn test_market_can_be_opened() {
    let state = create_test_state().await;

    state.engine.set_market_open(true);
    assert!(
        state.engine.is_market_open(),
        "Market should be open after toggle"
    );
}

/// SM-MARKET-003: Market can be closed
#[tokio::test]
async fn test_market_can_be_closed() {
    let state = create_test_state().await;

    state.engine.set_market_open(true);
    assert!(state.engine.is_market_open());

    state.engine.set_market_open(false);
    assert!(
        !state.engine.is_market_open(),
        "Market should be closed after toggle"
    );
}

/// SM-MARKET-004: Orders rejected when market closed
#[tokio::test]
async fn test_orders_rejected_market_closed() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "CLOSED",
        "Closed Market User",
        dollars(100_000),
        vec![],
    )
    .await;

    // Close the market explicitly
    state.engine.set_market_open(false);
    assert!(!state.engine.is_market_open());

    let result = place_limit_buy(&state, user, &symbol, 10, dollars(100)).await;
    assert!(
        result.is_err(),
        "Orders should be rejected when market is closed"
    );
}

/// SM-MARKET-005: Orders accepted when market open
#[tokio::test]
async fn test_orders_accepted_market_open() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "OPEN",
        "Open Market User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let result = place_limit_buy(&state, user, &symbol, 10, dollars(100)).await;
    assert!(
        result.is_ok(),
        "Orders should be accepted when market is open"
    );
}

// =============================================================================
// FINANCIAL INVARIANTS (INV-FIN-*)
// =============================================================================

/// INV-FIN-001: User money >= 0 always
#[tokio::test]
async fn test_invariant_money_non_negative() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 1000)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Execute many trades
    for i in 0..10 {
        place_limit_sell(&state, seller, &symbol, 10, dollars(100) + i)
            .await
            .unwrap();
        place_limit_buy(&state, buyer, &symbol, 10, dollars(100) + i)
            .await
            .unwrap();
    }

    // Verify both users have non-negative money
    let seller_data = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_data = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    assert!(
        seller_data.money >= 0,
        "Seller money should be non-negative"
    );
    assert!(buyer_data.money >= 0, "Buyer money should be non-negative");
    assert!(
        seller_data.locked_money >= 0,
        "Seller locked_money should be non-negative"
    );
    assert!(
        buyer_data.locked_money >= 0,
        "Buyer locked_money should be non-negative"
    );
}

/// INV-FIN-002: locked_money <= money (available for orders)
#[tokio::test]
async fn test_invariant_locked_money_bounded() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user =
        create_test_user_with_portfolio(&state, "LOCK", "Lock Test User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Place multiple orders to lock money
    for i in 0..5 {
        place_limit_buy(&state, user, &symbol, 10, dollars(100) + i)
            .await
            .unwrap();
    }

    let user_data = state.user_repo.find_by_id(user).await.unwrap().unwrap();

    // Locked money should not exceed total money
    assert!(
        user_data.locked_money <= user_data.money,
        "Locked money {} should not exceed total money {}",
        user_data.locked_money,
        user_data.money
    );
}

/// INV-FIN-003: Cannot spend more than available
#[tokio::test]
async fn test_invariant_cannot_overspend() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let user = create_test_user_with_portfolio(
        &state,
        "OVERSPEND",
        "Overspend User",
        dollars(1_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Try to place order exceeding available funds
    let result = place_limit_buy(&state, user, &symbol, 100, dollars(100)).await;

    // Should be rejected - would cost $10,000 but only have $1,000
    assert!(result.is_err(), "Should not be able to overspend");
}

/// INV-FIN-004: Trade preserves total value in system
#[tokio::test]
async fn test_invariant_trade_preserves_value() {
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

    // Calculate initial total value
    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_before = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let trade_price = dollars(100);
    let trade_qty = 50u64;

    // Estimate initial total: seller money + buyer money
    // (ignoring stock value for simplicity - just checking money conservation)
    let total_money_before = seller_before.money + buyer_before.money;

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, trade_qty, trade_price)
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, trade_qty, trade_price)
        .await
        .unwrap();

    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let total_money_after = seller_after.money + buyer_after.money;

    // Total money in system should be preserved (money transferred, not created/destroyed)
    assert_eq!(
        total_money_before, total_money_after,
        "Total money should be preserved: before={}, after={}",
        total_money_before, total_money_after
    );
}

// =============================================================================
// ORDER BOOK INVARIANTS (INV-BOOK-*)
// =============================================================================

/// INV-BOOK-001: Bids sorted descending by price
#[tokio::test]
async fn test_invariant_bids_sorted_descending() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SORTED", "Sorted Book Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "SORT",
        "Sort Test User",
        dollars(1_000_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place bids at various prices (out of order)
    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(105))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(95))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 10, dollars(110))
        .await
        .unwrap();

    let depth = state.engine.get_order_book_depth(&symbol, 10);
    let (bids, _) = depth.unwrap();

    // Verify bids are sorted descending
    for i in 0..bids.len() - 1 {
        assert!(
            bids[i].0 >= bids[i + 1].0,
            "Bids should be sorted descending: {} >= {}",
            bids[i].0,
            bids[i + 1].0
        );
    }
}

/// INV-BOOK-002: Asks sorted ascending by price
#[tokio::test]
async fn test_invariant_asks_sorted_ascending() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SORTED", "Sorted Book Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "SORT",
        "Sort Test User",
        dollars(10_000),
        vec![("SORTED".to_string(), 1000)],
    )
    .await;

    open_market(&state);

    // Place asks at various prices (out of order)
    place_limit_sell(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, user, &symbol, 10, dollars(95))
        .await
        .unwrap();
    place_limit_sell(&state, user, &symbol, 10, dollars(110))
        .await
        .unwrap();
    place_limit_sell(&state, user, &symbol, 10, dollars(105))
        .await
        .unwrap();

    let depth = state.engine.get_order_book_depth(&symbol, 10);
    let (_, asks) = depth.unwrap();

    // Verify asks are sorted ascending
    for i in 0..asks.len() - 1 {
        assert!(
            asks[i].0 <= asks[i + 1].0,
            "Asks should be sorted ascending: {} <= {}",
            asks[i].0,
            asks[i + 1].0
        );
    }
}

/// INV-BOOK-003: No crossed book (best_bid < best_ask) after matching
#[tokio::test]
async fn test_invariant_no_crossed_book() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CROSS", "Cross Test Co.").await;

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

    // Create orders that would cross
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 5, dollars(105))
        .await
        .unwrap(); // Crosses at $100

    // After matching, remaining orders should not cross
    let depth = state.engine.get_order_book_depth(&symbol, 10);
    let (bids, asks) = depth.unwrap();

    if !bids.is_empty() && !asks.is_empty() {
        let best_bid = bids[0].0;
        let best_ask = asks[0].0;
        assert!(
            best_bid < best_ask,
            "Book should not be crossed: best_bid {} < best_ask {}",
            best_bid,
            best_ask
        );
    }
}

/// INV-BOOK-004: Order quantities positive
#[tokio::test]
async fn test_invariant_order_quantities_positive() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "QTY", "Quantity Test Co.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "QTY",
        "Quantity Test User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    place_limit_buy(&state, user, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, user, &symbol, 20, dollars(99))
        .await
        .unwrap();

    let depth = state.engine.get_order_book_depth(&symbol, 10);
    let (bids, _) = depth.unwrap();

    for (price, qty) in bids {
        assert!(
            qty > 0,
            "Order quantity at price {} should be positive",
            price
        );
    }
}

// =============================================================================
// TRADE INVARIANTS (INV-TRADE-*)
// =============================================================================

/// INV-TRADE-001: Trade has valid participants
#[tokio::test]
async fn test_invariant_trade_valid_participants() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRADE", "Trade Test Co.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("TRADE".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    // Subscribe to trades
    let mut trade_rx = state.engine.subscribe_trades();

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Get the trade
    let trade = tokio::time::timeout(tokio::time::Duration::from_millis(100), trade_rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Verify both participants exist
    // Trade uses maker_user_id and taker_user_id
    let maker_exists = state
        .user_repo
        .find_by_id(trade.maker_user_id)
        .await
        .unwrap()
        .is_some();
    let taker_exists = state
        .user_repo
        .find_by_id(trade.taker_user_id)
        .await
        .unwrap()
        .is_some();

    assert!(maker_exists, "Trade maker should exist");
    assert!(taker_exists, "Trade taker should exist");
}

/// INV-TRADE-002: Trade price within order limits
#[tokio::test]
async fn test_invariant_trade_price_valid() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PRICE", "Price Test Co.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("PRICE".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    let mut trade_rx = state.engine.subscribe_trades();

    open_market(&state);

    let sell_price = dollars(100);
    let buy_price = dollars(105); // Willing to pay more

    place_limit_sell(&state, seller, &symbol, 10, sell_price)
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, buy_price)
        .await
        .unwrap();

    let trade = tokio::time::timeout(tokio::time::Duration::from_millis(100), trade_rx.recv())
        .await
        .unwrap()
        .unwrap();

    // Trade price should be at the resting order's price (seller's ask)
    assert_eq!(
        trade.price, sell_price,
        "Trade should execute at ask price: {} == {}",
        trade.price, sell_price
    );
}

/// INV-TRADE-003: Trade quantity positive
#[tokio::test]
async fn test_invariant_trade_quantity_positive() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRADEQTY", "Trade Qty Co.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("TRADEQTY".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    let mut trade_rx = state.engine.subscribe_trades();

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    let trade = tokio::time::timeout(tokio::time::Duration::from_millis(100), trade_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert!(trade.qty > 0, "Trade quantity should be positive");
}

/// INV-TRADE-004: Trade updates portfolios correctly
#[tokio::test]
async fn test_invariant_trade_updates_portfolios() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PORT", "Portfolio Test Co.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("PORT".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    let seller_before = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_before = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let seller_shares_before = seller_before
        .portfolio
        .iter()
        .find(|p| p.symbol == "PORT")
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer_shares_before = buyer_before
        .portfolio
        .iter()
        .find(|p| p.symbol == "PORT")
        .map(|p| p.qty)
        .unwrap_or(0);

    open_market(&state);

    let trade_qty = 50u64;
    place_limit_sell(&state, seller, &symbol, trade_qty, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, trade_qty, dollars(100))
        .await
        .unwrap();

    let seller_after = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_after = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let seller_shares_after = seller_after
        .portfolio
        .iter()
        .find(|p| p.symbol == "PORT")
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer_shares_after = buyer_after
        .portfolio
        .iter()
        .find(|p| p.symbol == "PORT")
        .map(|p| p.qty)
        .unwrap_or(0);

    // Seller should have fewer shares
    assert_eq!(
        seller_shares_after,
        seller_shares_before - trade_qty,
        "Seller shares should decrease by trade qty"
    );

    // Buyer should have more shares
    assert_eq!(
        buyer_shares_after,
        buyer_shares_before + trade_qty,
        "Buyer shares should increase by trade qty"
    );
}

// =============================================================================
// SESSION STATE INVARIANTS
// =============================================================================

/// Test: Token count consistency
#[tokio::test]
async fn test_invariant_token_count_consistency() {
    let state = create_test_state().await;

    // Create multiple users with tokens
    let user1 = create_test_user(&state, "U1", "User 1", "pass").await;
    let user2 = create_test_user(&state, "U2", "User 2", "pass").await;

    let (token1, _) = state.tokens.create_token(user1);
    let (token2, _) = state.tokens.create_token(user2);

    // Both tokens should be valid
    assert!(state.tokens.validate_token(&token1).is_some());
    assert!(state.tokens.validate_token(&token2).is_some());

    // Revoke one token
    state.tokens.revoke_token(&token1);

    // Only one should be valid now
    assert!(state.tokens.validate_token(&token1).is_none());
    assert!(state.tokens.validate_token(&token2).is_some());
}

// =============================================================================
// COMPANY STATE INVARIANTS
// =============================================================================

/// Test: Company symbol uniqueness maintained
#[tokio::test]
async fn test_invariant_symbol_uniqueness() {
    let state = create_test_state().await;

    // Create company
    create_test_company(&state, "UNIQUE", "Unique Co.").await;

    // Try to create another with same symbol
    let result = state
        .admin
        .create_company(
            "UNIQUE".to_string(),
            "Duplicate Co.".to_string(),
            "Tech".to_string(),
            100,
        )
        .await;

    assert!(result.is_err(), "Duplicate symbols should be rejected");
}

/// Test: All companies have orderbooks
#[tokio::test]
async fn test_invariant_companies_have_orderbooks() {
    let state = create_test_state().await;

    // Create multiple companies
    let symbols = vec!["AAPL", "GOOGL", "MSFT"];
    for sym in &symbols {
        create_test_company(&state, sym, &format!("{} Inc", sym)).await;
    }

    // All should have orderbooks
    for sym in &symbols {
        let depth = state.engine.get_order_book_depth(sym, 1);
        assert!(depth.is_some(), "Company {} should have an orderbook", sym);
    }
}
