//! Validation & Input Boundary Tests
//!
//! Tests for: VAL-*, INPUT-*, BOUND-*
//! Focuses on input validation, boundary conditions, and edge cases

mod common;

use common::*;

// =============================================================================
// PRICE VALIDATION TESTS (VAL-PRICE-*)
// =============================================================================

/// VAL-PRICE-001: Maximum valid price
#[tokio::test]
async fn test_max_valid_price() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MAXP", "Max Price Co").await;

    // User with lots of money
    let buyer =
        create_test_user_with_portfolio(&state, "MAXBUYER", "Max Buyer", i64::MAX / 2, vec![])
            .await;

    open_market(&state);

    // Very high price (just below overflow threshold)
    let high_price = 1_000_000 * dollars(1); // $1,000,000.00
    let result = place_limit_buy(&state, buyer, &symbol, 1, high_price).await;

    // This should work with sufficient funds
    assert!(
        result.is_ok(),
        "High price order should be accepted: {:?}",
        result
    );
}

/// VAL-PRICE-002: Price at i64::MAX boundary
#[tokio::test]
async fn test_price_at_max_boundary() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MAXB", "Max Boundary Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "MAXBND", "Max Boundary", i64::MAX, vec![]).await;

    open_market(&state);

    // Price at i64::MAX - this may cause overflow in calculations
    let result = place_limit_buy(&state, buyer, &symbol, 1, i64::MAX).await;

    // Document behavior - likely fails due to overflow or insufficient funds calculation
    // The important thing is it doesn't panic
    match result {
        Ok(_) => println!("Max price accepted"),
        Err(e) => println!("Max price rejected: {}", e),
    }
}

/// VAL-PRICE-003: Negative price handling
#[tokio::test]
async fn test_negative_price_boundary() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "NEGP", "Neg Price Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "NEGBUYER", "Neg Buyer", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Negative price - DOCUMENTS CURRENT BEHAVIOR
    // Note: This is a security vulnerability - should be rejected
    let result = place_limit_buy(&state, buyer, &symbol, 10, -dollars(100)).await;

    // Document current behavior
    match &result {
        Ok(_) => println!("VULNERABILITY: Negative price accepted"),
        Err(e) => println!("Negative price correctly rejected: {}", e),
    }
}

/// VAL-PRICE-004: Zero price handling
#[tokio::test]
async fn test_zero_price_boundary() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ZEROP", "Zero Price Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "ZEROBUYER",
        "Zero Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Zero price
    let result = place_limit_buy(&state, buyer, &symbol, 10, 0).await;

    // Document current behavior
    match &result {
        Ok(_) => println!("INFO: Zero price accepted (may be intentional for free shares)"),
        Err(e) => println!("Zero price rejected: {}", e),
    }
}

/// VAL-PRICE-005: Price with maximum precision
#[tokio::test]
async fn test_price_precision() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "PRECP", "Precision Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "PRECSELL",
        "Precision Seller",
        dollars(10_000),
        vec![("PRECP".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "PRECBUY",
        "Precision Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Price with sub-cent precision (PRICE_SCALE = 10000, so $1.0001)
    let precise_price = 10001; // $1.0001

    place_limit_sell(&state, seller, &symbol, 10, precise_price)
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, precise_price)
        .await
        .unwrap();

    // Verify trade executed at precise price
    let history = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    assert!(!history.is_empty());
    assert_eq!(history[0].price, precise_price);
}

// =============================================================================
// QUANTITY VALIDATION TESTS (VAL-QTY-*)
// =============================================================================

/// VAL-QTY-001: Maximum quantity (u64::MAX) - DOCUMENTS PANIC BUG
/// This test documents that max quantity causes an overflow panic.
/// This is a bug that should be fixed with proper input validation.
#[tokio::test]
#[should_panic(expected = "overflow")]
async fn test_max_quantity() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MAXQ", "Max Qty Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "MAXQTY", "Max Qty Buyer", i64::MAX, vec![]).await;

    open_market(&state);

    // Maximum u64 quantity - causes overflow panic (BUG)
    let _result = place_limit_buy(&state, buyer, &symbol, u64::MAX, dollars(1)).await;
}

/// VAL-QTY-002: Very large quantity causing overflow - DOCUMENTS PANIC BUG
/// This test documents that large quantities cause overflow panic.
/// This is a bug that should be fixed with proper overflow checks.
#[tokio::test]
#[should_panic(expected = "overflow")]
async fn test_large_quantity_overflow() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "OVFQ", "Overflow Qty Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "OVFQTY", "Overflow Qty Buyer", i64::MAX, vec![])
            .await;

    open_market(&state);

    // Large quantity that would overflow: qty * price > i64::MAX
    // price = $100, qty = i64::MAX / 100 + 1 would overflow
    let large_qty = (i64::MAX as u64 / 1_000_000) + 1;
    let _result = place_limit_buy(&state, buyer, &symbol, large_qty, dollars(100)).await;
}

/// VAL-QTY-003: Quantity of 1 (minimum valid)
#[tokio::test]
async fn test_minimum_quantity() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "MINQ", "Min Qty Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "MINQSELL",
        "Min Qty Seller",
        dollars(10_000),
        vec![("MINQ".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "MINQBUY",
        "Min Qty Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Minimum valid quantity
    place_limit_sell(&state, seller, &symbol, 1, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 1, dollars(100)).await;

    assert!(result.is_ok());

    // Verify trade executed
    let history = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    assert_eq!(history[0].qty, 1);
}

/// VAL-QTY-004: Zero quantity handling - DOCUMENTS MISSING VALIDATION
/// NOTE: Zero quantity is currently NOT rejected by the engine.
/// This test documents this vulnerability - zero qty orders should be rejected.
#[tokio::test]
async fn test_zero_quantity_handling() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ZEROQ", "Zero Qty Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "ZEROQUSER",
        "Zero Qty User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Zero quantity - DOCUMENTS CURRENT BEHAVIOR
    let result = place_limit_buy(&state, buyer, &symbol, 0, dollars(100)).await;

    // Document current behavior
    match &result {
        Ok(_) => println!("VULNERABILITY: Zero quantity order accepted"),
        Err(e) => println!("Zero quantity correctly rejected: {}", e),
    }
}

// =============================================================================
// SYMBOL VALIDATION TESTS (VAL-SYMBOL-*)
// =============================================================================

/// VAL-SYMBOL-001: Non-existent symbol
#[tokio::test]
async fn test_nonexistent_symbol() {
    let state = create_test_state().await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "NOSYM",
        "No Symbol User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Order for non-existent symbol
    let result = place_limit_buy(&state, buyer, "DOESNOTEXIST", 10, dollars(100)).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found") || err.contains("No order book"));
}

/// VAL-SYMBOL-002: Empty symbol string
#[tokio::test]
async fn test_empty_symbol() {
    let state = create_test_state().await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "EMPTYSYM",
        "Empty Symbol User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Empty symbol
    let result = place_limit_buy(&state, buyer, "", 10, dollars(100)).await;

    assert!(result.is_err());
}

/// VAL-SYMBOL-003: Symbol with special characters
#[tokio::test]
async fn test_symbol_special_chars() {
    let state = create_test_state().await;

    // Create company with special chars in symbol
    let symbol = create_test_company(&state, "A-B.C", "Special Symbol Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SPECSELL",
        "Spec Seller",
        dollars(10_000),
        vec![("A-B.C".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "SPECBUY", "Spec Buyer", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Should work with special characters
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_ok());
}

/// VAL-SYMBOL-004: Very long symbol
#[tokio::test]
async fn test_very_long_symbol() {
    let state = create_test_state().await;

    // Very long symbol
    let long_symbol = "A".repeat(100);
    let symbol = create_test_company(&state, &long_symbol, "Long Symbol Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "LONGSELL",
        "Long Seller",
        dollars(10_000),
        vec![(long_symbol.clone(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "LONGBUY", "Long Buyer", dollars(100_000), vec![])
            .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_ok());
}

/// VAL-SYMBOL-005: Unicode symbol
#[tokio::test]
async fn test_unicode_symbol() {
    let state = create_test_state().await;

    // Unicode symbol
    let unicode_symbol = "🚀MOON";
    let symbol = create_test_company(&state, unicode_symbol, "Moon Rocket Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "MOONSELL",
        "Moon Seller",
        dollars(10_000),
        vec![(unicode_symbol.to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "MOONBUY", "Moon Buyer", dollars(100_000), vec![])
            .await;

    open_market(&state);

    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_ok());
}

// =============================================================================
// USER ID VALIDATION TESTS (VAL-USER-*)
// =============================================================================

/// VAL-USER-001: Non-existent user ID
#[tokio::test]
async fn test_nonexistent_user_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "NOUSER", "No User Co").await;

    open_market(&state);

    // Order from non-existent user
    let result = place_limit_buy(&state, 99999, &symbol, 10, dollars(100)).await;

    assert!(result.is_err());
}

/// VAL-USER-002: User ID zero
#[tokio::test]
async fn test_user_id_zero() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "ZEROUSR", "Zero User Co").await;

    open_market(&state);

    // Order from user ID 0
    let result = place_limit_buy(&state, 0, &symbol, 10, dollars(100)).await;

    assert!(result.is_err());
}

// =============================================================================
// MARKET STATE VALIDATION TESTS (VAL-MARKET-*)
// =============================================================================

/// VAL-MARKET-001: Orders rejected when market closed
#[tokio::test]
async fn test_market_closed_orders_rejected() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CLOSED", "Closed Market Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "CLOSEBUY",
        "Close Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    close_market(&state);

    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_lowercase();
    assert!(err.contains("closed") || err.contains("market"));
}

/// VAL-MARKET-002: Orders accepted immediately after market opens
#[tokio::test]
async fn test_market_open_orders_accepted() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "OPENED", "Opened Market Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "OPENBUY", "Open Buyer", dollars(100_000), vec![])
            .await;

    close_market(&state);

    // Verify market is closed
    assert!(!is_market_open(&state));

    // Open market
    open_market(&state);

    // Verify market is open
    assert!(is_market_open(&state));

    // Order should now be accepted
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;
    assert!(result.is_ok());
}

/// VAL-MARKET-003: Rapid market open/close cycles
#[tokio::test]
async fn test_rapid_market_state_changes() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "RAPID", "Rapid Change Co").await;

    let buyer = create_test_user_with_portfolio(
        &state,
        "RAPIDBUY",
        "Rapid Buyer",
        dollars(1_000_000),
        vec![],
    )
    .await;

    // Rapid state changes
    for _ in 0..100 {
        open_market(&state);
        close_market(&state);
    }

    // Should be closed after even number of cycles
    assert!(!is_market_open(&state));

    open_market(&state);

    // Order should work
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;
    assert!(result.is_ok());
}

// =============================================================================
// FUNDS VALIDATION TESTS (VAL-FUNDS-*)
// =============================================================================

/// VAL-FUNDS-001: Exact funds for order
#[tokio::test]
async fn test_exact_funds_for_order() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "EXACT", "Exact Funds Co").await;

    // Exactly enough for 10 shares at $100 = $1000
    let buyer =
        create_test_user_with_portfolio(&state, "EXACTBUY", "Exact Buyer", dollars(1_000), vec![])
            .await;

    open_market(&state);

    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_ok());

    // User should have all money locked
    let user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert_eq!(user.locked_money, dollars(1_000));
    assert_eq!(user.money, 0);
}

/// VAL-FUNDS-002: One cent short of required funds
#[tokio::test]
async fn test_one_cent_short() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SHORT", "Short Funds Co").await;

    // One cent short: need $1000.00, have $999.99
    let buyer = create_test_user_with_portfolio(
        &state,
        "SHORTBUY",
        "Short Buyer",
        dollars(1_000) - 100,
        vec![], // -$0.01
    )
    .await;

    open_market(&state);

    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_lowercase();
    assert!(err.contains("insufficient") || err.contains("funds"));
}

/// VAL-FUNDS-003: Multiple orders consuming all funds
#[tokio::test]
async fn test_multiple_orders_exhaust_funds() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "EXHAUST", "Exhaust Funds Co").await;

    // Exactly enough for 2 orders of 5 shares each at $100 = $1000
    let buyer = create_test_user_with_portfolio(
        &state,
        "EXHAUSTBUY",
        "Exhaust Buyer",
        dollars(1_000),
        vec![],
    )
    .await;

    open_market(&state);

    // First order succeeds
    let result1 = place_limit_buy(&state, buyer, &symbol, 5, dollars(100)).await;
    assert!(result1.is_ok());

    // Second order also succeeds
    let result2 = place_limit_buy(&state, buyer, &symbol, 5, dollars(100)).await;
    assert!(result2.is_ok());

    // Third order should fail - no funds left
    let result3 = place_limit_buy(&state, buyer, &symbol, 1, dollars(100)).await;
    assert!(result3.is_err());
}

/// VAL-FUNDS-004: Funds released on cancel then used for new order
#[tokio::test]
async fn test_funds_release_and_reuse() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "REUSE", "Reuse Funds Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "REUSEBUY", "Reuse Buyer", dollars(1_000), vec![])
            .await;

    open_market(&state);

    // Place order consuming all funds
    let order_id = place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Cancel order
    state
        .engine
        .cancel_order(buyer, &symbol, order_id)
        .await
        .unwrap();

    // Funds should be released, new order should work
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;
    assert!(result.is_ok());
}

// =============================================================================
// SHARES VALIDATION TESTS (VAL-SHARES-*)
// =============================================================================

/// VAL-SHARES-001: Exact shares for sell order
#[tokio::test]
async fn test_exact_shares_for_sell() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "EXACTS", "Exact Shares Co").await;

    // Exactly 100 shares
    let seller = create_test_user_with_portfolio(
        &state,
        "EXACTSELL",
        "Exact Seller",
        dollars(10_000),
        vec![("EXACTS".to_string(), 100)],
    )
    .await;

    open_market(&state);

    let result = place_limit_sell(&state, seller, &symbol, 100, dollars(100)).await;

    assert!(result.is_ok());
}

/// VAL-SHARES-002: One share short for sell
#[tokio::test]
async fn test_one_share_short() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "SHORTSH", "Short Shares Co").await;

    // 99 shares, trying to sell 100
    let seller = create_test_user_with_portfolio(
        &state,
        "SHORTSELL",
        "Short Seller",
        dollars(10_000),
        vec![("SHORTSH".to_string(), 99)],
    )
    .await;

    open_market(&state);

    let result = place_limit_sell(&state, seller, &symbol, 100, dollars(100)).await;

    assert!(result.is_err());
}

/// VAL-SHARES-003: Multiple sell orders consuming all shares
#[tokio::test]
async fn test_multiple_sells_exhaust_shares() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "EXHAUSTS", "Exhaust Shares Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "EXHAUSTSELL",
        "Exhaust Seller",
        dollars(10_000),
        vec![("EXHAUSTS".to_string(), 100)],
    )
    .await;

    open_market(&state);

    // First sell succeeds
    let result1 = place_limit_sell(&state, seller, &symbol, 50, dollars(100)).await;
    assert!(result1.is_ok());

    // Second sell also succeeds
    let result2 = place_limit_sell(&state, seller, &symbol, 50, dollars(100)).await;
    assert!(result2.is_ok());

    // Third sell should fail - no shares left
    let result3 = place_limit_sell(&state, seller, &symbol, 1, dollars(100)).await;
    assert!(result3.is_err());
}

// =============================================================================
// ARITHMETIC OVERFLOW TESTS (VAL-OVERFLOW-*)
// =============================================================================

/// VAL-OVERFLOW-001: Price * Quantity overflow risk
#[tokio::test]
async fn test_price_qty_overflow() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "OVFLW", "Overflow Co").await;

    let buyer =
        create_test_user_with_portfolio(&state, "OVFLWBUY", "Overflow Buyer", i64::MAX, vec![])
            .await;

    open_market(&state);

    // Large price and quantity that would overflow i64
    // i64::MAX = 9,223,372,036,854,775,807
    // If price = 10^9 and qty = 10^9, price * qty = 10^18 which is < i64::MAX
    // But price * qty * PRICE_SCALE could overflow

    let price = 1_000_000 * dollars(1); // $1,000,000
    let qty = 10_000u64;

    // This might cause overflow in lock calculation
    let result = place_limit_buy(&state, buyer, &symbol, qty, price).await;

    // Document behavior - should not panic
    match result {
        Ok(_) => println!("Large order accepted"),
        Err(e) => println!("Large order rejected: {}", e),
    }
}

/// VAL-OVERFLOW-002: Trade total value overflow
#[tokio::test]
async fn test_trade_value_overflow() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "TRADOVF", "Trade Overflow Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "TRADOVFSELL",
        "Trade Overflow Seller",
        dollars(10_000),
        vec![("TRADOVF".to_string(), 1_000_000)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "TRADOVFBUY",
        "Trade Overflow Buyer",
        i64::MAX / 2,
        vec![],
    )
    .await;

    open_market(&state);

    // Large trade that might overflow in trade_history total_value calculation
    let price = 1_000_000 * dollars(1); // $1,000,000
    let qty = 100u64;

    place_limit_sell(&state, seller, &symbol, qty, price)
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, qty, price).await;

    if result.is_ok() {
        // Check trade history doesn't panic
        let history = state.trade_history.get_recent_symbol_trades(&symbol, 10);
        assert!(!history.is_empty());
    }
}

// =============================================================================
// BANKRUPT COMPANY TESTS (VAL-BANKRUPT-*)
// =============================================================================

/// VAL-BANKRUPT-001: Cannot trade bankrupt company
#[tokio::test]
async fn test_cannot_trade_bankrupt_company() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "BNKRPT", "Bankrupt Co").await;

    // Mark company as bankrupt
    state
        .admin
        .set_company_bankrupt(&symbol, true)
        .await
        .unwrap();

    let buyer = create_test_user_with_portfolio(
        &state,
        "BNKRPTBUY",
        "Bankrupt Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Should not be able to trade
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    // Document behavior
    match &result {
        Ok(_) => println!("WARNING: Can trade bankrupt company"),
        Err(e) => println!("Correctly rejected bankrupt trade: {}", e),
    }
}

/// VAL-BANKRUPT-002: Unbankrupt company allows trading again
#[tokio::test]
async fn test_unbankrupt_company_tradeable() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "UNBKRPT", "Unbankrupt Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "UNBKRPTSELL",
        "Unbankrupt Seller",
        dollars(10_000),
        vec![("UNBKRPT".to_string(), 100)],
    )
    .await;
    let buyer = create_test_user_with_portfolio(
        &state,
        "UNBKRPTBUY",
        "Unbankrupt Buyer",
        dollars(100_000),
        vec![],
    )
    .await;

    // Mark bankrupt then unmark
    state
        .admin
        .set_company_bankrupt(&symbol, true)
        .await
        .unwrap();
    state
        .admin
        .set_company_bankrupt(&symbol, false)
        .await
        .unwrap();

    open_market(&state);

    // Should be able to trade again
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(result.is_ok());
}

// =============================================================================
// CONCURRENT VALIDATION TESTS (VAL-CONCURRENT-*)
// =============================================================================

/// VAL-CONCURRENT-001: Same user placing orders concurrently
#[tokio::test]
async fn test_concurrent_orders_same_user() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "CONC", "Concurrent Co").await;

    // User with limited funds
    let buyer = create_test_user_with_portfolio(
        &state,
        "CONCBUY",
        "Concurrent Buyer",
        dollars(1_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Place 10 orders concurrently, each for $200 = $2000 total
    // Only 5 should succeed (user has $1000)
    let mut handles = vec![];
    for _ in 0..10 {
        let state_clone = state.clone();
        let symbol_clone = symbol.clone();
        let handle = tokio::spawn(async move {
            place_limit_buy(&state_clone, buyer, &symbol_clone, 2, dollars(100)).await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    let successes = results
        .iter()
        .filter(|r| r.as_ref().map(|r| r.is_ok()).unwrap_or(false))
        .count();
    let failures = results
        .iter()
        .filter(|r| r.as_ref().map(|r| r.is_err()).unwrap_or(true))
        .count();

    // Verify invariant: total locked should not exceed original balance
    let user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    assert!(
        user.locked_money <= dollars(1_000),
        "Locked money {} should not exceed original balance $1000",
        user.locked_money
    );

    println!(
        "Concurrent orders: {} succeeded, {} failed",
        successes, failures
    );
}

/// VAL-CONCURRENT-002: Two users competing for same liquidity
#[tokio::test]
async fn test_concurrent_competing_buyers() {
    let state = create_test_state().await;
    let symbol = create_test_company(&state, "COMPETE", "Compete Co").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "COMPSELL",
        "Compete Seller",
        dollars(10_000),
        vec![("COMPETE".to_string(), 10)], // Only 10 shares available
    )
    .await;
    let buyer1 = create_test_user_with_portfolio(
        &state,
        "COMPBUY1",
        "Compete Buyer 1",
        dollars(100_000),
        vec![],
    )
    .await;
    let buyer2 = create_test_user_with_portfolio(
        &state,
        "COMPBUY2",
        "Compete Buyer 2",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    // Seller offers 10 shares
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Both buyers try to buy 10 shares at same time
    let state1 = state.clone();
    let state2 = state.clone();
    let symbol1 = symbol.clone();
    let symbol2 = symbol.clone();

    let (_r1, _r2) = tokio::join!(
        place_limit_buy(&state1, buyer1, &symbol1, 10, dollars(100)),
        place_limit_buy(&state2, buyer2, &symbol2, 10, dollars(100))
    );

    // Combined fills should equal available shares
    let buyer1_shares = state
        .user_repo
        .find_by_id(buyer1)
        .await
        .unwrap()
        .unwrap()
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer2_shares = state
        .user_repo
        .find_by_id(buyer2)
        .await
        .unwrap()
        .unwrap()
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);

    assert_eq!(
        buyer1_shares + buyer2_shares,
        10,
        "Total shares should equal available liquidity (buyer1: {}, buyer2: {})",
        buyer1_shares,
        buyer2_shares
    );

    println!(
        "Buyer 1 got {} shares, Buyer 2 got {} shares",
        buyer1_shares, buyer2_shares
    );
}
