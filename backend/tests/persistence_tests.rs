//! Persistence & Recovery Integration Tests
//!
//! Tests for: PERSIST-*, RECOVER-*, IDGEN-*

mod common;

use common::*;
use std::fs;
use stockmart_backend::infrastructure::id_generator::IdGenerator;
use tempfile::TempDir;

// =============================================================================
// DATA PERSISTENCE TESTS (PERSIST-*)
// =============================================================================

/// PERSIST-001: Users saved to JSON
#[tokio::test]
async fn test_users_saved_to_json() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state = create_test_state().await;

    // Create some users
    create_test_user(&state, "USER1", "User One", "pass1").await;
    create_test_user(&state, "USER2", "User Two", "pass2").await;
    create_test_user(&state, "USER3", "User Three", "pass3").await;

    // Create persistence service and save
    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir.clone(),
    );
    persistence.save_data().await;

    // Verify file exists and contains users
    let users_path = format!("{}/users.json", data_dir);
    assert!(fs::metadata(&users_path).is_ok(), "users.json should exist");

    let content = fs::read_to_string(&users_path).unwrap();
    assert!(content.contains("USER1"));
    assert!(content.contains("USER2"));
    assert!(content.contains("USER3"));
}

/// PERSIST-002: Companies saved to JSON
#[tokio::test]
async fn test_companies_saved_to_json() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state = create_test_state().await;

    // Create some companies
    create_test_company(&state, "AAPL", "Apple Inc.").await;
    create_test_company(&state, "GOOGL", "Alphabet Inc.").await;
    create_test_company(&state, "MSFT", "Microsoft Corp.").await;

    // Create persistence service and save
    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir.clone(),
    );
    persistence.save_data().await;

    // Verify file exists and contains companies
    let companies_path = format!("{}/companies.json", data_dir);
    assert!(
        fs::metadata(&companies_path).is_ok(),
        "companies.json should exist"
    );

    let content = fs::read_to_string(&companies_path).unwrap();
    assert!(content.contains("AAPL"));
    assert!(content.contains("GOOGL"));
    assert!(content.contains("MSFT"));
    assert!(content.contains("Apple Inc."));
}

// =============================================================================
// DATA RECOVERY TESTS (RECOVER-*)
// =============================================================================

/// RECOVER-001: Load users on startup
#[tokio::test]
async fn test_load_users_on_startup() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    // First, create and save some users
    let state1 = create_test_state().await;
    create_test_user(&state1, "LOADED_USER", "Loaded User", "pass").await;

    let persistence1 = stockmart_backend::service::persistence::PersistenceService::new(
        state1.user_repo.clone(),
        state1.company_repo.clone(),
        data_dir.clone(),
    );
    persistence1.save_data().await;

    // Now create a fresh state and load the data
    let state2 = create_test_state().await;
    let persistence2 = stockmart_backend::service::persistence::PersistenceService::new(
        state2.user_repo.clone(),
        state2.company_repo.clone(),
        data_dir.clone(),
    );
    persistence2.load_data().await;

    // User should be loaded
    let users = state2.user_repo.all().await.unwrap();
    assert!(
        users.iter().any(|u| u.regno == "LOADED_USER"),
        "User should be restored from disk"
    );
}

/// RECOVER-002: Load companies on startup
#[tokio::test]
async fn test_load_companies_on_startup() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    // First, create and save some companies
    let state1 = create_test_state().await;
    create_test_company(&state1, "SAVED_CO", "Saved Company").await;

    let persistence1 = stockmart_backend::service::persistence::PersistenceService::new(
        state1.user_repo.clone(),
        state1.company_repo.clone(),
        data_dir.clone(),
    );
    persistence1.save_data().await;

    // Now create a fresh state and load the data
    let state2 = create_test_state().await;
    let persistence2 = stockmart_backend::service::persistence::PersistenceService::new(
        state2.user_repo.clone(),
        state2.company_repo.clone(),
        data_dir.clone(),
    );
    persistence2.load_data().await;

    // Company should be loaded
    let companies = state2.company_repo.all().await.unwrap();
    assert!(
        companies.iter().any(|c| c.symbol == "SAVED_CO"),
        "Company should be restored from disk"
    );
}

/// RECOVER-005: Handle missing data files
#[tokio::test]
async fn test_handle_missing_data_files() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    // Create a fresh state with no data files
    let state = create_test_state().await;
    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir.clone(),
    );

    // Loading should not panic with missing files
    persistence.load_data().await;

    // State should be empty but valid
    let users = state.user_repo.all().await.unwrap();
    let companies = state.company_repo.all().await.unwrap();
    assert!(users.is_empty(), "Should have no users");
    assert!(companies.is_empty(), "Should have no companies");
}

/// RECOVER-006: Handle corrupt JSON
#[tokio::test]
async fn test_handle_corrupt_json() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    // Write corrupt JSON files
    let users_path = format!("{}/users.json", data_dir);
    let companies_path = format!("{}/companies.json", data_dir);
    fs::write(&users_path, "{ invalid json }").unwrap();
    fs::write(&companies_path, "not even close to json").unwrap();

    // Loading should not panic
    let state = create_test_state().await;
    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir,
    );

    // This should handle the error gracefully
    persistence.load_data().await;

    // State should remain empty
    let users = state.user_repo.all().await.unwrap();
    assert!(users.is_empty(), "Should have no users after corrupt load");
}

// =============================================================================
// ID GENERATOR TESTS (IDGEN-*)
// =============================================================================

/// IDGEN-001: User IDs monotonically increase
#[tokio::test]
async fn test_user_ids_monotonically_increase() {
    use stockmart_backend::infrastructure::id_generator::AtomicIdGenerator;

    let gen = AtomicIdGenerator::new();
    let mut prev_id = 0u64;

    for _ in 0..100 {
        let id = gen.next_id();
        assert!(id > prev_id, "ID {} should be greater than {}", id, prev_id);
        prev_id = id;
    }
}

/// IDGEN-002: Order IDs unique across generator instance
#[tokio::test]
async fn test_order_ids_unique() {
    use std::collections::HashSet;
    use stockmart_backend::infrastructure::id_generator::IdGenerators;

    let gens = IdGenerators::new();
    let mut ids = HashSet::new();

    for _ in 0..1000 {
        let id = gens.next_order_id();
        assert!(ids.insert(id), "Order ID {} should be unique", id);
    }
}

/// IDGEN-003: Trade IDs unique
#[tokio::test]
async fn test_trade_ids_unique() {
    use std::collections::HashSet;
    use stockmart_backend::infrastructure::id_generator::IdGenerators;

    let gens = IdGenerators::new();
    let mut ids = HashSet::new();

    for _ in 0..1000 {
        let id = gens.next_trade_id();
        assert!(ids.insert(id), "Trade ID {} should be unique", id);
    }
}

/// IDGEN-004: Thread-safe generation
#[tokio::test]
async fn test_thread_safe_id_generation() {
    use std::sync::Arc;
    use stockmart_backend::infrastructure::id_generator::AtomicIdGenerator;

    let gen = Arc::new(AtomicIdGenerator::new());
    let mut handles = vec![];

    // Spawn multiple tasks generating IDs concurrently
    for _ in 0..10 {
        let gen_clone = Arc::clone(&gen);
        handles.push(tokio::spawn(async move {
            let mut ids = Vec::new();
            for _ in 0..100 {
                ids.push(gen_clone.next_id());
            }
            ids
        }));
    }

    // Collect all IDs
    let mut all_ids = std::collections::HashSet::new();
    for handle in handles {
        let ids = handle.await.unwrap();
        for id in ids {
            assert!(
                all_ids.insert(id),
                "ID {} should be unique across threads",
                id
            );
        }
    }

    // Should have generated 1000 unique IDs
    assert_eq!(all_ids.len(), 1000);
}

/// IDGEN-005: Reset from max persisted ID
#[tokio::test]
async fn test_reset_from_max_persisted_id() {
    use stockmart_backend::infrastructure::id_generator::{AtomicIdGenerator, IdGenerator};

    // Simulate having persisted max ID of 500
    let gen = AtomicIdGenerator::from_max_id(500);

    // Next ID should be 501
    let id1 = gen.next_id();
    assert_eq!(id1, 501, "Should continue from max + 1");

    let id2 = gen.next_id();
    assert_eq!(id2, 502);

    // Test reset functionality
    gen.reset(1000);
    let id3 = gen.next_id();
    assert_eq!(id3, 1000, "After reset, should start from reset value");
}

/// Test: IdGenerators collection properly initializes
#[tokio::test]
async fn test_id_generators_collection() {
    use stockmart_backend::infrastructure::id_generator::IdGenerators;

    let gens = IdGenerators::new();

    // Each generator should start from 1
    assert_eq!(gens.next_user_id(), 1);
    assert_eq!(gens.next_company_id(), 1);
    assert_eq!(gens.next_order_id(), 1);
    assert_eq!(gens.next_trade_id(), 1);
    assert_eq!(gens.next_sync_id(), 1);

    // Second calls should return 2
    assert_eq!(gens.next_user_id(), 2);
    assert_eq!(gens.next_company_id(), 2);
}

/// Test: IdGenerators from max IDs
#[tokio::test]
async fn test_id_generators_from_max_ids() {
    use stockmart_backend::infrastructure::id_generator::IdGenerators;

    let gens = IdGenerators::from_max_ids(100, 50, 200, 300);

    // Should continue from max + 1
    assert_eq!(gens.next_user_id(), 101);
    assert_eq!(gens.next_company_id(), 51);
    assert_eq!(gens.next_order_id(), 201);
    assert_eq!(gens.next_trade_id(), 301);
    // Sync always starts fresh
    assert_eq!(gens.next_sync_id(), 1);
}

// =============================================================================
// DATA INTEGRITY TESTS
// =============================================================================

/// Test: Round-trip persistence preserves user data
#[tokio::test]
async fn test_roundtrip_user_data() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state1 = create_test_state().await;

    // Create user with portfolio
    let user_id = create_test_user_with_portfolio(
        &state1,
        "ROUNDTRIP",
        "Round Trip User",
        dollars(50_000),
        vec![("AAPL".to_string(), 100)],
    )
    .await;

    // Save
    let persistence1 = stockmart_backend::service::persistence::PersistenceService::new(
        state1.user_repo.clone(),
        state1.company_repo.clone(),
        data_dir.clone(),
    );
    persistence1.save_data().await;

    // Load into fresh state
    let state2 = create_test_state().await;
    let persistence2 = stockmart_backend::service::persistence::PersistenceService::new(
        state2.user_repo.clone(),
        state2.company_repo.clone(),
        data_dir,
    );
    persistence2.load_data().await;

    // Verify user data preserved
    let loaded_user = state2.user_repo.find_by_id(user_id).await.unwrap();
    assert!(loaded_user.is_some(), "User should be loaded");

    let user = loaded_user.unwrap();
    assert_eq!(user.regno, "ROUNDTRIP");
    assert_eq!(user.name, "Round Trip User");
    assert_eq!(user.money, dollars(50_000));

    // Portfolio should be preserved
    let position = user.portfolio.iter().find(|p| p.symbol == "AAPL");
    assert!(position.is_some(), "AAPL position should be preserved");
    assert_eq!(position.unwrap().qty, 100);
}

/// Test: Round-trip persistence preserves company data
#[tokio::test]
async fn test_roundtrip_company_data() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state1 = create_test_state().await;

    // Create company
    create_test_company(&state1, "ROUNDTRIP", "Round Trip Corp").await;

    // Modify the company (sector and volatility are mutable)
    let mut company = state1
        .company_repo
        .find_by_symbol("ROUNDTRIP")
        .await
        .unwrap()
        .unwrap();
    company.sector = "Modified Sector".to_string();
    company.volatility = 500;
    state1.company_repo.save(company).await.unwrap();

    // Save
    let persistence1 = stockmart_backend::service::persistence::PersistenceService::new(
        state1.user_repo.clone(),
        state1.company_repo.clone(),
        data_dir.clone(),
    );
    persistence1.save_data().await;

    // Load into fresh state
    let state2 = create_test_state().await;
    let persistence2 = stockmart_backend::service::persistence::PersistenceService::new(
        state2.user_repo.clone(),
        state2.company_repo.clone(),
        data_dir,
    );
    persistence2.load_data().await;

    // Verify company data preserved
    let loaded_company = state2
        .company_repo
        .find_by_symbol("ROUNDTRIP")
        .await
        .unwrap();
    assert!(loaded_company.is_some(), "Company should be loaded");

    let company = loaded_company.unwrap();
    assert_eq!(company.symbol, "ROUNDTRIP");
    assert_eq!(company.name, "Round Trip Corp");
    assert_eq!(company.sector, "Modified Sector");
    assert_eq!(company.volatility, 500);
}

/// Test: Persistence handles empty state
#[tokio::test]
async fn test_persistence_empty_state() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state = create_test_state().await;

    // Save empty state
    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir.clone(),
    );
    persistence.save_data().await;

    // Files should exist with empty arrays
    let users_content = fs::read_to_string(format!("{}/users.json", data_dir)).unwrap();
    let companies_content = fs::read_to_string(format!("{}/companies.json", data_dir)).unwrap();

    assert!(users_content.trim() == "[]", "Users should be empty array");
    assert!(
        companies_content.trim() == "[]",
        "Companies should be empty array"
    );
}

/// Test: Multiple save operations are idempotent
#[tokio::test]
async fn test_multiple_saves_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap().to_string();

    let state = create_test_state().await;
    create_test_user(&state, "IDEMPOTENT", "Idempotent User", "pass").await;

    let persistence = stockmart_backend::service::persistence::PersistenceService::new(
        state.user_repo.clone(),
        state.company_repo.clone(),
        data_dir.clone(),
    );

    // Save multiple times
    persistence.save_data().await;
    let content1 = fs::read_to_string(format!("{}/users.json", data_dir)).unwrap();

    persistence.save_data().await;
    let content2 = fs::read_to_string(format!("{}/users.json", data_dir)).unwrap();

    persistence.save_data().await;
    let content3 = fs::read_to_string(format!("{}/users.json", data_dir)).unwrap();

    // All saves should produce identical content
    assert_eq!(content1, content2, "Multiple saves should be idempotent");
    assert_eq!(content2, content3, "Multiple saves should be idempotent");
}
