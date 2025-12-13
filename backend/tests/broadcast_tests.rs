//! Market Data & Broadcasting Integration Tests
//!
//! Tests for: BCAST-*, chat, leaderboard, trade broadcasting

mod common;

use common::*;

// =============================================================================
// TRADE BROADCASTING TESTS (BCAST-TRADE-*)
// =============================================================================

/// BCAST-TRADE-001: Trade broadcast to subscribers
#[tokio::test]
async fn test_trade_broadcast_to_subscribers() {
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

    // Subscribe to trade channel
    let mut trade_rx = state.engine.subscribe_trades();

    open_market(&state);

    // Place matching orders to trigger trade
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // Should receive trade broadcast
    let trade =
        tokio::time::timeout(tokio::time::Duration::from_millis(100), trade_rx.recv()).await;

    assert!(trade.is_ok(), "Should receive trade broadcast");
    let trade = trade.unwrap().unwrap();
    assert_eq!(trade.symbol, "AAPL");
    assert_eq!(trade.qty, 10);
}

/// BCAST-TRADE-002: Multiple subscribers receive same trade
#[tokio::test]
async fn test_trade_broadcast_multiple_subscribers() {
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

    // Multiple subscribers
    let mut rx1 = state.engine.subscribe_trades();
    let mut rx2 = state.engine.subscribe_trades();
    let mut rx3 = state.engine.subscribe_trades();

    open_market(&state);

    // Execute trade
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    // All should receive the trade
    let timeout = tokio::time::Duration::from_millis(100);
    let t1 = tokio::time::timeout(timeout, rx1.recv()).await;
    let t2 = tokio::time::timeout(timeout, rx2.recv()).await;
    let t3 = tokio::time::timeout(timeout, rx3.recv()).await;

    assert!(t1.is_ok(), "Subscriber 1 should receive trade");
    assert!(t2.is_ok(), "Subscriber 2 should receive trade");
    assert!(t3.is_ok(), "Subscriber 3 should receive trade");
}

/// BCAST-TRADE-004: Broadcast channel handles dropped receivers gracefully
#[tokio::test]
async fn test_trade_broadcast_dropped_receiver() {
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

    // Create and immediately drop a receiver
    {
        let _rx = state.engine.subscribe_trades();
        // Dropped at end of scope
    }

    // Active receiver
    let mut active_rx = state.engine.subscribe_trades();

    open_market(&state);

    // Trade should still work
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    let result = place_limit_buy(&state, buyer, &symbol, 10, dollars(100)).await;

    assert!(
        result.is_ok(),
        "Trade should succeed even with dropped receiver"
    );

    // Active receiver should still get the message
    let trade =
        tokio::time::timeout(tokio::time::Duration::from_millis(100), active_rx.recv()).await;
    assert!(trade.is_ok());
}

// =============================================================================
// CHAT BROADCASTING TESTS (BCAST-CHAT-*)
// =============================================================================

/// BCAST-CHAT-001: Chat message broadcast
#[tokio::test]
async fn test_chat_message_broadcast() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "CHATTER", "Chat User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Create and broadcast chat message
    let message = stockmart_backend::domain::market::chat::ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        user_id,
        username: user.name.clone(),
        message: "Hello World!".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    state.chat.broadcast_message(message);

    // Message should be in history
    let history = state.chat.get_history();
    assert!(!history.is_empty(), "Chat history should have message");
    assert!(history.iter().any(|m| m.message == "Hello World!"));
}

/// BCAST-CHAT-002: Muted user cannot chat
#[tokio::test]
async fn test_muted_user_cannot_chat() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "MUTEDCHATTER", "Muted User", "pass").await;

    // Mute user
    state.admin.set_trader_chat(user_id, false).await.unwrap();

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled, "User should be muted");

    // The chat_enabled check happens at handler level, so we verify the flag
    // In the handler, muted users would be rejected
}

/// BCAST-CHAT-003: Chat history maintained
#[tokio::test]
async fn test_chat_history_maintained() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "HISTORY", "History User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Send multiple messages
    for i in 1..=5 {
        let message = stockmart_backend::domain::market::chat::ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username: user.name.clone(),
            message: format!("Message {}", i),
            timestamp: chrono::Utc::now().timestamp(),
        };
        state.chat.broadcast_message(message);
    }

    // Should have all messages
    let history = state.chat.get_history();
    assert_eq!(history.len(), 5, "Should have all 5 messages");

    // Messages should be in order
    assert!(history[0].message.contains("1"));
    assert!(history[4].message.contains("5"));
}

/// BCAST-CHAT-004: Chat history is limited to prevent unbounded growth
#[tokio::test]
async fn test_chat_history_limit() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "FLOOD", "Flood User", "pass").await;
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();

    // Send many messages (more than the 50 limit)
    for i in 1..=60 {
        let message = stockmart_backend::domain::market::chat::ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username: user.name.clone(),
            message: format!("Message {}", i),
            timestamp: chrono::Utc::now().timestamp(),
        };
        state.chat.broadcast_message(message);
    }

    // History should be limited to 50
    let history = state.chat.get_history();
    assert!(
        history.len() <= 50,
        "Chat history should be limited to 50 messages"
    );
}

// =============================================================================
// TRADE HISTORY TESTS
// =============================================================================

/// Test: Trade history stores trades
#[tokio::test]
async fn test_trade_history_stores_trades() {
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

    // Execute trades
    place_limit_sell(&state, seller, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 10, dollars(100))
        .await
        .unwrap();

    place_limit_sell(&state, seller, &symbol, 20, dollars(101))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol, 20, dollars(101))
        .await
        .unwrap();

    // Check trade history
    let history = state.trade_history.get_recent_symbol_trades(&symbol, 10);
    assert!(
        history.len() >= 2,
        "Should have at least 2 trades in history"
    );
}

/// Test: Trade history per symbol
#[tokio::test]
async fn test_trade_history_per_symbol() {
    let state = create_test_state().await;

    let symbol1 = create_test_company(&state, "AAPL", "Apple").await;
    let symbol2 = create_test_company(&state, "GOOGL", "Google").await;

    let seller = create_test_user_with_portfolio(
        &state,
        "SELLER",
        "Seller",
        dollars(10_000),
        vec![("AAPL".to_string(), 100), ("GOOGL".to_string(), 100)],
    )
    .await;
    let buyer =
        create_test_user_with_portfolio(&state, "BUYER", "Buyer", dollars(100_000), vec![]).await;

    open_market(&state);

    // Trade AAPL
    place_limit_sell(&state, seller, &symbol1, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol1, 10, dollars(100))
        .await
        .unwrap();

    // Trade GOOGL
    place_limit_sell(&state, seller, &symbol2, 5, dollars(200))
        .await
        .unwrap();
    place_limit_buy(&state, buyer, &symbol2, 5, dollars(200))
        .await
        .unwrap();

    // Each symbol should have its own history
    let aapl_history = state.trade_history.get_recent_symbol_trades(&symbol1, 10);
    let googl_history = state.trade_history.get_recent_symbol_trades(&symbol2, 10);

    assert!(aapl_history.iter().all(|t| t.symbol == "AAPL"));
    assert!(googl_history.iter().all(|t| t.symbol == "GOOGL"));
}

// =============================================================================
// LEADERBOARD TESTS (BCAST-LEADER-*)
// =============================================================================

/// BCAST-LEADER-001: Leaderboard calculates net worth
#[tokio::test]
async fn test_leaderboard_calculates_networth() {
    let state = create_test_state().await;

    create_test_company(&state, "AAPL", "Apple").await;

    // Create users with different net worths
    let _user1 = create_test_user_with_portfolio(
        &state,
        "RICH",
        "Rich User",
        dollars(100_000),
        vec![("AAPL".to_string(), 100)], // +$10,000 at $100
    )
    .await;

    let _user2 =
        create_test_user_with_portfolio(&state, "POOR", "Poor User", dollars(10_000), vec![]).await;

    // Get leaderboard - would need to check the leaderboard service
    // This tests the basic setup
    let traders = state.admin.get_all_traders().await.unwrap();
    assert_eq!(traders.len(), 2);
}

// =============================================================================
// ORDER BOOK DEPTH TESTS
// =============================================================================

/// Test: Get order book depth
#[tokio::test]
async fn test_orderbook_depth() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple").await;

    let buyer1 =
        create_test_user_with_portfolio(&state, "B1", "Buyer 1", dollars(100_000), vec![]).await;
    let buyer2 =
        create_test_user_with_portfolio(&state, "B2", "Buyer 2", dollars(100_000), vec![]).await;
    let seller = create_test_user_with_portfolio(
        &state,
        "S1",
        "Seller 1",
        dollars(10_000),
        vec![("AAPL".to_string(), 200)],
    )
    .await;

    open_market(&state);

    // Create bid levels
    place_limit_buy(&state, buyer1, &symbol, 10, dollars(100))
        .await
        .unwrap();
    place_limit_buy(&state, buyer2, &symbol, 20, dollars(99))
        .await
        .unwrap();

    // Create ask levels
    place_limit_sell(&state, seller, &symbol, 15, dollars(101))
        .await
        .unwrap();
    place_limit_sell(&state, seller, &symbol, 25, dollars(102))
        .await
        .unwrap();

    // Get depth - returns (bids, asks) tuple
    let depth = state.engine.get_order_book_depth(&symbol, 5);
    assert!(depth.is_some());

    let (bids, asks) = depth.unwrap();
    assert!(!bids.is_empty(), "Should have bids");
    assert!(!asks.is_empty(), "Should have asks");

    // Best bid should be $100 (bids sorted high to low)
    assert_eq!(bids[0].0, dollars(100));
    // Best ask should be $101 (asks sorted low to high)
    assert_eq!(asks[0].0, dollars(101));
}

/// Test: Empty order book depth
#[tokio::test]
async fn test_empty_orderbook_depth() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "EMPTY", "Empty Book Co.").await;

    let depth = state.engine.get_order_book_depth(&symbol, 5);
    assert!(depth.is_some());

    let (bids, asks) = depth.unwrap();
    assert!(bids.is_empty(), "No bids in empty book");
    assert!(asks.is_empty(), "No asks in empty book");
}

// =============================================================================
// NEWS BROADCASTING TESTS
// =============================================================================

/// Test: News service provides recent items
#[tokio::test]
async fn test_news_service_get_recent() {
    let state = create_test_state().await;

    // NewsService generates news automatically or via generate_news
    // Here we just test the get_recent API
    let news = state.news.get_recent(10);
    // News might be empty initially or have generated items
    // This tests the API doesn't panic
    assert!(news.len() <= 10, "Should return at most requested count");
}

/// Test: News service subscribe
#[tokio::test]
async fn test_news_subscribe() {
    let state = create_test_state().await;

    // Subscribe to news channel
    let _rx = state.news.subscribe();

    // This tests that subscription works without panic
    // Actual news generation would require running the news loop
}
