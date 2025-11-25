use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};

mod domain;
mod repository;
mod service {
    pub mod engine;
    pub mod market;
    pub mod admin;
    pub mod indices;
    pub mod news;
    pub mod leaderboard;
    pub mod persistence;
    pub mod chat;
}
mod api;
mod config;

use crate::repository::{UserRepository, CompanyRepository};
use crate::repository::memory::{InMemoryUserRepository, InMemoryCompanyRepository};
use crate::service::engine::MatchingEngine;
use crate::service::market::MarketService;
use crate::service::admin::AdminService;
use crate::service::indices::IndicesService;
use crate::service::news::NewsService;
use crate::service::leaderboard::LeaderboardService;
use crate::service::persistence::PersistenceService;
use crate::service::chat::ChatService;
use crate::api::ws::{ws_handler, AppState};
use crate::domain::models::{User, Company};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize Repositories
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let company_repo = Arc::new(InMemoryCompanyRepository::new());

    // Initialize Engine
    let engine = Arc::new(MatchingEngine::new(user_repo.clone()));

    // Initialize Market Service
    let market_service = Arc::new(MarketService::new());
    let market_clone = market_service.clone();
    let trade_rx = engine.subscribe_trades();
    
    tokio::spawn(async move {
        market_clone.run(trade_rx).await;
    });

    // Initialize Persistence Service
    let persistence_service = Arc::new(PersistenceService::new(
        user_repo.clone(),
        company_repo.clone(),
        "./data".to_string(),
    ));
    
    // Load existing data
    persistence_service.load_data().await;

    // Spawn persistence task
    let persistence_clone = persistence_service.clone();
    tokio::spawn(async move {
        persistence_clone.run().await;
    });

    // Initialize Data (Seed) ONLY if empty
    if user_repo.all().await.unwrap().is_empty() {
        let user = User::new("REG123".to_string(), "Test Student".to_string(), "pass".to_string());
        let uid = user.id;
        user_repo.save(user).await.unwrap();
        tracing::info!("Created Test User: ID={}, RegNo=REG123", uid);
    }

    if company_repo.all().await.unwrap().is_empty() {
        // Create a test company
        let company = Company {
            id: crate::domain::models::next_company_id(),
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            sector: "Tech".to_string(),
            total_shares: 1_000_000,
            bankrupt: false,
            price_precision: 2,
            volatility: 10,
        };
        company_repo.save(company.clone()).await.unwrap();
        engine.create_orderbook(company.symbol.clone());
        tracing::info!("Created Test Company: {}", company.symbol);
    } else {
        // Re-create orderbooks for existing companies
        if let Ok(companies) = company_repo.all().await {
            for company in companies {
                engine.create_orderbook(company.symbol.clone());
            }
        }
    }

    // Initialize Indices Service
    let indices_service = Arc::new(IndicesService::new(market_service.clone(), company_repo.clone()));
    let indices_clone = indices_service.clone();
    tokio::spawn(async move {
        indices_clone.run().await;
    });

    // Initialize News Service
    let news_service = Arc::new(NewsService::new());
    let news_clone = news_service.clone();
    tokio::spawn(async move {
        news_clone.run().await;
    });

    // Initialize Leaderboard Service
    let leaderboard_service = Arc::new(LeaderboardService::new(user_repo.clone(), market_service.clone()));
    let leaderboard_clone = leaderboard_service.clone();
    tokio::spawn(async move {
        leaderboard_clone.run().await;
    });

    // Initialize Admin Service
    let admin_service = Arc::new(AdminService::new(
        engine.clone(),
        company_repo.clone(),
    ));

    // Initialize Chat Service
    let chat_service = Arc::new(ChatService::new());

    // Shared State
    let state = Arc::new(AppState {
        engine,
        market: market_service,
        admin: admin_service,
        indices: indices_service,
        news: news_service,
        leaderboard: leaderboard_service,
        chat: chat_service,
        user_repo,
    });

    // Build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "StockMart Backend Running" }))
        .route("/ws", get(ws_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
