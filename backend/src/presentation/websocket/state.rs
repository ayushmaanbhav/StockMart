//! WebSocket application state.
//!
//! Contains the shared application state for WebSocket connections.

use std::sync::Arc;

use crate::config::ConfigService;
use crate::domain::{CompanyRepository, UserRepository};
use crate::service::admin::AdminService;
use crate::service::chat::ChatService;
use crate::service::engine::MatchingEngine;
use crate::service::event_log::EventLogger;
use crate::service::indices::IndicesService;
use crate::service::leaderboard::LeaderboardService;
use crate::service::market::MarketService;
use crate::service::news::NewsService;
use crate::service::orders::OrdersService;
use crate::service::session::SessionManager;
use crate::service::token::TokenService;
use crate::service::trade_history::TradeHistoryService;

/// Shared application state for WebSocket connections.
///
/// This struct holds all the services and repositories needed
/// to handle WebSocket messages and maintain application state.
pub struct AppState {
    /// Matching engine for order processing
    pub engine: Arc<MatchingEngine>,
    /// Market data service (candles, circuit breakers)
    pub market: Arc<MarketService>,
    /// Admin service for administrative actions
    pub admin: Arc<AdminService>,
    /// Market indices service
    pub indices: Arc<IndicesService>,
    /// News service
    pub news: Arc<NewsService>,
    /// Leaderboard service
    pub leaderboard: Arc<LeaderboardService>,
    /// Chat service
    pub chat: Arc<ChatService>,
    /// User repository
    pub user_repo: Arc<dyn UserRepository>,
    /// Company repository
    pub company_repo: Arc<dyn CompanyRepository>,
    /// Configuration service
    pub config: Arc<ConfigService>,
    /// Session manager
    pub sessions: Arc<SessionManager>,
    /// Event logger
    pub event_log: Arc<EventLogger>,
    /// Orders service
    pub orders: Arc<OrdersService>,
    /// Trade history service
    pub trade_history: Arc<TradeHistoryService>,
    /// Secure token service for authentication
    pub tokens: Arc<TokenService>,
    /// Server start time (Unix timestamp)
    pub server_start_time: i64,
}
