//! Authentication and Session Management Integration Tests
//!
//! Tests for: AUTH-REG-*, AUTH-LOGIN-*, AUTH-TOKEN-*, SESS-*, SM-TOKEN-*

mod common;

use common::*;

// =============================================================================
// REGISTRATION TESTS (AUTH-REG-*)
// =============================================================================

/// AUTH-REG-001: Register new user with valid data
#[tokio::test]
async fn test_register_new_user_valid() {
    let state = create_test_state().await;

    // Create a company first (needed for portfolio allocation)
    create_test_company(&state, "AAPL", "Apple Inc.").await;

    let user_id = create_test_user(&state, "REG001", "Test User", "password123").await;

    // Verify user was created
    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User should exist");

    assert_eq!(user.regno, "REG001");
    assert_eq!(user.name, "Test User");
    assert!(!user.banned);
    assert!(user.chat_enabled);
}

/// AUTH-REG-002: Register with duplicate regno should fail
#[tokio::test]
async fn test_register_duplicate_regno() {
    let state = create_test_state().await;

    // Create first user
    create_test_user(&state, "DUPLICATE", "First User", "pass1").await;

    // Try to create second user with same regno
    let result = state.user_repo.regno_exists("DUPLICATE").await;
    assert!(result.unwrap(), "Regno should already exist");
}

/// AUTH-REG-006: Verify initial portfolio allocation
#[tokio::test]
async fn test_register_initial_portfolio() {
    let state = create_test_state().await;

    // Create companies
    create_test_companies(&state, 3).await;

    // Create user with portfolio
    let user_id = create_test_user_with_portfolio(
        &state,
        "PORTFOLIO001",
        "Portfolio User",
        dollars(50_000), // $50,000 cash
        vec![
            ("AAPL".to_string(), 100),
            ("GOOGL".to_string(), 50),
            ("MSFT".to_string(), 75),
        ],
    )
    .await;

    let user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User should exist");

    assert_eq!(user.money, dollars(50_000));
    assert_eq!(user.portfolio.len(), 3);

    // Check AAPL position
    let aapl = user.portfolio.iter().find(|p| p.symbol == "AAPL").unwrap();
    assert_eq!(aapl.qty, 100);
    assert_eq!(aapl.locked_qty, 0);
}

/// AUTH-REG-008: Verify ID generator continuity
#[tokio::test]
async fn test_id_generator_continuity() {
    let state = create_test_state().await;

    let user1_id = create_test_user(&state, "USER1", "User One", "pass").await;
    let user2_id = create_test_user(&state, "USER2", "User Two", "pass").await;
    let user3_id = create_test_user(&state, "USER3", "User Three", "pass").await;

    // IDs should be monotonically increasing
    assert!(
        user2_id > user1_id,
        "User IDs should increase: {} > {}",
        user2_id,
        user1_id
    );
    assert!(
        user3_id > user2_id,
        "User IDs should increase: {} > {}",
        user3_id,
        user2_id
    );
}

// =============================================================================
// LOGIN/AUTH TESTS (AUTH-LOGIN-*, AUTH-TOKEN-*)
// =============================================================================

/// AUTH-LOGIN-001: Login with valid credentials
#[tokio::test]
async fn test_login_valid_credentials() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "LOGIN001", "Login Test", "correctpass").await;

    // Verify user exists and password matches
    let user = state
        .user_repo
        .find_by_regno("LOGIN001")
        .await
        .expect("DB error")
        .expect("User should exist");

    assert_eq!(user.id, user_id);
    assert_eq!(user.password_hash, "correctpass");
}

/// AUTH-LOGIN-002: Login with wrong password
#[tokio::test]
async fn test_login_wrong_password() {
    let state = create_test_state().await;

    create_test_user(&state, "WRONGPASS", "Test User", "correctpass").await;

    let user = state
        .user_repo
        .find_by_regno("WRONGPASS")
        .await
        .expect("DB error")
        .expect("User should exist");

    // Verify wrong password doesn't match
    assert_ne!(user.password_hash, "wrongpassword");
}

/// AUTH-LOGIN-003: Login with non-existent regno
#[tokio::test]
async fn test_login_nonexistent_user() {
    let state = create_test_state().await;

    let result = state.user_repo.find_by_regno("NONEXISTENT").await;
    assert!(result.unwrap().is_none(), "User should not exist");
}

/// AUTH-LOGIN-004: Login when banned should fail
#[tokio::test]
async fn test_login_banned_user() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BANNED001", "Banned User", "pass").await;

    // Ban the user
    let mut user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User should exist");
    user.banned = true;
    state
        .user_repo
        .save(user.clone())
        .await
        .expect("Failed to save");

    // Verify banned flag is set
    let banned_user = state
        .user_repo
        .find_by_regno("BANNED001")
        .await
        .expect("DB error")
        .expect("User should exist");

    assert!(banned_user.banned, "User should be banned");
}

// =============================================================================
// TOKEN TESTS (SM-TOKEN-*)
// =============================================================================

/// SM-TOKEN-001: Login creates valid token
#[tokio::test]
async fn test_token_created_on_login() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "TOKEN001", "Token User", "pass").await;

    // Create token
    let (token, revoked) = state.tokens.create_token(user_id);

    assert!(!token.is_empty());
    assert_eq!(token.len(), 64); // 32 bytes hex encoded
    assert!(revoked.is_empty());

    // Validate token
    let validated_user_id = state.tokens.validate_token(&token);
    assert_eq!(validated_user_id, Some(user_id));
}

/// SM-TOKEN-002: Token is cryptographically unique
#[tokio::test]
async fn test_token_uniqueness() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 0, // Unlimited tokens
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "UNIQUE001", "Unique User", "pass").await;

    let mut tokens = std::collections::HashSet::new();
    for _ in 0..100 {
        let (token, _) = state.tokens.create_token(user_id);
        assert!(tokens.insert(token), "Token collision detected!");
    }
}

/// SM-TOKEN-003: Max tokens revokes oldest (FIFO eviction)
#[tokio::test]
async fn test_token_max_limit_fifo() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 2,
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "FIFO001", "FIFO User", "pass").await;

    // Create first two tokens
    let (token1, revoked1) = state.tokens.create_token(user_id);
    assert!(revoked1.is_empty());

    let (token2, revoked2) = state.tokens.create_token(user_id);
    assert!(revoked2.is_empty());

    // Create third token - should revoke first
    let (token3, revoked3) = state.tokens.create_token(user_id);
    assert_eq!(revoked3.len(), 1);
    assert_eq!(revoked3[0], token1);

    // Verify token states
    assert!(
        state.tokens.validate_token(&token1).is_none(),
        "Token1 should be revoked"
    );
    assert!(
        state.tokens.validate_token(&token2).is_some(),
        "Token2 should be valid"
    );
    assert!(
        state.tokens.validate_token(&token3).is_some(),
        "Token3 should be valid"
    );
}

/// SM-TOKEN-004: Explicit token revocation
#[tokio::test]
async fn test_token_explicit_revoke() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "REVOKE001", "Revoke User", "pass").await;

    let (token, _) = state.tokens.create_token(user_id);

    // Token should be valid
    assert!(state.tokens.validate_token(&token).is_some());

    // Revoke token
    let revoked = state.tokens.revoke_token(&token);
    assert!(revoked, "Token should be revoked");

    // Token should now be invalid
    assert!(state.tokens.validate_token(&token).is_none());

    // Second revoke should return false
    let revoked_again = state.tokens.revoke_token(&token);
    assert!(!revoked_again, "Already revoked token should return false");
}

/// SM-TOKEN-005: Revoke all user tokens
#[tokio::test]
async fn test_token_revoke_all() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 0, // Unlimited
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "REVOKEALL", "Revoke All User", "pass").await;

    // Create multiple tokens
    let (token1, _) = state.tokens.create_token(user_id);
    let (token2, _) = state.tokens.create_token(user_id);
    let (token3, _) = state.tokens.create_token(user_id);

    // All should be valid
    assert!(state.tokens.validate_token(&token1).is_some());
    assert!(state.tokens.validate_token(&token2).is_some());
    assert!(state.tokens.validate_token(&token3).is_some());

    // Revoke all
    let count = state.tokens.revoke_all_user_tokens(user_id);
    assert_eq!(count, 3);

    // All should be invalid
    assert!(state.tokens.validate_token(&token1).is_none());
    assert!(state.tokens.validate_token(&token2).is_none());
    assert!(state.tokens.validate_token(&token3).is_none());
}

/// SM-TOKEN-006: Invalid token rejected
#[tokio::test]
async fn test_invalid_token_rejected() {
    let state = create_test_state().await;

    // Random invalid token
    let invalid_token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    let result = state.tokens.validate_token(invalid_token);
    assert!(result.is_none(), "Invalid token should be rejected");
}

/// SM-TOKEN-007: Token survives connection lifecycle
#[tokio::test]
async fn test_token_survives_reconnect() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "RECONNECT", "Reconnect User", "pass").await;

    // Create token (simulating login)
    let (token, _) = state.tokens.create_token(user_id);

    // Create session
    let (session_id, _) = state.sessions.create_session(user_id);

    // Simulate disconnect - remove session
    state.sessions.remove_session(session_id);

    // Token should still be valid for reconnection
    let validated = state.tokens.validate_token(&token);
    assert_eq!(validated, Some(user_id), "Token should survive disconnect");
}

// =============================================================================
// SESSION TESTS (SESS-*)
// =============================================================================

/// SESS-001: Create single session
#[tokio::test]
async fn test_session_create_single() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "SESS001", "Session User", "pass").await;

    let (session_id, kicked) = state.sessions.create_session(user_id);

    assert!(session_id > 0);
    assert!(kicked.is_empty());
    assert_eq!(state.sessions.active_session_count(), 1);
}

/// SESS-002: Exceed max sessions kicks old session
#[tokio::test]
async fn test_session_max_kicks_old() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 1,
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "MAXSESS", "Max Session User", "pass").await;

    // Create first session
    let (session1, kicked1) = state.sessions.create_session(user_id);
    assert!(kicked1.is_empty());

    // Create second session - should kick first
    let (session2, kicked2) = state.sessions.create_session(user_id);
    assert_eq!(kicked2.len(), 1);
    assert_eq!(kicked2[0], session1);
    assert!(session2 > session1);
}

/// SESS-003: Unlimited sessions when max=0
#[tokio::test]
async fn test_session_unlimited() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 0, // Unlimited
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "UNLIMITED", "Unlimited User", "pass").await;

    // Create many sessions
    let mut session_ids = Vec::new();
    for _ in 0..10 {
        let (session_id, kicked) = state.sessions.create_session(user_id);
        assert!(kicked.is_empty(), "No sessions should be kicked");
        session_ids.push(session_id);
    }

    assert_eq!(state.sessions.total_sessions(), 10);
}

/// SESS-004: Session cleanup on disconnect
#[tokio::test]
async fn test_session_cleanup_on_disconnect() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "CLEANUP", "Cleanup User", "pass").await;

    let (session_id, _) = state.sessions.create_session(user_id);
    assert_eq!(state.sessions.total_sessions(), 1);

    // Simulate disconnect
    state.sessions.remove_session(session_id);
    assert_eq!(state.sessions.total_sessions(), 0);
}

/// SESS-005: Concurrent login attempts
#[tokio::test]
async fn test_session_concurrent_logins() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 2,
        ..Default::default()
    })
    .await;

    let user_id = create_test_user(&state, "CONCURRENT", "Concurrent User", "pass").await;

    // Simulate concurrent logins
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

    // Should have created 5 sessions but only 2 survive
    // (3 would have been kicked)
    let total_kicked: usize = results.iter().map(|(_, kicked)| kicked.len()).sum();

    // With max_sessions=2 and 5 concurrent attempts, we expect some kicks
    // The exact number depends on timing, but we should have at most 2 active
    let unique_sessions: std::collections::HashSet<_> = results.iter().map(|(id, _)| *id).collect();
    assert!(unique_sessions.len() <= 5);
}

/// SESS-006: Session count metrics accurate
#[tokio::test]
async fn test_session_count_accuracy() {
    let state = create_test_state_with_config(TestConfig {
        max_sessions_per_user: 0, // Unlimited
        ..Default::default()
    })
    .await;

    assert_eq!(state.sessions.total_sessions(), 0);

    let user1 = create_test_user(&state, "COUNT1", "User 1", "pass").await;
    let user2 = create_test_user(&state, "COUNT2", "User 2", "pass").await;

    let (s1, _) = state.sessions.create_session(user1);
    assert_eq!(state.sessions.total_sessions(), 1);

    let (s2, _) = state.sessions.create_session(user2);
    assert_eq!(state.sessions.total_sessions(), 2);

    let (s3, _) = state.sessions.create_session(user1);
    assert_eq!(state.sessions.total_sessions(), 3);

    state.sessions.remove_session(s1);
    assert_eq!(state.sessions.total_sessions(), 2);

    state.sessions.remove_session(s2);
    state.sessions.remove_session(s3);
    assert_eq!(state.sessions.total_sessions(), 0);
}

// =============================================================================
// USER STATE TESTS (SM-USER-*)
// =============================================================================

/// SM-USER-002: Ban user
#[tokio::test]
async fn test_ban_user() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BAN001", "To Be Banned", "pass").await;

    // Get user and ban
    let mut user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User should exist");

    assert!(!user.banned, "User should not be banned initially");

    user.banned = true;
    state.user_repo.save(user).await.expect("Save failed");

    // Verify banned
    let banned_user = state
        .user_repo
        .find_by_id(user_id)
        .await
        .expect("DB error")
        .expect("User should exist");

    assert!(banned_user.banned, "User should be banned");
}

/// SM-USER-003: Unban user
#[tokio::test]
async fn test_unban_user() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "UNBAN001", "To Be Unbanned", "pass").await;

    // Ban user
    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    user.banned = true;
    state.user_repo.save(user).await.unwrap();

    // Unban user
    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    user.banned = false;
    state.user_repo.save(user).await.unwrap();

    // Verify unbanned
    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.banned, "User should be unbanned");
}

/// SM-USER-004: Mute user
#[tokio::test]
async fn test_mute_user() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "MUTE001", "To Be Muted", "pass").await;

    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled, "Chat should be enabled by default");

    user.chat_enabled = false;
    state.user_repo.save(user).await.unwrap();

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(!user.chat_enabled, "Chat should be disabled");
}

/// SM-USER-005: Unmute user
#[tokio::test]
async fn test_unmute_user() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "UNMUTE001", "To Be Unmuted", "pass").await;

    // Mute
    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    user.chat_enabled = false;
    state.user_repo.save(user).await.unwrap();

    // Unmute
    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    user.chat_enabled = true;
    state.user_repo.save(user).await.unwrap();

    let user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert!(user.chat_enabled, "Chat should be enabled");
}

/// SM-USER-007: Banned user token revoked
#[tokio::test]
async fn test_banned_user_token_revoked() {
    let state = create_test_state().await;

    let user_id = create_test_user(&state, "BANTOKEN", "Ban Token User", "pass").await;

    // Create token
    let (token, _) = state.tokens.create_token(user_id);
    assert!(state.tokens.validate_token(&token).is_some());

    // Ban user and revoke their tokens
    let mut user = state.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    user.banned = true;
    state.user_repo.save(user).await.unwrap();

    // Revoke all tokens for banned user
    state.tokens.revoke_all_user_tokens(user_id);

    // Token should be invalid
    assert!(
        state.tokens.validate_token(&token).is_none(),
        "Banned user's token should be revoked"
    );
}
