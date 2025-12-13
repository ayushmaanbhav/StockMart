//! Trading Engine Integration Tests
//!
//! Tests for: TRADE-PLACE-*, TRADE-CANCEL-*, BOOK-*, SETTLE-*, SM-ORDER-*

mod common;

use common::*;

// =============================================================================
// ORDER PLACEMENT TESTS (TRADE-PLACE-*)
// =============================================================================

/// TRADE-PLACE-001: Place limit buy order (market open)
#[tokio::test]
async fn test_place_limit_buy_market_open() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id =
        create_test_user_with_portfolio(&state, "BUYER001", "Buyer", dollars(100_000), vec![])
            .await;

    open_market(&state);

    let result = place_limit_buy(&state, user_id, &symbol, 10, dollars(100)).await;
    assert!(result.is_ok(), "Order should succeed: {:?}", result);

    let order_id = result.unwrap();
    assert!(order_id > 0);

    // Verify money locked
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user.locked_money, dollars(1_000)); // 10 * $100
}

/// TRADE-PLACE-002: Place limit sell order (market open)
#[tokio::test]
async fn test_place_limit_sell_market_open() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "SELLER001",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    let result = place_limit_sell(&state, user_id, &symbol, 50, dollars(150)).await;
    assert!(result.is_ok(), "Order should succeed: {:?}", result);

    // Verify shares locked
    assert_user_position(&state, user_id, &symbol, 100, 50).await;
}

/// TRADE-PLACE-005: Place order (market closed) should fail
#[tokio::test]
async fn test_place_order_market_closed() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "CLOSEDMKT",
        "Closed Market User",
        dollars(100_000),
        vec![],
    )
    .await;

    close_market(&state);

    let result = place_limit_buy(&state, user_id, &symbol, 10, dollars(100)).await;
    assert!(result.is_err(), "Order should fail when market is closed");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("MARKET_CLOSED") || err_msg.to_lowercase().contains("closed"),
        "Error should mention market closed"
    );
}

/// TRADE-PLACE-007: Place order for non-existent symbol should fail
#[tokio::test]
async fn test_place_order_nonexistent_symbol() {
    let state = create_test_state().await;

    let user_id = create_test_user_with_portfolio(
        &state,
        "NOSYMBOL",
        "No Symbol User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let result = place_limit_buy(&state, user_id, "FAKE", 10, dollars(100)).await;
    assert!(result.is_err(), "Order should fail for non-existent symbol");
}

/// TRADE-PLACE-008: Place buy order with insufficient funds should fail
#[tokio::test]
async fn test_place_buy_insufficient_funds() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "POORBUYER",
        "Poor Buyer",
        dollars(100), // Only $100
        vec![],
    )
    .await;

    open_market(&state);

    // Try to buy $10,000 worth of stock
    let result = place_limit_buy(&state, user_id, &symbol, 100, dollars(100)).await;
    assert!(result.is_err(), "Order should fail with insufficient funds");
    assert!(
        result.unwrap_err().to_lowercase().contains("insufficient"),
        "Error should mention insufficient funds"
    );
}

/// TRADE-PLACE-009: Place sell order with insufficient shares should fail
#[tokio::test]
async fn test_place_sell_insufficient_shares() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "POORSELLER",
        "Poor Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 10)], // Only 10 shares
    )
    .await;

    open_market(&state);

    // Try to sell 100 shares
    let result = place_limit_sell(&state, user_id, &symbol, 100, dollars(100)).await;
    assert!(
        result.is_err(),
        "Order should fail with insufficient shares"
    );
}

/// TRADE-PLACE-011: Place order with qty=0
/// NOTE: Current implementation does not validate qty>0.
/// This test documents current behavior - qty=0 orders are accepted but have no effect.
#[tokio::test]
async fn test_place_order_zero_qty() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "ZEROQTY",
        "Zero Qty User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Current behavior: qty=0 orders are accepted (no validation)
    let result = place_limit_buy(&state, user_id, &symbol, 0, dollars(100)).await;
    // Document current behavior - this is a known gap in validation
    assert!(
        result.is_ok(),
        "Currently qty=0 orders are accepted (missing validation)"
    );
}

/// TRADE-PLACE-012: Place limit order with price<=0
/// NOTE: Current implementation does not validate price>0.
/// This test documents current behavior.
#[tokio::test]
async fn test_place_order_zero_price() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "ZEROPRICE",
        "Zero Price User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Current behavior: price=0 orders are accepted (no validation)
    let result = place_limit_buy(&state, user_id, &symbol, 10, 0).await;
    // Document current behavior - this is a known gap in validation
    assert!(
        result.is_ok(),
        "Currently price=0 orders are accepted (missing validation)"
    );
}

/// TRADE-PLACE-017: Verify locked_money updated on buy order
#[tokio::test]
async fn test_locked_money_on_buy() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "LOCKMONEY",
        "Lock Money User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place first order
    place_limit_buy(&state, user_id, &symbol, 10, dollars(100))
        .await
        .unwrap();
    assert_user_locked_money(&state, user_id, dollars(1_000)).await;

    // Place second order
    place_limit_buy(&state, user_id, &symbol, 20, dollars(50))
        .await
        .unwrap();
    assert_user_locked_money(&state, user_id, dollars(2_000)).await; // 1000 + 1000
}

/// TRADE-PLACE-018: Verify locked_qty updated on sell order
#[tokio::test]
async fn test_locked_qty_on_sell() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "LOCKQTY",
        "Lock Qty User",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Place first sell order
    place_limit_sell(&state, user_id, &symbol, 30, dollars(150))
        .await
        .unwrap();
    assert_user_position(&state, user_id, &symbol, 100, 30).await;

    // Place second sell order
    place_limit_sell(&state, user_id, &symbol, 20, dollars(160))
        .await
        .unwrap();
    assert_user_position(&state, user_id, &symbol, 100, 50).await;
}

// =============================================================================
// ORDER CANCELLATION TESTS (TRADE-CANCEL-*)
// =============================================================================

/// TRADE-CANCEL-001: Cancel own open order
#[tokio::test]
async fn test_cancel_own_order() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "CANCEL001",
        "Cancel User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place order
    let order_id = place_limit_buy(&state, user_id, &symbol, 10, dollars(100))
        .await
        .unwrap();
    assert_user_locked_money(&state, user_id, dollars(1_000)).await;

    // Cancel order
    let result = state.engine.cancel_order(user_id, &symbol, order_id).await;
    assert!(result.is_ok(), "Cancel should succeed");

    // Verify funds released
    assert_user_locked_money(&state, user_id, 0).await;
}

/// TRADE-CANCEL-002: Cancel another user's order should fail
#[tokio::test]
async fn test_cancel_other_users_order() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user1 =
        create_test_user_with_portfolio(&state, "OWNER", "Order Owner", dollars(100_000), vec![])
            .await;
    let user2 =
        create_test_user_with_portfolio(&state, "OTHER", "Other User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // User1 places order
    let order_id = place_limit_buy(&state, user1, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // User2 tries to cancel
    let result = state.engine.cancel_order(user2, &symbol, order_id).await;
    assert!(
        result.is_err(),
        "Should not be able to cancel other user's order"
    );
}

/// TRADE-CANCEL-003: Cancel non-existent order should fail
#[tokio::test]
async fn test_cancel_nonexistent_order() {
    let state = create_test_state().await;

    let user_id = create_test_user_with_portfolio(
        &state,
        "CANCELFAKE",
        "Cancel Fake User",
        dollars(100_000),
        vec![],
    )
    .await;

    let result = state.engine.cancel_order(user_id, "FAKE", 999999).await;
    assert!(result.is_err(), "Cancel non-existent order should fail");
}

/// TRADE-CANCEL-005: Verify locked_money released on buy cancel
#[tokio::test]
async fn test_locked_money_released_on_cancel() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let initial_money = dollars(100_000);
    let user_id = create_test_user_with_portfolio(
        &state,
        "RELEASE001",
        "Release User",
        initial_money,
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user_id, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Verify locked
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user.locked_money, dollars(1_000));

    // Cancel
    state
        .engine
        .cancel_order(user_id, &symbol, order_id)
        .await
        .unwrap();

    // Verify released
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user.locked_money, 0);
    assert_eq!(user.money, initial_money); // Money restored
}

/// TRADE-CANCEL-006: Verify locked_qty released on sell cancel
#[tokio::test]
async fn test_locked_qty_released_on_cancel() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "RELEASEQTY",
        "Release Qty User",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_sell(&state, user_id, &symbol, 50, dollars(150))
        .await
        .unwrap();

    // Verify locked
    assert_user_position(&state, user_id, &symbol, 100, 50).await;

    // Cancel
    state
        .engine
        .cancel_order(user_id, &symbol, order_id)
        .await
        .unwrap();

    // Verify released
    assert_user_position(&state, user_id, &symbol, 100, 0).await;
}

// =============================================================================
// ORDER BOOK TESTS (BOOK-*)
// =============================================================================

/// BOOK-PTP-001: Orders at same price matched by time (FIFO)
#[tokio::test]
async fn test_price_time_priority_fifo() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create sellers
    let seller1 = create_test_user_with_portfolio(
        &state,
        "SELLER1",
        "Seller 1",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let seller2 = create_test_user_with_portfolio(
        &state,
        "SELLER2",
        "Seller 2",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    // Create buyer with enough money
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Seller1 places ask first at $100
    place_limit_sell(&state, seller1, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Seller2 places ask second at same price $100
    place_limit_sell(&state, seller2, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Subscribe to trades
    let mut collector = TradeCollector::new(&state);

    // Buyer places order for 50 shares - should match seller1 (first in time)
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    collector.collect();

    // Should have one trade with seller1
    assert_eq!(collector.count(), 1);
    let trade = &collector.trades()[0];
    assert_eq!(trade.qty, 50);
    assert_eq!(
        trade.maker_user_id, seller1,
        "Should match seller1 (first in time)"
    );
}

/// BOOK-PTP-002: Better price matched first (buy side)
#[tokio::test]
async fn test_better_bid_matched_first() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create two buyers with different bid prices
    let buyer1 =
        create_test_user_with_portfolio(&state, "BUYER1", "Buyer 1", dollars(100_000), vec![])
            .await;
    let buyer2 =
        create_test_user_with_portfolio(&state, "BUYER2", "Buyer 2", dollars(100_000), vec![])
            .await;

    // Create seller
    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Buyer1 bids $100
    place_limit_buy(&state, buyer1, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Buyer2 bids $110 (better price, placed second)
    place_limit_buy(&state, buyer2, &symbol, 50, dollars(110))
        .await
        .unwrap();

    let mut collector = TradeCollector::new(&state);

    // Seller sells 50 shares at market (or low price)
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    collector.collect();

    // Should match buyer2 (better price) even though buyer1 was first
    assert_eq!(collector.count(), 1);
    let trade = &collector.trades()[0];
    assert_eq!(trade.taker_user_id, seller);
    assert_eq!(
        trade.maker_user_id, buyer2,
        "Should match buyer2 (better price)"
    );
    assert_eq!(trade.price, dollars(110)); // Trade at maker's price
}

/// BOOK-MATCH-001: Single buy matches single sell
#[tokio::test]
async fn test_simple_match() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    // Seller posts ask at $100
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    let mut collector = TradeCollector::new(&state);

    // Buyer hits the ask
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    collector.collect();

    assert_eq!(collector.count(), 1);
    let trade = &collector.trades()[0];
    assert_eq!(trade.symbol, symbol);
    assert_eq!(trade.qty, 50);
    assert_eq!(trade.price, dollars(100));

    // Verify book is empty
    assert_book_empty(&state, &symbol);
}

/// BOOK-MATCH-002: One buy matches multiple sells
#[tokio::test]
async fn test_one_buy_multiple_sells() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let seller1 = create_test_user_with_portfolio(
        &state,
        "SELLER1",
        "Seller 1",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let seller2 = create_test_user_with_portfolio(
        &state,
        "SELLER2",
        "Seller 2",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Two sellers post asks
    place_limit_sell(&state, seller1, &symbol, 30, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, seller2, &symbol, 30, dollars(101))
        .await
        .unwrap();

    let mut collector = TradeCollector::new(&state);

    // Buyer wants 50 shares, willing to pay up to $101
    place_limit_buy(&state, buyer, &symbol, 50, dollars(101))
        .await
        .unwrap();

    collector.collect();

    // Should match both sellers
    assert_eq!(collector.count(), 2);

    // First trade should be with seller1 at $100 (better price)
    let trade1 = &collector.trades()[0];
    assert_eq!(trade1.maker_user_id, seller1);
    assert_eq!(trade1.qty, 30);
    assert_eq!(trade1.price, dollars(100));

    // Second trade should be with seller2 at $101
    let trade2 = &collector.trades()[1];
    assert_eq!(trade2.maker_user_id, seller2);
    assert_eq!(trade2.qty, 20); // Only 20 more needed
    assert_eq!(trade2.price, dollars(101));
}

/// BOOK-MATCH-004: Limit buy at price < best ask rests in book
#[tokio::test]
async fn test_limit_buy_rests_in_book() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    // Seller posts ask at $100
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    let mut collector = TradeCollector::new(&state);

    // Buyer bids $95 (below ask)
    place_limit_buy(&state, buyer, &symbol, 30, dollars(95))
        .await
        .unwrap();

    collector.collect();

    // No trades should occur
    assert_eq!(collector.count(), 0);

    // Book should have both orders
    if let Some((bids, asks)) = state.engine.get_order_book_depth(&symbol, 10) {
        assert_eq!(bids.len(), 1, "Should have one bid");
        assert_eq!(asks.len(), 1, "Should have one ask");
        assert_eq!(bids[0], (dollars(95), 30));
        assert_eq!(asks[0], (dollars(100), 50));
    }

    // Verify book invariant
    check_book_invariant(&state, &symbol).unwrap();
}

/// BOOK-STATE-001: get_depth() returns correct levels
#[tokio::test]
async fn test_depth_correct_levels() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create multiple users for orders
    let mut buyers = Vec::new();
    let mut sellers = Vec::new();

    for i in 0..5 {
        let buyer = create_test_user_with_portfolio(
            &state,
            &format!("BUYER{}", i),
            &format!("Buyer {}", i),
            dollars(100_000),
            vec![],
        )
        .await;
        buyers.push(buyer);

        let seller = create_test_user_with_portfolio(
            &state,
            &format!("SELLER{}", i),
            &format!("Seller {}", i),
            dollars(10_000),
            vec![("AAPL".to_string(), 100)],
        )
        .await;
        sellers.push(seller);
    }

    open_market(&state);

    // Place bids at different prices (descending)
    for (i, buyer) in buyers.iter().enumerate() {
        let price = dollars(100 - i as i64 * 2); // 100, 98, 96, 94, 92
        place_limit_buy(&state, *buyer, &symbol, 10, price)
            .await
            .unwrap();
    }

    // Place asks at different prices (ascending)
    for (i, seller) in sellers.iter().enumerate() {
        let price = dollars(105 + i as i64 * 2); // 105, 107, 109, 111, 113
        place_limit_sell(&state, *seller, &symbol, 10, price)
            .await
            .unwrap();
    }

    let (bids, asks) = state.engine.get_order_book_depth(&symbol, 10).unwrap();

    // Bids should be sorted descending
    assert_eq!(bids.len(), 5);
    assert_eq!(bids[0].0, dollars(100));
    assert_eq!(bids[4].0, dollars(92));

    // Asks should be sorted ascending
    assert_eq!(asks.len(), 5);
    assert_eq!(asks[0].0, dollars(105));
    assert_eq!(asks[4].0, dollars(113));

    // Verify invariant
    check_book_invariant(&state, &symbol).unwrap();
}

// =============================================================================
// ORDER STATE MACHINE TESTS (SM-ORDER-*)
// =============================================================================

/// SM-ORDER-001: New order starts Open
#[tokio::test]
async fn test_order_starts_open() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id = create_test_user_with_portfolio(
        &state,
        "OPEN001",
        "Open Order User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let order_id = place_limit_buy(&state, user_id, &symbol, 10, dollars(90))
        .await
        .unwrap();

    // Get order from OrdersService
    let orders = state.orders.get_user_orders(user_id);
    assert_eq!(orders.len(), 1);

    let order = &orders[0];
    assert_eq!(order.order_id, order_id);
    assert_eq!(order.filled_qty, 0);
    use stockmart_backend::domain::trading::order::OrderStatus;
    assert!(order.status == OrderStatus::Open || order.status == OrderStatus::Partial);
}

/// SM-ORDER-003: Full fill from Open to Filled
#[tokio::test]
async fn test_order_full_fill() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    // Seller posts ask
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Buyer hits the ask - order fully filled
    let order_id = place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Order should be filled and removed from active orders
    use stockmart_backend::domain::trading::order::OrderStatus;
    let buyer_orders = state.orders.get_user_orders(buyer);
    let is_inactive = |o: &stockmart_backend::domain::ui_models::OpenOrderUI| {
        o.status != OrderStatus::Open && o.status != OrderStatus::Partial
    };
    assert!(
        buyer_orders.is_empty() || buyer_orders.iter().all(is_inactive),
        "Filled order should not be active"
    );
}

/// SM-ORDER-002 & SM-ORDER-004: Partial fill updates status
#[tokio::test]
async fn test_order_partial_fill() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    // Buyer posts bid for 100 shares
    let buyer_order_id = place_limit_buy(&state, buyer, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // Seller sells only 30 shares
    place_limit_sell(&state, seller, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Buyer's order should be partially filled
    use stockmart_backend::domain::trading::order::OrderStatus;
    let buyer_orders = state.orders.get_user_orders(buyer);
    let order = buyer_orders.iter().find(|o| o.order_id == buyer_order_id);

    assert!(order.is_some(), "Order should still exist");
    let order = order.unwrap();
    assert_eq!(order.filled_qty, 30);
    assert_eq!(order.remaining_qty, 70);
    assert!(
        order.status == OrderStatus::Open || order.status == OrderStatus::Partial,
        "Partially filled order should still be active"
    );
}

/// SM-ORDER-005 & SM-ORDER-006: Cancel from Open/Partial
#[tokio::test]
async fn test_order_cancel_states() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "CANCELSTATES",
        "Cancel States User",
        dollars(100_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Test cancel from Open
    use stockmart_backend::domain::trading::order::OrderStatus;
    let order1 = place_limit_buy(&state, user, &symbol, 50, dollars(90))
        .await
        .unwrap();
    state
        .engine
        .cancel_order(user, &symbol, order1)
        .await
        .unwrap();

    let orders = state.orders.get_user_orders(user);
    let is_active = |o: &stockmart_backend::domain::ui_models::OpenOrderUI| {
        o.status == OrderStatus::Open || o.status == OrderStatus::Partial
    };
    assert!(
        orders
            .iter()
            .find(|o| o.order_id == order1 && is_active(o))
            .is_none(),
        "Cancelled order should not be active"
    );

    // Test cancel from Partial (need a counter-party for partial fill)
    let seller = create_test_user_with_portfolio(
        &state,
        "PARTIALSELLER",
        "Partial Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 50)],
    )
    .await;

    let order2 = place_limit_buy(&state, user, &symbol, 100, dollars(95))
        .await
        .unwrap();

    // Partial fill
    place_limit_sell(&state, seller, &symbol, 30, dollars(95))
        .await
        .unwrap();

    // Verify partial state
    let orders = state.orders.get_user_orders(user);
    let partial_order = orders.iter().find(|o| o.order_id == order2);
    assert!(partial_order.is_some());
    assert_eq!(partial_order.unwrap().filled_qty, 30);

    // Cancel partial order
    state
        .engine
        .cancel_order(user, &symbol, order2)
        .await
        .unwrap();

    let orders = state.orders.get_user_orders(user);
    assert!(
        orders
            .iter()
            .find(|o| o.order_id == order2 && is_active(o))
            .is_none(),
        "Cancelled partial order should not be active"
    );
}

// =============================================================================
// SETTLEMENT TESTS (SETTLE-*)
// =============================================================================

/// SETTLE-BUY-001: Buyer receives shares
#[tokio::test]
async fn test_buyer_receives_shares() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Buyer should now have shares
    assert_user_position(&state, buyer, &symbol, 50, 0).await;
}

/// SETTLE-BUY-002: Buyer locked_money released
#[tokio::test]
async fn test_buyer_locked_money_released() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let initial_money = dollars(100_000);
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", initial_money, vec![]).await;

    open_market(&state);

    // Buyer posts bid
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();
    assert_user_locked_money(&state, buyer, dollars(5_000)).await;

    // Seller fills
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Buyer locked_money should be released
    assert_user_locked_money(&state, buyer, 0).await;

    // Buyer money should be reduced by trade value
    let user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(user.money, initial_money - dollars(5_000));
}

/// SETTLE-SELL-001: Seller receives money
#[tokio::test]
async fn test_seller_receives_money() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let initial_seller_money = dollars(10_000);
    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        initial_seller_money,
        vec![("AAPL".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Seller should receive money
    let user = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    assert_eq!(user.money, initial_seller_money + dollars(5_000));
}

/// SETTLE-SELL-002: Seller locked_qty released
#[tokio::test]
async fn test_seller_locked_qty_released() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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

    // Seller posts ask - qty locked
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    assert_user_position(&state, seller, &symbol, 100, 50).await;

    // Buyer fills
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Seller locked_qty released and qty reduced
    assert_user_position(&state, seller, &symbol, 50, 0).await;
}

// =============================================================================
// MARKET STATE TESTS (SM-MKT-*)
// =============================================================================

/// SM-MKT-001: Market starts open (current default behavior)
/// NOTE: The engine defaults to open. This test verifies actual behavior.
#[tokio::test]
async fn test_market_starts_open() {
    let state = create_test_state().await;

    // Current implementation starts with market open
    assert!(
        is_market_open(&state),
        "Market should start open (current default)"
    );
}

/// SM-MKT-002: Open market
#[tokio::test]
async fn test_open_market() {
    let state = create_test_state().await;

    close_market(&state);
    assert!(!is_market_open(&state));

    open_market(&state);
    assert!(is_market_open(&state));
}

/// SM-MKT-003: Close market
#[tokio::test]
async fn test_close_market() {
    let state = create_test_state().await;

    open_market(&state);
    assert!(is_market_open(&state));

    close_market(&state);
    assert!(!is_market_open(&state));
}

// =============================================================================
// FINANCIAL INVARIANT TESTS (INV-FIN-*)
// =============================================================================

/// INV-FIN-001: User money >= 0 always
#[tokio::test]
async fn test_money_never_negative() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

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
        let price = dollars(100 + i);
        place_limit_sell(&state, seller, &symbol, 10, price)
            .await
            .ok();
        place_limit_buy(&state, buyer, &symbol, 10, price)
            .await
            .ok();

        // Check invariant after each trade
        check_money_invariant(&state, buyer).await.unwrap();
        check_money_invariant(&state, seller).await.unwrap();
    }
}

/// INV-FIN-003: locked_qty <= qty for all positions
#[tokio::test]
async fn test_locked_qty_never_exceeds_qty() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user = create_test_user_with_portfolio(
        &state,
        "LOCKINV",
        "Lock Invariant User",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Try to lock all shares
    place_limit_sell(&state, user, &symbol, 100, dollars(150))
        .await
        .unwrap();

    check_position_invariant(&state, user).await.unwrap();

    // Try to sell more (should fail)
    let result = place_limit_sell(&state, user, &symbol, 1, dollars(150)).await;
    assert!(result.is_err(), "Should not be able to oversell");

    // Invariant should still hold
    check_position_invariant(&state, user).await.unwrap();
}

/// INV-BOOK-003: No crossing orders after matching
#[tokio::test]
async fn test_no_crossed_book() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create users
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;
    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // Place orders that should match
    place_limit_buy(&state, buyer, &symbol, 50, dollars(105))
        .await
        .unwrap();
    place_limit_sell(&state, seller, &symbol, 30, dollars(100))
        .await
        .unwrap();

    // Book should not be crossed
    check_book_invariant(&state, &symbol).unwrap();
}
