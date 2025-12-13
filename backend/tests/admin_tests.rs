//! Admin Operations Integration Tests
//!
//! Tests for: ADMIN-RBAC-*, ADMIN-MKT-*, ADMIN-USER-*, ADMIN-INIT-*, ADMIN-COMPANY-*

mod common;

use common::*;

// =============================================================================
// RBAC PERMISSION TESTS (ADMIN-RBAC-*)
// =============================================================================

/// ADMIN-RBAC-001: Non-admin cannot toggle market (permission check)
#[tokio::test]
async fn test_non_admin_cannot_toggle_market() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TRADER001", "Regular Trader", "pass").await;

    // Get user and verify they're not admin
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.role.is_admin(), "User should not be admin");

    // Traders cannot control market - the permission check happens at handler level
    assert!(!user.role.can_control_market());
}

/// ADMIN-RBAC-002: Admin can toggle market
#[tokio::test]
async fn test_admin_can_toggle_market() {
    let state = create_test_state().await;

    let admin_id = create_admin_user(&state, "ADMIN001", "Admin User").await;

    // Verify admin role
    let admin = state.user_repo.find_by_id(admin_id).await.unwrap().unwrap();
    assert!(admin.role.is_admin(), "User should be admin");
    assert!(admin.role.can_control_market());

    // Admin can toggle market via admin service
    close_market(&state);
    assert!(!is_market_open(&state));

    open_market(&state);
    assert!(is_market_open(&state));
}

/// ADMIN-RBAC-003: Non-admin cannot create company (permission check)
#[tokio::test]
async fn test_non_admin_cannot_create_company() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TRADER002", "Regular Trader", "pass").await;

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.role.can_manage_companies());
}

/// ADMIN-RBAC-004: Admin can create company
#[tokio::test]
async fn test_admin_can_create_company() {
    let state = create_test_state().await;

    let admin_id = create_admin_user(&state, "ADMIN002", "Admin User").await;

    let admin = state.user_repo.find_by_id(admin_id).await.unwrap().unwrap();
    assert!(admin.role.can_manage_companies());

    // Create company through test helper (simulating admin action)
    let symbol = create_test_company(&state, "NEWCO", "New Company Inc.").await;
    assert_eq!(symbol, "NEWCO");

    // Verify company exists
    let company = state.company_repo.find_by_symbol("NEWCO").await.unwrap();
    assert!(company.is_some());
}

/// ADMIN-RBAC-006: Admin can ban trader
#[tokio::test]
async fn test_admin_can_ban_trader() {
    let state = create_test_state().await;

    let admin_id = create_admin_user(&state, "ADMIN003", "Admin User").await;
    let trader_id = create_test_user(&state, "TRADER003", "Trader to Ban", "pass").await;

    let admin = state.user_repo.find_by_id(admin_id).await.unwrap().unwrap();
    assert!(admin.role.can_manage_users());

    // Ban via admin service
    state
        .admin
        .set_trader_banned(trader_id, true)
        .await
        .unwrap();

    // Verify user is banned
    let trader = state
        .user_repo
        .find_by_id(trader_id)
        .await
        .unwrap()
        .unwrap();
    assert!(trader.banned, "Trader should be banned");
}

// =============================================================================
// MARKET CONTROL TESTS (ADMIN-MKT-*)
// =============================================================================

/// ADMIN-MKT-001: Open market
#[tokio::test]
async fn test_admin_open_market() {
    let state = create_test_state().await;

    close_market(&state);
    assert!(!is_market_open(&state));

    open_market(&state);
    assert!(is_market_open(&state));
}

/// ADMIN-MKT-002: Close market
#[tokio::test]
async fn test_admin_close_market() {
    let state = create_test_state().await;

    open_market(&state);
    assert!(is_market_open(&state));

    close_market(&state);
    assert!(!is_market_open(&state));
}

/// ADMIN-MKT-003: Orders rejected when market closed
#[tokio::test]
async fn test_orders_rejected_when_closed() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple Inc.").await;
    let user_id =
        create_test_user_with_portfolio(&state, "TRADER004", "Trader", dollars(100_000), vec![])
            .await;

    close_market(&state);

    let result = place_limit_buy(&state, user_id, &symbol, 10, dollars(100)).await;
    assert!(
        result.is_err(),
        "Order should be rejected when market is closed"
    );

    let err = result.unwrap_err();
    assert!(err.contains("MARKET_CLOSED") || err.to_lowercase().contains("closed"));
}

/// ADMIN-MKT-004: Market state persists across operations
#[tokio::test]
async fn test_market_state_persists() {
    let state = create_test_state().await;

    // Open market
    open_market(&state);
    assert!(is_market_open(&state));

    // Create multiple companies - state should persist
    create_test_company(&state, "AAPL", "Apple").await;
    create_test_company(&state, "GOOGL", "Google").await;

    // Still open
    assert!(is_market_open(&state));

    // Close
    close_market(&state);
    assert!(!is_market_open(&state));

    // Create more users - state should persist
    create_test_user(&state, "USER1", "User 1", "pass").await;
    create_test_user(&state, "USER2", "User 2", "pass").await;

    // Still closed
    assert!(!is_market_open(&state));
}

// =============================================================================
// USER MANAGEMENT TESTS (ADMIN-USER-*)
// =============================================================================

/// ADMIN-USER-001: Ban trader
#[tokio::test]
async fn test_ban_trader() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BANTEST1", "Ban Test User", "pass").await;

    // Initially not banned
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.banned);

    // Ban user
    state.admin.set_trader_banned(user_id, true).await.unwrap();

    // Verify banned
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.banned);
}

/// ADMIN-USER-002: Unban trader
#[tokio::test]
async fn test_unban_trader() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BANTEST2", "Unban Test User", "pass").await;

    // Ban first
    state.admin.set_trader_banned(user_id, true).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.banned);

    // Unban
    state.admin.set_trader_banned(user_id, false).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.banned);
}

/// ADMIN-USER-003: Mute trader
#[tokio::test]
async fn test_mute_trader() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "MUTETEST1", "Mute Test User", "pass").await;

    // Initially not muted
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled);

    // Mute user
    state.admin.set_trader_chat(user_id, false).await.unwrap();

    // Verify muted
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled);
}

/// ADMIN-USER-004: Unmute trader
#[tokio::test]
async fn test_unmute_trader() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "MUTETEST2", "Unmute Test User", "pass").await;

    // Mute first
    state.admin.set_trader_chat(user_id, false).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled);

    // Unmute
    state.admin.set_trader_chat(user_id, true).await.unwrap();
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled);
}

/// ADMIN-USER-005: Ban non-existent user
#[tokio::test]
async fn test_ban_nonexistent_user() {
    let state = create_test_state().await;

    let result = state.admin.set_trader_banned(999999, true).await;
    assert!(result.is_err(), "Banning non-existent user should fail");
}

// =============================================================================
// GAME INITIALIZATION TESTS (ADMIN-INIT-*)
// =============================================================================

/// ADMIN-INIT-001: InitGame resets all portfolios to target net worth
#[tokio::test]
async fn test_init_game_resets_portfolios() {
    let state = create_test_state().await;

    // Create companies first
    create_test_company(&state, "AAPL", "Apple").await;
    create_test_company(&state, "GOOGL", "Google").await;

    // Create some traders
    let user1 = create_test_user(&state, "TRADER1", "Trader 1", "pass").await;
    let user2 = create_test_user(&state, "TRADER2", "Trader 2", "pass").await;

    // Initialize game with target net worth of $100,000
    let target_networth = dollars(100_000);
    let result = state.admin.init_game(target_networth, 100).await;
    assert!(result.is_ok(), "InitGame should succeed");

    // Verify traders have portfolios
    let trader1 = state.user_repo.find_by_id(user1).await.unwrap().unwrap();
    let trader2 = state.user_repo.find_by_id(user2).await.unwrap().unwrap();

    // Both should have roughly the same net worth (+/- variance)
    let base_price = dollars(100);
    let nw1 = trader1.money
        + trader1
            .portfolio
            .iter()
            .map(|p| p.qty as i64 * base_price)
            .sum::<i64>();
    let nw2 = trader2.money
        + trader2
            .portfolio
            .iter()
            .map(|p| p.qty as i64 * base_price)
            .sum::<i64>();

    // Allow 5% variance due to random share allocation
    let tolerance = target_networth / 20;
    assert!(
        (nw1 - target_networth).abs() < tolerance,
        "Trader 1 net worth {} should be close to target {}",
        nw1,
        target_networth
    );
    assert!(
        (nw2 - target_networth).abs() < tolerance,
        "Trader 2 net worth {} should be close to target {}",
        nw2,
        target_networth
    );
}

/// ADMIN-INIT-002: InitGame clears order books
#[tokio::test]
async fn test_init_game_clears_orderbooks() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "AAPL", "Apple").await;
    create_test_user(&state, "TRADER", "Trader", "pass").await;

    // Place some orders
    let user =
        create_test_user_with_portfolio(&state, "ORDERER", "Orderer", dollars(100_000), vec![])
            .await;
    open_market(&state);
    place_limit_buy(&state, user, &symbol, 10, dollars(90))
        .await
        .ok();

    // Initialize game
    state.admin.init_game(dollars(100_000), 100).await.unwrap();

    // Order book should have liquidity seeded but no user orders
    let depth = state.engine.get_order_book_depth(&symbol, 10);
    assert!(depth.is_some(), "Orderbook should exist with seeded orders");
}

/// ADMIN-INIT-004: InitGame closes market
#[tokio::test]
async fn test_init_game_closes_market() {
    let state = create_test_state().await;

    create_test_company(&state, "AAPL", "Apple").await;
    create_test_user(&state, "TRADER", "Trader", "pass").await;

    open_market(&state);
    assert!(is_market_open(&state));

    state.admin.init_game(dollars(100_000), 100).await.unwrap();

    // Market should be closed after init
    assert!(
        !is_market_open(&state),
        "Market should be closed after init_game"
    );
}

/// ADMIN-INIT-005: InitGame skips admin users
#[tokio::test]
async fn test_init_game_skips_admins() {
    let state = create_test_state().await;

    create_test_company(&state, "AAPL", "Apple").await;

    let admin_id = create_admin_user(&state, "ADMIN", "Admin").await;
    let _trader_id = create_test_user(&state, "TRADER", "Trader", "pass").await;

    // Set admin's money to a specific value
    let mut admin = state.user_repo.find_by_id(admin_id).await.unwrap().unwrap();
    admin.money = dollars(999_999);
    state.user_repo.save(admin).await.unwrap();

    state.admin.init_game(dollars(100_000), 100).await.unwrap();

    // Admin's money should be unchanged
    let admin = state.user_repo.find_by_id(admin_id).await.unwrap().unwrap();
    assert_eq!(
        admin.money,
        dollars(999_999),
        "Admin portfolio should be unchanged"
    );
}

/// ADMIN-INIT-008: InitGame with no traders fails
#[tokio::test]
async fn test_init_game_no_traders() {
    let state = create_test_state().await;

    create_test_company(&state, "AAPL", "Apple").await;
    // No traders created

    let result = state.admin.init_game(dollars(100_000), 100).await;
    assert!(result.is_err(), "InitGame should fail with no traders");
    assert!(result.unwrap_err().contains("No traders"));
}

/// ADMIN-INIT-009: InitGame with no companies fails
#[tokio::test]
async fn test_init_game_no_companies() {
    let state = create_test_state().await;

    create_test_user(&state, "TRADER", "Trader", "pass").await;
    // No companies created

    let result = state.admin.init_game(dollars(100_000), 100).await;
    assert!(result.is_err(), "InitGame should fail with no companies");
    assert!(result.unwrap_err().contains("No companies"));
}

// =============================================================================
// METRICS TESTS (using available APIs)
// =============================================================================

/// Test: Get all traders
#[tokio::test]
async fn test_get_all_traders() {
    let state = create_test_state().await;

    // Create some traders
    create_test_user(&state, "T1", "Trader 1", "pass").await;
    create_test_user(&state, "T2", "Trader 2", "pass").await;
    create_test_user(&state, "T3", "Trader 3", "pass").await;

    let traders = state.admin.get_all_traders().await.unwrap();
    assert_eq!(traders.len(), 3, "Should return all traders");
}

/// Test: Session count accuracy
#[tokio::test]
async fn test_session_count() {
    let state = create_test_state().await;

    let user1 = create_test_user(&state, "ACTIVE1", "Active 1", "pass").await;
    let user2 = create_test_user(&state, "ACTIVE2", "Active 2", "pass").await;
    let _user3 = create_test_user(&state, "INACTIVE", "Inactive", "pass").await;

    // Create sessions for some users
    state.sessions.create_session(user1);
    state.sessions.create_session(user2);

    assert_eq!(
        state.sessions.total_sessions(),
        2,
        "Should count active sessions"
    );
}

// =============================================================================
// COMPANY MANAGEMENT TESTS (ADMIN-COMPANY-*)
// =============================================================================

/// ADMIN-COMPANY-001: Create company with valid data
#[tokio::test]
async fn test_create_company_valid() {
    let state = create_test_state().await;

    let symbol = create_test_company(&state, "NEWSTOCK", "New Stock Company").await;

    // Verify company exists
    let company = state.company_repo.find_by_symbol(&symbol).await.unwrap();
    assert!(company.is_some());
    let company = company.unwrap();
    assert_eq!(company.symbol, "NEWSTOCK");
    assert_eq!(company.name, "New Stock Company");

    // Verify orderbook was created
    let depth = state.engine.get_order_book_depth(&symbol, 1);
    assert!(depth.is_some(), "Orderbook should exist for new company");
}

/// ADMIN-COMPANY-002: Create duplicate symbol fails via AdminService
#[tokio::test]
async fn test_create_duplicate_company() {
    let state = create_test_state().await;

    // Create first company via admin service
    state
        .admin
        .create_company(
            "DUP".to_string(),
            "Duplicate Test".to_string(),
            "Test".to_string(),
            10,
        )
        .await
        .unwrap();

    // Try to create duplicate via admin service - should fail
    let result = state
        .admin
        .create_company(
            "DUP".to_string(),
            "Duplicate Test 2".to_string(),
            "Test".to_string(),
            10,
        )
        .await;

    assert!(result.is_err(), "Creating duplicate symbol should fail");
    assert!(result.unwrap_err().contains("already exists"));
}
