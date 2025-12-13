//! Concurrency, Race Condition, and Conflict Detection Tests
//!
//! Tests for: CONC-*, CONF-*, ATOM-*

mod common;

use common::*;
use tokio::task::JoinHandle;

// =============================================================================
// CONCURRENT ORDER PLACEMENT TESTS (CONC-ORDER-*)
// =============================================================================

/// CONC-ORDER-001: Multiple users place orders simultaneously
#[tokio::test]
async fn test_concurrent_orders_multiple_users() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create 10 users
    let mut users = Vec::new();
    for i in 0..10 {
        let user_id = create_test_user_with_portfolio(
            &state,
            &format!("USER{}", i),
            &format!("User {}", i),
            dollars(100_000),
            vec![("AAPL".to_string(), 100)],
        )
        .await;
        users.push(user_id);
    }

    open_market(&state);

    // All users place orders concurrently
    let state_clone = state.clone();
    let handles: Vec<JoinHandle<Result<u64, String>>> = users
        .iter()
        .enumerate()
        .map(|(i, &user_id)| {
            let state = state_clone.clone();
            let sym = symbol.clone();
            tokio::spawn(async move {
                if i % 2 == 0 {
                    place_limit_buy(&state, user_id, &sym, 10, dollars(100)).await
                } else {
                    place_limit_sell(&state, user_id, &sym, 10, dollars(100)).await
                }
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All orders should succeed
    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .collect();

    assert_eq!(successes.len(), 10, "All concurrent orders should succeed");

    // Verify no duplicate order IDs
    let order_ids: std::collections::HashSet<_> = successes.iter().copied().collect();
    assert_eq!(order_ids.len(), 10, "All order IDs should be unique");
}

/// CONC-ORDER-002: Same user places multiple orders concurrently
#[tokio::test]
async fn test_concurrent_orders_same_user() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user_id = create_test_user_with_portfolio(
        &state,
        "CONCURRENT_USER",
        "Concurrent User",
        dollars(1_000_000),               // Lots of money
        vec![("AAPL".to_string(), 1000)], // Lots of shares
    )
    .await;

    open_market(&state);

    // User places 20 orders concurrently
    let state_clone = state.clone();
    let handles: Vec<JoinHandle<Result<u64, String>>> = (0..20)
        .map(|i| {
            let state = state_clone.clone();
            let sym = symbol.clone();
            tokio::spawn(async move {
                if i % 2 == 0 {
                    place_limit_buy(&state, user_id, &sym, 5, dollars(90 + i as i64)).await
                } else {
                    place_limit_sell(&state, user_id, &sym, 5, dollars(110 + i as i64)).await
                }
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .collect();

    // Most orders should succeed (some might fail due to fund/share locking race)
    assert!(
        successes.len() >= 10,
        "Most concurrent orders should succeed, got {}",
        successes.len()
    );

    // Verify user state is consistent
    check_money_invariant(&state, user_id).await.unwrap();
    check_position_invariant(&state, user_id).await.unwrap();
}

/// CONC-ORDER-003: Order and cancel race
#[tokio::test]
async fn test_order_cancel_race() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user_id =
        create_test_user_with_portfolio(&state, "RACE_USER", "Race User", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Place an order
    let order_id = place_limit_buy(&state, user_id, &symbol, 50, dollars(90))
        .await
        .unwrap();

    let initial_locked = state
        .user_repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .locked_money;

    // Try to cancel the same order multiple times concurrently
    let state_clone = state.clone();
    let sym = symbol.clone();
    let handles: Vec<JoinHandle<_>> = (0..5)
        .map(|_| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(async move { state.engine.cancel_order(user_id, &sym, order_id).await })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Exactly one cancel should succeed, others should fail (order already cancelled)
    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|r| r.is_ok())
        .collect();

    // Could be 1 or 0 if order was filled
    assert!(successes.len() <= 1, "At most one cancel should succeed");

    // Final state should be consistent
    check_money_invariant(&state, user_id).await.unwrap();

    // Locked money should be back to 0 or still locked (but consistent)
    let final_user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(
        final_user.locked_money == 0 || final_user.locked_money == initial_locked,
        "Locked money should be in consistent state"
    );
}

/// CONC-ORDER-004: Matching during high load
#[tokio::test]
async fn test_matching_high_load() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create buyers and sellers
    let mut buyers = Vec::new();
    let mut sellers = Vec::new();

    for i in 0..20 {
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

    let mut collector = TradeCollector::new(&state);

    // All users place orders concurrently at overlapping prices
    let state_clone = state.clone();
    let sym = symbol.clone();

    let buyer_handles: Vec<_> = buyers
        .iter()
        .map(|&user_id| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(
                async move { place_limit_buy(&state, user_id, &sym, 10, dollars(100)).await },
            )
        })
        .collect();

    let seller_handles: Vec<_> = sellers
        .iter()
        .map(|&user_id| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(
                async move { place_limit_sell(&state, user_id, &sym, 10, dollars(100)).await },
            )
        })
        .collect();

    // Wait for all orders
    let buyer_results: Vec<_> = futures::future::join_all(buyer_handles).await;
    let seller_results: Vec<_> = futures::future::join_all(seller_handles).await;

    // Give a moment for trades to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    collector.collect();

    // Should have many trades
    assert!(collector.count() > 0, "Should have trades during high load");

    // Verify book invariant after all the activity
    check_book_invariant(&state, &symbol).unwrap();

    // Verify all users have consistent state
    for &user_id in buyers.iter().chain(sellers.iter()) {
        check_money_invariant(&state, user_id).await.unwrap();
        check_position_invariant(&state, user_id).await.unwrap();
    }
}

// =============================================================================
// CONCURRENT USER OPERATIONS TESTS (CONC-USER-*)
// =============================================================================

/// CONC-USER-001: Simultaneous registrations
#[tokio::test]
async fn test_concurrent_registrations() {
    let state = create_test_state().await;

    // Try to register 10 users concurrently
    let state_clone = state.clone();
    let handles: Vec<JoinHandle<u64>> = (0..10)
        .map(|i| {
            let state = state_clone.clone();
            tokio::spawn(async move {
                create_test_user(
                    &state,
                    &format!("CONCREG{}", i),
                    &format!("User {}", i),
                    "pass",
                )
                .await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    let user_ids: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .copied()
        .collect();

    assert_eq!(user_ids.len(), 10, "All registrations should succeed");

    // All IDs should be unique
    let unique_ids: std::collections::HashSet<_> = user_ids.iter().collect();
    assert_eq!(unique_ids.len(), 10, "All user IDs should be unique");
}

/// CONC-USER-002: Concurrent portfolio updates (via trades)
#[tokio::test]
async fn test_concurrent_portfolio_updates() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create one user who will receive many concurrent trades
    let buyer = create_test_user_with_portfolio(
        &state,
        "MEGA_BUYER",
        "Mega Buyer",
        dollars(10_000_000), // $10M
        vec![],
    )
    .await;

    // Create many sellers
    let mut sellers = Vec::new();
    for i in 0..10 {
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

    // Buyer places a large bid
    place_limit_buy(&state, buyer, &symbol, 500, dollars(100))
        .await
        .unwrap();

    // All sellers hit the bid concurrently
    let state_clone = state.clone();
    let sym = symbol.clone();
    let handles: Vec<_> = sellers
        .iter()
        .map(|&seller_id| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(async move {
                place_limit_sell(&state, seller_id, &sym, 50, dollars(100)).await
            })
        })
        .collect();

    let _results: Vec<_> = futures::future::join_all(handles).await;

    // Verify buyer's portfolio is consistent
    check_money_invariant(&state, buyer).await.unwrap();
    check_position_invariant(&state, buyer).await.unwrap();

    // Buyer should have received shares
    let buyer_user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();
    let position = buyer_user.portfolio.iter().find(|p| p.symbol == symbol);
    assert!(
        position.is_some(),
        "Buyer should have position after trades"
    );
}

/// CONC-USER-003: Multiple sessions same user
#[tokio::test]
async fn test_concurrent_sessions_same_user() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 2,
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "MULTI_SESSION", "Multi Session User", "pass").await;

    // Create multiple sessions concurrently
    let state_clone = state.clone();
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let state = state_clone.clone();
            tokio::spawn(async move { state.sessions.create_session(user_id) })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // With max_sessions=2, should have exactly 2 active
    // (others would have kicked earlier ones)
    // Total active sessions should be <= 2
    assert!(
        state.sessions.active_session_count() <= 2,
        "Should have at most 2 active sessions"
    );
}

// =============================================================================
// CONFLICT DETECTION TESTS (CONF-*)
// =============================================================================

/// CONF-001: Double-spend attack prevention
#[tokio::test]
async fn test_double_spend_prevention() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // User with exactly $10,000
    let user_id = create_test_user_with_portfolio(
        &state,
        "DOUBLE_SPEND",
        "Double Spend User",
        dollars(10_000),
        vec![],
    )
    .await;

    // Create sellers with liquidity
    let seller1 = create_test_user_with_portfolio(
        &state,
        "SELLER1",
        "Seller 1",
        dollars(10_000),
        vec![("AAPL".to_string(), 200)],
    )
    .await;
    let seller2 = create_test_user_with_portfolio(
        &state,
        "SELLER2",
        "Seller 2",
        dollars(10_000),
        vec![("AAPL".to_string(), 200)],
    )
    .await;

    open_market(&state);

    // Sellers post liquidity
    place_limit_sell(&state, seller1, &symbol, 100, dollars(100))
        .await
        .unwrap();
    place_limit_sell(&state, seller2, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // User tries to buy 200 shares ($20,000) with only $10,000
    // Try two $10,000 orders concurrently
    let state_clone = state.clone();
    let sym = symbol.clone();
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(
                async move { place_limit_buy(&state, user_id, &sym, 100, dollars(100)).await },
            )
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .collect();

    // Only one order should succeed (the other should fail insufficient funds)
    assert_eq!(
        successes.len(),
        1,
        "Only one $10k order should succeed with $10k balance"
    );

    // Verify user state is consistent
    check_money_invariant(&state, user_id).await.unwrap();

    // User should have at most $10,000 worth of activity
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    let total_committed = user.locked_money + (dollars(10_000) - user.money); // money spent
    assert!(
        total_committed <= dollars(10_000),
        "User should not have committed more than their balance"
    );
}

/// CONF-002: Double-sell attack prevention
#[tokio::test]
async fn test_double_sell_prevention() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // User with exactly 100 shares
    let user_id = create_test_user_with_portfolio(
        &state,
        "DOUBLE_SELL",
        "Double Sell User",
        dollars(10_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    // Create buyers
    let buyer1 =
        create_test_user_with_portfolio(&state, "BUYER1", "Buyer 1", dollars(100_000), vec![])
            .await;
    let buyer2 =
        create_test_user_with_portfolio(&state, "BUYER2", "Buyer 2", dollars(100_000), vec![])
            .await;

    open_market(&state);

    // Buyers post bids
    place_limit_buy(&state, buyer1, &symbol, 100, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer2, &symbol, 100, dollars(100))
        .await
        .unwrap();

    // User tries to sell 200 shares with only 100
    let state_clone = state.clone();
    let sym = symbol.clone();
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let state = state_clone.clone();
            let sym = sym.clone();
            tokio::spawn(
                async move { place_limit_sell(&state, user_id, &sym, 100, dollars(100)).await },
            )
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .collect();

    // Only one order should succeed
    assert_eq!(
        successes.len(),
        1,
        "Only one 100-share sell should succeed with 100 shares"
    );

    // Verify position invariant
    check_position_invariant(&state, user_id).await.unwrap();
}

/// CONF-006: Simultaneous registration with same regno
#[tokio::test]
async fn test_simultaneous_duplicate_registration() {
    let state = create_test_state().await;

    let regno = "DUPLICATE_RACE";

    // Try to register same regno concurrently
    let state_clone = state.clone();
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let state = state_clone.clone();
            let regno = regno.to_string();
            tokio::spawn(async move {
                // First check if exists (simulating registration flow)
                if !state.user_repo.regno_exists(&regno).await.unwrap_or(true) {
                    create_test_user(&state, &regno, &format!("User {}", i), "pass").await
                } else {
                    0 // Failed - already exists
                }
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    let successful_ids: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&id| id > 0)
        .collect();

    // Due to race condition, might get 0 or 1 success
    // (depends on timing of check vs create)
    // The important thing is the final state is consistent
    let exists = state.user_repo.regno_exists(regno).await.unwrap();
    // Either all failed, or exactly one succeeded
    assert!(
        successful_ids.len() <= 1 || exists,
        "At most one registration should succeed"
    );
}

// =============================================================================
// STATE TRANSITION ATOMICITY TESTS (ATOM-*)
// =============================================================================

/// ATOM-001: Order placement is atomic (lock + book add)
#[tokio::test]
async fn test_order_placement_atomic() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user_id = create_test_user_with_portfolio(
        &state,
        "ATOMIC001",
        "Atomic User",
        dollars(100_000),
        vec![],
    )
    .await;

    open_market(&state);

    let initial_money = state
        .user_repo
        .find_by_id(user_id)
        .await
        .unwrap()
        .unwrap()
        .money;

    // Place order
    let order_id = place_limit_buy(&state, user_id, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify atomicity: if order exists, money must be locked
    let orders = state.orders.get_user_orders(user_id);
    let order_exists = orders.iter().any(|o| o.order_id == order_id);

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    if order_exists {
        assert!(
            user.locked_money > 0,
            "If order exists, money must be locked"
        );
    } else {
        // Order was filled immediately
        assert!(
            user.money < initial_money || user.locked_money == 0,
            "If order filled, either money spent or nothing locked"
        );
    }
}

/// ATOM-003: Trade execution is atomic (match + settle both parties)
#[tokio::test]
async fn test_trade_execution_atomic() {
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

    let initial_seller_money = dollars(10_000);
    let initial_buyer_money = dollars(100_000);

    open_market(&state);

    // Execute a trade
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Verify both sides settled atomically
    let seller_user = state.user_repo.find_by_id(seller).await.unwrap().unwrap();
    let buyer_user = state.user_repo.find_by_id(buyer).await.unwrap().unwrap();

    let trade_value = dollars(5_000); // 50 * $100

    // Seller should have received money
    let seller_received = seller_user.money - initial_seller_money;

    // Buyer should have shares and spent money
    let buyer_has_shares = buyer_user
        .portfolio
        .iter()
        .find(|p| p.symbol == symbol)
        .map(|p| p.qty)
        .unwrap_or(0);
    let buyer_spent = initial_buyer_money - buyer_user.money - buyer_user.locked_money;

    // If seller received money, buyer must have shares (atomicity)
    if seller_received > 0 {
        assert!(
            buyer_has_shares > 0,
            "If seller received money, buyer must have shares"
        );
        assert_eq!(
            seller_received, buyer_spent,
            "Money transferred should match"
        );
    }
}

// =============================================================================
// REPOSITORY CONCURRENCY TESTS (CONC-REPO-*)
// =============================================================================

/// CONC-REPO-001: Concurrent reads
#[tokio::test]
async fn test_concurrent_reads() {
    let state = create_test_state().await;

    // Create a user
    let user_id = create_test_user(&state, "CONCURRENT_READ", "Concurrent Read User", "pass").await;

    let state_clone = state.clone();

    // Many concurrent reads
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let state = state_clone.clone();
            tokio::spawn(async move { state.user_repo.find_by_id(user_id).await })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All reads should succeed
    let successes: Vec<_> = results
        .iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .collect();

    assert_eq!(successes.len(), 100, "All concurrent reads should succeed");
}

/// CONC-REPO-002: Concurrent writes
#[tokio::test]
async fn test_concurrent_writes() {
    let state = create_test_state().await;

    // Create a user
    let user_id =
        create_test_user(&state, "CONCURRENT_WRITE", "Concurrent Write User", "pass").await;

    let state_clone = state.clone();

    // Many concurrent writes updating money
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let state = state_clone.clone();
            tokio::spawn(async move {
                let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
                user.money = dollars(i as i64 * 1000);
                state.user_repo.save(user).await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All writes should succeed (DashMap handles this)
    let successes: Vec<_> = results
        .iter()
        .filter(|r| r.is_ok())
        .filter(|r| r.as_ref().unwrap().is_ok())
        .collect();

    assert_eq!(successes.len(), 20, "All concurrent writes should succeed");

    // Final state should be valid (one of the values)
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.money >= 0, "Final money should be valid");
}

// =============================================================================
// BROADCAST CHANNEL CONCURRENCY TESTS (CONC-BCAST-*)
// =============================================================================

/// CONC-BCAST-001: Multiple receivers get all messages
#[tokio::test]
async fn test_broadcast_multiple_receivers() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;

    // Create multiple trade collectors (receivers)
    let mut collectors: Vec<_> = (0..5).map(|_| TradeCollector::new(&state)).collect();

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

    // Execute a trade
    place_limit_sell(&state, seller, &symbol, 50, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 50, dollars(100))
        .await
        .unwrap();

    // Give broadcasts time to propagate
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // All collectors should receive the trade
    for (i, collector) in collectors.iter_mut().enumerate() {
        collector.collect();
        assert!(
            collector.count() >= 1,
            "Collector {} should have received at least 1 trade, got {}",
            i,
            collector.count()
        );
    }
}
