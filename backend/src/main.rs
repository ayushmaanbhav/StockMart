//! StockMart Backend - Main entry point.
//!
//! This module initializes all services and starts the HTTP server
//! with graceful shutdown support.

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod api;
mod config;
mod domain;
mod infrastructure;
mod presentation;
mod service;

use crate::api::ws::{ws_handler, AppState};
use crate::config::ConfigService;
use crate::domain::models::{Company, User};
use crate::domain::{CompanyRepository, UserRepository};
use crate::infrastructure::id_generator::IdGenerators;
use crate::infrastructure::persistence::{InMemoryCompanyRepository, InMemoryUserRepository};
use crate::infrastructure::shutdown::{wait_for_shutdown_signal, ShutdownSignal};
use crate::service::admin::AdminService;
use crate::service::chat::ChatService;
use crate::service::engine::MatchingEngine;
use crate::service::event_log::EventLogger;
use crate::service::indices::IndicesService;
use crate::service::leaderboard::LeaderboardService;
use crate::service::market::MarketService;
use crate::service::news::NewsService;
use crate::service::orders::OrdersService;
use crate::service::persistence::PersistenceService;
use crate::service::session::SessionManager;
use crate::service::token::TokenService;
use crate::service::trade_history::TradeHistoryService;

#[tokio::main]
async fn main() {
    // Initialize tracing with more verbose output
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,stockmart=debug")),
        )
        .init();

    info!("=== StockMart Backend Starting ===");

    // Read configuration from environment variables
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    info!("Configuration: PORT={}, DATA_DIR={}", port, data_dir);

    // Create shutdown signal for coordinating graceful shutdown
    let shutdown_signal = ShutdownSignal::new();

    // Initialize all services
    let (state, persistence_service) = initialize_services(&shutdown_signal, &data_dir).await;

    // Build the application router
    let app = build_router(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown(
            shutdown_signal.clone(),
            persistence_service,
        ))
        .await
        .expect("Server error");

    info!("=== StockMart Backend Shutdown Complete ===");
}

/// Initialize all application services.
async fn initialize_services(
    shutdown_signal: &ShutdownSignal,
    data_dir: &str,
) -> (Arc<AppState>, Arc<PersistenceService>) {
    // Initialize Config Service
    let config_service = Arc::new(ConfigService::new(data_dir.to_string()));
    info!("Config service initialized");

    // Initialize Session Manager
    let session_manager = Arc::new(SessionManager::new(config_service.max_sessions_per_user()));
    info!("Session manager initialized");

    // Initialize Token Service (with same max tokens as sessions)
    let token_service = Arc::new(TokenService::new(config_service.max_sessions_per_user()));
    info!("Token service initialized");

    // Initialize Repositories
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepository::new());
    let company_repo: Arc<dyn CompanyRepository> = Arc::new(InMemoryCompanyRepository::new());
    info!("Repositories initialized");

    // Initialize Orders Service (needed by engine)
    let orders_service = Arc::new(OrdersService::new());
    info!("Orders service initialized");

    // Initialize Trade History Service (needed by engine)
    let trade_history_service = Arc::new(TradeHistoryService::new());
    info!("Trade history service initialized");

    // Initialize Engine
    let engine = Arc::new(MatchingEngine::new(
        user_repo.clone(),
        orders_service.clone(),
        trade_history_service.clone(),
    ));
    info!("Matching engine initialized");

    // Initialize Market Service
    let market_service = Arc::new(MarketService::new());
    let market_clone = market_service.clone();
    let trade_rx = engine.subscribe_trades();
    let shutdown_clone = shutdown_signal.clone();

    tokio::spawn(async move {
        tokio::select! {
            _ = market_clone.run(trade_rx) => {}
            _ = shutdown_clone.wait() => {
                info!("Market service shutting down");
            }
        }
    });
    info!("Market service started");

    // Initialize Persistence Service
    let persistence_service = Arc::new(PersistenceService::new(
        user_repo.clone(),
        company_repo.clone(),
        data_dir.to_string(),
    ));

    // Load existing data
    persistence_service.load_data().await;
    info!("Data loaded from persistence");

    // Initialize ID generators from loaded data to avoid ID conflicts
    let max_user_id = user_repo
        .all()
        .await
        .map(|users| users.iter().map(|u| u.id).max().unwrap_or(0))
        .unwrap_or(0);
    let max_company_id = company_repo
        .all()
        .await
        .map(|companies| companies.iter().map(|c| c.id).max().unwrap_or(0))
        .unwrap_or(0);
    IdGenerators::init_from_persisted(max_user_id, max_company_id);

    // Spawn persistence task with shutdown support
    let persistence_clone = persistence_service.clone();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = persistence_clone.run() => {}
            _ = shutdown_clone.wait() => {
                info!("Persistence service shutting down");
            }
        }
    });

    // Initialize test data if empty (only if INIT_TEST_DATA=true)
    // This is disabled by default for production safety
    let init_test_data = std::env::var("INIT_TEST_DATA")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if init_test_data {
        info!("INIT_TEST_DATA=true: Initializing test data if empty");
        initialize_test_data(&user_repo, &company_repo, &engine).await;
    } else {
        // Just ensure orderbooks exist for companies
        if let Ok(companies) = company_repo.all().await {
            for company in companies {
                engine.create_orderbook(company.symbol.clone());
            }
            info!(
                "Restored orderbooks for {} companies",
                company_repo.all().await.map(|c| c.len()).unwrap_or(0)
            );
        }
    }

    // Initialize Indices Service
    let indices_service = Arc::new(IndicesService::new(
        market_service.clone(),
        company_repo.clone(),
    ));
    let indices_clone = indices_service.clone();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = indices_clone.run() => {}
            _ = shutdown_clone.wait() => {
                info!("Indices service shutting down");
            }
        }
    });

    // Initialize News Service (uses company repo for actual symbols)
    let news_service = Arc::new(NewsService::new(company_repo.clone()));
    let news_clone = news_service.clone();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = news_clone.run() => {}
            _ = shutdown_clone.wait() => {
                info!("News service shutting down");
            }
        }
    });

    // Initialize Leaderboard Service
    let leaderboard_service = Arc::new(LeaderboardService::new(
        user_repo.clone(),
        market_service.clone(),
    ));
    let leaderboard_clone = leaderboard_service.clone();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        tokio::select! {
            _ = leaderboard_clone.run() => {}
            _ = shutdown_clone.wait() => {
                info!("Leaderboard service shutting down");
            }
        }
    });

    // Initialize Admin Service
    let admin_service = Arc::new(AdminService::new(
        engine.clone(),
        company_repo.clone(),
        user_repo.clone(),
    ));

    // Initialize Chat Service
    let chat_service = Arc::new(ChatService::new());

    // Initialize Event Logger
    let event_logger = Arc::new(EventLogger::new(data_dir, false));
    info!("Event logger initialized");

    // Spawn trade logging task
    let trade_log_rx = engine.subscribe_trades();
    let event_logger_clone = event_logger.clone();
    let shutdown_clone = shutdown_signal.clone();
    tokio::spawn(async move {
        let mut rx = trade_log_rx;
        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(trade) => {
                            event_logger_clone.log_trade_executed(
                                trade.id,
                                &trade.symbol,
                                trade.taker_user_id,
                                trade.maker_user_id,
                                trade.qty,
                                trade.price,
                                trade.taker_order_id,
                                trade.maker_order_id,
                            );
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Trade logger lagged {} messages", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("Trade broadcast channel closed");
                            break;
                        }
                    }
                }
                _ = shutdown_clone.wait() => {
                    info!("Trade logging task shutting down");
                    break;
                }
            }
        }
    });
    info!("Trade logging task started");

    // Create shared application state
    let server_start_time = chrono::Utc::now().timestamp();
    let state = Arc::new(AppState {
        engine,
        market: market_service,
        admin: admin_service,
        indices: indices_service,
        news: news_service,
        leaderboard: leaderboard_service,
        chat: chat_service,
        user_repo,
        company_repo,
        config: config_service,
        sessions: session_manager,
        event_log: event_logger,
        orders: orders_service,
        trade_history: trade_history_service,
        tokens: token_service,
        server_start_time,
    });
    info!("Application state created");

    (state, persistence_service)
}

/// Initialize test data if repositories are empty.
async fn initialize_test_data(
    user_repo: &Arc<dyn UserRepository>,
    company_repo: &Arc<dyn CompanyRepository>,
    engine: &Arc<MatchingEngine>,
) {
    // Create test user if empty
    if user_repo.all().await.unwrap_or_default().is_empty() {
        let user = User::new(
            "REG123".to_string(),
            "Test Student".to_string(),
            "pass".to_string(),
        );
        let uid = user.id;
        if let Err(e) = user_repo.save(user).await {
            tracing::error!("Failed to create test user: {}", e);
        } else {
            info!("Created test user: ID={}, RegNo=REG123", uid);
        }
    }

    // Create test company if empty
    if company_repo.all().await.unwrap_or_default().is_empty() {
        let company = Company {
            id: crate::infrastructure::id_generator::IdGenerators::global().next_company_id(),
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            sector: "Tech".to_string(),
            total_shares: 1_000_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 10,
        };
        let symbol = company.symbol.clone();
        if let Err(e) = company_repo.save(company).await {
            tracing::error!("Failed to create test company: {}", e);
        } else {
            engine.create_orderbook(symbol.clone());
            info!("Created test company: {}", symbol);
        }
    } else {
        // Re-create orderbooks for existing companies
        if let Ok(companies) = company_repo.all().await {
            let count = companies.len();
            for company in companies {
                engine.create_orderbook(company.symbol.clone());
            }
            info!("Restored orderbooks for {} companies", count);
        }
    }
}

/// Build the application router.
fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { "StockMart Backend Running" }))
        .route("/ws", get(ws_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Graceful shutdown handler.
///
/// Waits for shutdown signal (Ctrl+C or SIGTERM), then:
/// 1. Triggers the shutdown signal to stop background tasks
/// 2. Saves all pending data to disk
/// 3. Allows the server to complete in-flight requests
async fn graceful_shutdown(shutdown_signal: ShutdownSignal, persistence: Arc<PersistenceService>) {
    // Wait for shutdown signal
    wait_for_shutdown_signal().await;

    info!("Initiating graceful shutdown...");

    // Trigger shutdown for all background tasks
    shutdown_signal.trigger();

    // Give background tasks a moment to notice the shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Save all data before shutting down
    info!("Saving data to disk...");
    persistence.save_data().await;
    info!("Data saved successfully");

    // Flush event log
    info!("Flushing event log...");
    // The event logger will flush when dropped

    info!("Graceful shutdown complete");
}
