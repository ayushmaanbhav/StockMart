use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use crate::domain::models::{Order, OrderSide, OrderType, TimeInForce, next_order_id, User};
use crate::service::engine::MatchingEngine;
use crate::service::market::MarketService;
use crate::service::admin::AdminService;
use crate::service::indices::IndicesService;
use crate::service::news::NewsService;
use crate::service::leaderboard::LeaderboardService;
use crate::service::chat::ChatService;
use crate::service::session::SessionManager;
use crate::config::ConfigService;
use crate::repository::{UserRepository, CompanyRepository};
use crate::domain::models::ChatMessage;
use crate::service::event_log::EventLogger;
use crate::service::orders::OrdersService;
use crate::service::trade_history::TradeHistoryService;
use crate::domain::ui_models::{
    self, AdminDashboardMetrics, AdminOpenOrderUI, AdminTradeHistoryItem, CandleUI, CompanyUI,
    FullStateSyncPayload, LeaderboardEntryUI, MarketIndexUI, NewsItemUI, OpenOrderUI,
    OrderbookLevelUI, OrderbookUI, PortfolioItemUI, PortfolioStateUI, TradeHistoryItem,
    TradeHistoryResponse, next_sync_id,
};

// Shared state
pub struct AppState {
    pub engine: Arc<MatchingEngine>,
    pub market: Arc<MarketService>,
    pub admin: Arc<AdminService>,
    pub indices: Arc<IndicesService>,
    pub news: Arc<NewsService>,
    pub leaderboard: Arc<LeaderboardService>,
    pub chat: Arc<ChatService>,
    pub user_repo: Arc<dyn UserRepository>,
    pub company_repo: Arc<dyn CompanyRepository>,
    pub config: Arc<ConfigService>,
    pub sessions: Arc<SessionManager>,
    pub event_log: Arc<EventLogger>,
    pub orders: Arc<OrdersService>,
    pub trade_history: Arc<TradeHistoryService>,
}

/// Messages from client to server
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    /// Authenticate with user ID token (for reconnection/stored sessions)
    Auth { token: String },

    /// Login with registration number and password
    Login { regno: String, password: String },

    /// Register a new user
    Register { regno: String, name: String, password: String },
    
    /// Place a trading order
    PlaceOrder {
        symbol: String,
        side: String,           // "Buy", "Sell", "Short"
        order_type: String,     // "Market", "Limit"
        time_in_force: Option<String>,  // "GTC", "IOC" (default: GTC)
        qty: u64,
        price: i64,
    },
    
    /// Cancel an existing order
    CancelOrder { symbol: String, order_id: u64 },
    
    /// Subscribe to market data for a symbol
    Subscribe { symbol: String },
    
    /// Request order book depth
    GetDepth { symbol: String, levels: Option<usize> },
    
    /// Admin actions
    AdminAction { action: String, payload: serde_json::Value },
    
    /// Send chat message
    Chat { message: String },
    
    /// Request current portfolio
    GetPortfolio,
    
    /// Ping for keepalive
    Ping {},

    /// Get public config (registration mode, etc.)
    GetConfig {},

    /// Request full state sync (on connect/reconnect)
    RequestSync {
        /// Optional component name for partial sync, None for full sync
        component: Option<String>,
    },

    /// Get user's trade history
    GetTradeHistory {
        page: Option<u32>,
        page_size: Option<u32>,
        symbol: Option<String>,  // Filter by symbol
    },

    /// Get stock trade history (for orderbook tab)
    GetStockTrades {
        symbol: String,
        count: Option<usize>,
    },
}

/// Messages from server to client
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    /// Authentication successful
    AuthSuccess { user_id: u64, name: String },
    
    /// Authentication failed
    AuthFailed { reason: String },
    
    /// Registration successful
    RegisterSuccess { user_id: u64, name: String },
    
    /// Registration failed
    RegisterFailed { reason: String },
    
    /// Order acknowledged
    OrderAck { 
        order_id: u64, 
        status: String,
        filled_qty: u64,
        remaining_qty: u64,
    },
    
    /// Order rejected
    OrderRejected { 
        reason: String,
        error_code: String,  // For frontend error handling
    },
    
    /// Order cancelled
    OrderCancelled { order_id: u64 },
    
    /// Trade executed
    TradeUpdate { 
        symbol: String, 
        price: i64, 
        qty: u64,
        timestamp: i64,
    },
    
    /// Candlestick update
    CandleUpdate { symbol: String, candle: crate::domain::models::Candle },
    
    /// Order book depth
    DepthUpdate {
        symbol: String,
        bids: Vec<(i64, u64)>,  // (price, qty)
        asks: Vec<(i64, u64)>,
        spread: Option<i64>,
    },
    
    /// Market index update
    IndexUpdate { name: String, value: i64 },
    
    /// Portfolio update
    PortfolioUpdate { 
        money: i64, 
        locked: i64,
        margin_locked: i64,
        net_worth: i64,  // Calculated total value
        items: Vec<crate::domain::models::Portfolio>,
    },
    
    /// Circuit breaker triggered
    CircuitBreaker { symbol: String, halted_until: i64, reason: String },
    
    /// Market status changed
    MarketStatus { is_open: bool },
    
    /// News item
    NewsUpdate { news: crate::service::news::NewsItem },
    
    /// Leaderboard update
    LeaderboardUpdate { entries: Vec<crate::service::leaderboard::LeaderboardEntry> },
    
    /// Chat message
    ChatUpdate { message: ChatMessage },
    
    /// General error
    Error { code: String, message: String },
    
    /// Pong response
    Pong { timestamp: i64 },
    
    /// System announcement
    System { message: String },

    /// List of all tradeable companies
    CompanyList { companies: Vec<CompanyInfo> },

    /// Public config for initialization
    Config {
        registration_mode: String,
        chat_enabled: bool,
        currency: CurrencyConfigPayload,
    },

    /// Session kicked (another session took over)
    SessionKicked { reason: String },

    // ==================== NEW UI-READY MESSAGE TYPES ====================

    /// Full state sync - sent on connect/reconnect
    FullStateSync {
        payload: FullStateSyncPayload,
    },

    /// Enhanced portfolio with pre-computed values
    PortfolioUpdateUI {
        money: i64,
        locked_money: i64,
        margin_locked: i64,
        portfolio_value: i64,
        net_worth: i64,
        items: Vec<PortfolioItemUI>,
    },

    /// Open orders list update
    OpenOrdersUpdate {
        orders: Vec<OpenOrderUI>,
    },

    /// UI-ready index update with change data
    IndexUpdateUI {
        index: MarketIndexUI,
    },

    /// UI-ready leaderboard update
    LeaderboardUpdateUI {
        entries: Vec<LeaderboardEntryUI>,
    },

    /// Trade history response (for user's trades)
    TradeHistory {
        trades: Vec<TradeHistoryItem>,
        total_count: u64,
        page: u32,
        page_size: u32,
        has_more: bool,
    },

    /// Stock trade history response (for orderbook tab)
    StockTradeHistory {
        symbol: String,
        trades: Vec<TradeHistoryItem>,
    },

    /// Admin: All trades with filters (enhanced with both parties)
    AdminTradeHistory {
        trades: Vec<AdminTradeHistoryItem>,
        total_count: u64,
        page: u32,
        page_size: u32,
        has_more: bool,
    },

    /// Admin: All open orders (enhanced with user info)
    AdminOpenOrders {
        orders: Vec<AdminOpenOrderUI>,
        total_count: usize,
    },

    /// Admin: Dashboard metrics
    AdminDashboardMetrics {
        metrics: AdminDashboardMetrics,
    },

    /// Admin: Orderbook view with individual orders
    AdminOrderbook {
        symbol: String,
        bids: Vec<AdminOpenOrderUI>,
        asks: Vec<AdminOpenOrderUI>,
    },

    // === Component Sync Responses ===

    /// Portfolio sync response
    PortfolioSync {
        sync_id: u64,
        money: i64,
        locked_money: i64,
        margin_locked: i64,
        portfolio_value: i64,
        net_worth: i64,
        items: Vec<PortfolioItemUI>,
    },

    /// Open orders sync response
    OpenOrdersSync {
        sync_id: u64,
        orders: Vec<OpenOrderUI>,
    },

    /// Leaderboard sync response
    LeaderboardSync {
        sync_id: u64,
        entries: Vec<LeaderboardEntryUI>,
    },

    /// Indices sync response
    IndicesSync {
        sync_id: u64,
        indices: Vec<MarketIndexUI>,
    },

    /// Orderbook sync response
    OrderbookSync {
        sync_id: u64,
        symbol: String,
        orderbook: OrderbookUI,
    },

    /// Candles sync response
    CandlesSync {
        sync_id: u64,
        symbol: String,
        candles: Vec<CandleUI>,
    },

    /// News sync response
    NewsSync {
        sync_id: u64,
        news: Vec<NewsItemUI>,
    },

    /// Chat sync response
    ChatSync {
        sync_id: u64,
        messages: Vec<ChatMessage>,
    },
}

/// Company info for the company list
#[derive(Debug, Serialize, Clone)]
pub struct CompanyInfo {
    pub id: u64,
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub volatility: i64,
}

/// Currency configuration payload for client
#[derive(Debug, Serialize, Clone)]
pub struct CurrencyConfigPayload {
    pub symbol: String,
    pub code: String,
    pub locale: String,
    pub decimals: u8,
    pub symbol_position: String,
}

impl From<&crate::config::CurrencyConfig> for CurrencyConfigPayload {
    fn from(config: &crate::config::CurrencyConfig) -> Self {
        Self {
            symbol: config.symbol.clone(),
            code: config.code.clone(),
            locale: config.locale.clone(),
            decimals: config.decimals,
            symbol_position: config.symbol_position.clone(),
        }
    }
}

impl ServerMessage {
    fn error(code: &str, message: &str) -> Self {
        ServerMessage::Error { 
            code: code.to_string(), 
            message: message.to_string() 
        }
    }
}

/// Helper to send a message to the client
async fn send_message<T: Serialize>(sender: &mut futures::stream::SplitSink<WebSocket, Message>, msg: &T) {
    if let Ok(json) = serde_json::to_string(msg) {
        let _ = sender.send(Message::Text(json)).await;
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    info!("New WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();
    let mut user_id: Option<u64> = None;
    let mut session_id: Option<u64> = None;
    let mut subscribed_symbols: Vec<String> = Vec::new();

    // Send initial config to the client
    let public_config = state.config.get_public_config();
    let config_msg = ServerMessage::Config {
        registration_mode: format!("{:?}", public_config.registration_mode),
        chat_enabled: public_config.chat_enabled,
        currency: CurrencyConfigPayload::from(&public_config.currency),
    };
    send_message(&mut sender, &config_msg).await;
    debug!("Sent initial config to client");

    // Subscribe to broadcast channels
    let mut trade_rx = state.engine.subscribe_trades();
    let mut candle_rx = state.market.subscribe_candles();
    let mut cb_rx = state.market.subscribe_circuit_breakers();
    let mut index_rx = state.indices.subscribe_indices();
    let mut news_rx = state.news.subscribe();
    let mut lb_rx = state.leaderboard.subscribe();
    let mut chat_rx = state.chat.subscribe();

    loop {
        tokio::select! {
            // Handle incoming client messages
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = handle_client_message(
                            &text,
                            &mut sender,
                            &state,
                            &mut user_id,
                            &mut session_id,
                            &mut subscribed_symbols,
                        ).await {
                            error!("Error handling client message: {}", e);
                        }
                    },
                    Ok(Message::Ping(data)) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    },
                    Ok(Message::Close(_)) => {
                        info!("Client disconnected (user_id={:?}, session_id={:?})", user_id, session_id);
                        break;
                    },
                    Err(e) => {
                        error!("WebSocket error: {} (user_id={:?})", e, user_id);
                        break;
                    },
                    _ => {}
                }
            }
            
            // Handle trade updates
            Ok(trade) = trade_rx.recv() => {
                let msg = ServerMessage::TradeUpdate {
                    symbol: trade.symbol.clone(),
                    price: trade.price,
                    qty: trade.qty,
                    timestamp: trade.timestamp,
                };
                send_message(&mut sender, &msg).await;

                // Send updated depth if user is subscribed to this symbol
                if subscribed_symbols.contains(&trade.symbol) {
                    if let Some((bids, asks)) = state.engine.get_order_book_depth(&trade.symbol, 10) {
                        let spread = match (bids.first(), asks.first()) {
                            (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                            _ => None,
                        };
                        let depth_msg = ServerMessage::DepthUpdate {
                            symbol: trade.symbol.clone(),
                            bids,
                            asks,
                            spread,
                        };
                        send_message(&mut sender, &depth_msg).await;
                    }
                }

                // If this user was involved, send portfolio update
                if let Some(uid) = user_id {
                    if trade.maker_user_id == uid || trade.taker_user_id == uid {
                        if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                            let net_worth = calculate_net_worth(&user, &state.market);
                            let msg = ServerMessage::PortfolioUpdate {
                                money: user.money,
                                locked: user.locked_money,
                                margin_locked: user.margin_locked,
                                net_worth,
                                items: user.portfolio,
                            };
                            send_message(&mut sender, &msg).await;
                        }
                    }
                }
            }
            
            // Handle candle updates
            Ok(candle) = candle_rx.recv() => {
                let msg = ServerMessage::CandleUpdate { 
                    symbol: candle.symbol.clone(), 
                    candle 
                };
                send_message(&mut sender, &msg).await;
            }
            
            // Handle circuit breaker updates
            Ok((symbol, halted_until)) = cb_rx.recv() => {
                let msg = ServerMessage::CircuitBreaker { 
                    symbol, 
                    halted_until,
                    reason: "10% price movement threshold exceeded".to_string(),
                };
                send_message(&mut sender, &msg).await;
            }
            
            // Handle index updates (now using UI-ready data)
            Ok(index) = index_rx.recv() => {
                // Send both old format (for backward compatibility) and new UI format
                let msg = ServerMessage::IndexUpdate {
                    name: index.name.clone(),
                    value: index.value
                };
                send_message(&mut sender, &msg).await;

                // Also send UI-ready format
                let ui_msg = ServerMessage::IndexUpdateUI { index };
                send_message(&mut sender, &ui_msg).await;
            }
            
            // Handle news updates
            Ok(news) = news_rx.recv() => {
                let msg = ServerMessage::NewsUpdate { news };
                send_message(&mut sender, &msg).await;
            }
            
            // Handle leaderboard updates (now using UI-ready data with correct net worth)
            Ok(entries) = lb_rx.recv() => {
                // Send UI-ready format with user_id and change_rank
                let msg = ServerMessage::LeaderboardUpdateUI { entries };
                send_message(&mut sender, &msg).await;
            }
            
            // Handle chat messages
            Ok(message) = chat_rx.recv() => {
                let msg = ServerMessage::ChatUpdate { message };
                send_message(&mut sender, &msg).await;
            }

            else => break,
        }
    }

    // Cleanup session on disconnect
    if let Some(sid) = session_id {
        state.sessions.remove_session(sid);
        info!("Session {} cleaned up for user {:?}", sid, user_id);
    }
}

async fn handle_client_message(
    text: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    subscribed_symbols: &mut Vec<String>,
) -> Result<(), String> {
    let client_msg: ClientMessage = serde_json::from_str(text)
        .map_err(|e| {
            let msg = ServerMessage::error("PARSE_ERROR", &format!("Invalid JSON: {}", e));
            let _ = futures::executor::block_on(send_message(sender, &msg));
            e.to_string()
        })?;

    debug!("Handling client message: {:?}", std::mem::discriminant(&client_msg));

    match client_msg {
        ClientMessage::Auth { token } => {
            handle_auth(sender, state, user_id, session_id, &token).await;
        }

        ClientMessage::Login { regno, password } => {
            handle_login(sender, state, user_id, session_id, regno, password).await;
        }

        ClientMessage::Register { regno, name, password } => {
            handle_register(sender, state, user_id, session_id, regno, name, password).await;
        }
        
        ClientMessage::PlaceOrder { symbol, side, order_type, time_in_force, qty, price } => {
            handle_place_order(sender, state, *user_id, symbol, side, order_type, time_in_force, qty, price).await;
        }
        
        ClientMessage::CancelOrder { symbol, order_id } => {
            handle_cancel_order(sender, state, *user_id, &symbol, order_id).await;
        }
        
        ClientMessage::Subscribe { symbol } => {
            handle_subscribe(sender, state, subscribed_symbols, &symbol).await;
        }
        
        ClientMessage::GetDepth { symbol, levels } => {
            handle_get_depth(sender, state, &symbol, levels.unwrap_or(10)).await;
        }
        
        ClientMessage::AdminAction { action, payload } => {
            handle_admin_action(sender, state, *user_id, &action, payload).await;
        }
        
        ClientMessage::Chat { message } => {
            handle_chat(sender, state, *user_id, message).await;
        }
        
        ClientMessage::GetPortfolio => {
            handle_get_portfolio(sender, state, *user_id).await;
        }
        
        ClientMessage::Ping {} => {
            let msg = ServerMessage::Pong { timestamp: chrono::Utc::now().timestamp() };
            send_message(sender, &msg).await;
        }

        ClientMessage::GetConfig {} => {
            let public_config = state.config.get_public_config();
            let config_msg = ServerMessage::Config {
                registration_mode: format!("{:?}", public_config.registration_mode),
                chat_enabled: public_config.chat_enabled,
                currency: CurrencyConfigPayload::from(&public_config.currency),
            };
            send_message(sender, &config_msg).await;
            debug!("Sent config response");
        }

        ClientMessage::RequestSync { component } => {
            handle_request_sync(sender, state, *user_id, subscribed_symbols, component).await;
        }

        ClientMessage::GetTradeHistory { page, page_size, symbol } => {
            handle_get_trade_history(sender, state, *user_id, page, page_size, symbol).await;
        }

        ClientMessage::GetStockTrades { symbol, count } => {
            handle_get_stock_trades(sender, state, &symbol, count).await;
        }
    }

    Ok(())
}

async fn handle_auth(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    token: &str,
) {
    debug!("Auth attempt with token: {}", token);

    match token.parse::<u64>() {
        Ok(uid) => {
            match state.user_repo.find_by_id(uid).await {
                Ok(Some(user)) => {
                    if user.banned {
                        warn!("Auth failed: user {} is banned", uid);
                        let msg = ServerMessage::AuthFailed { reason: "Account has been banned".to_string() };
                        send_message(sender, &msg).await;
                        return;
                    }

                    // Create session (kicks old sessions if max reached)
                    let (sid, kicked) = state.sessions.create_session(uid);
                    *session_id = Some(sid);
                    *user_id = Some(uid);

                    if !kicked.is_empty() {
                        info!("User {} authenticated, kicked {} old session(s)", uid, kicked.len());
                    } else {
                        info!("User {} authenticated with session {}", uid, sid);
                    }

                    let auth_msg = ServerMessage::AuthSuccess {
                        user_id: uid,
                        name: user.name.clone()
                    };
                    send_message(sender, &auth_msg).await;

                    // Send company list
                    if let Ok(companies) = state.company_repo.all().await {
                        let company_list: Vec<CompanyInfo> = companies.iter().map(|c| CompanyInfo {
                            id: c.id,
                            symbol: c.symbol.clone(),
                            name: c.name.clone(),
                            sector: c.sector.clone(),
                            volatility: c.volatility,
                        }).collect();
                        let companies_msg = ServerMessage::CompanyList { companies: company_list };
                        send_message(sender, &companies_msg).await;
                    }

                    // Send initial portfolio
                    let net_worth = calculate_net_worth(&user, &state.market);
                    let portfolio_msg = ServerMessage::PortfolioUpdate {
                        money: user.money,
                        locked: user.locked_money,
                        margin_locked: user.margin_locked,
                        net_worth,
                        items: user.portfolio.clone(),
                    };
                    send_message(sender, &portfolio_msg).await;

                    // Send market status
                    let status_msg = ServerMessage::MarketStatus {
                        is_open: state.engine.is_market_open()
                    };
                    send_message(sender, &status_msg).await;
                }
                Ok(None) => {
                    warn!("Auth failed: user {} not found", uid);
                    let msg = ServerMessage::AuthFailed { reason: "User not found".to_string() };
                    send_message(sender, &msg).await;
                }
                Err(e) => {
                    error!("Auth error for uid {}: {}", uid, e);
                    let msg = ServerMessage::AuthFailed { reason: format!("Auth error: {}", e) };
                    send_message(sender, &msg).await;
                }
            }
        }
        Err(_) => {
            warn!("Auth failed: invalid token format");
            let msg = ServerMessage::AuthFailed { reason: "Invalid token format".to_string() };
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_login(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    regno: String,
    password: String,
) {
    debug!("Login attempt for regno: {}", regno);

    // Find user by registration number
    match state.user_repo.find_by_regno(&regno).await {
        Ok(Some(user)) => {
            // Verify password (simple comparison - in production use proper hashing)
            if user.password_hash != password {
                warn!("Login failed for {}: invalid password", regno);
                let msg = ServerMessage::AuthFailed { reason: "Invalid password".to_string() };
                send_message(sender, &msg).await;
                return;
            }

            if user.banned {
                warn!("Login failed for {}: account banned", regno);
                let msg = ServerMessage::AuthFailed { reason: "Account has been banned".to_string() };
                send_message(sender, &msg).await;
                return;
            }

            // Create session (kicks old sessions if max reached)
            let (sid, kicked) = state.sessions.create_session(user.id);
            *session_id = Some(sid);
            *user_id = Some(user.id);

            if !kicked.is_empty() {
                info!("User {} (regno={}) logged in, kicked {} old session(s)", user.name, regno, kicked.len());
            } else {
                info!("User {} (regno={}) logged in with session {}", user.name, regno, sid);
            }

            // Log login event
            state.event_log.log_user_login(user.id, &regno, &user.name);

            let auth_msg = ServerMessage::AuthSuccess {
                user_id: user.id,
                name: user.name.clone()
            };
            send_message(sender, &auth_msg).await;

            // Send company list
            if let Ok(companies) = state.company_repo.all().await {
                let company_list: Vec<CompanyInfo> = companies.iter().map(|c| CompanyInfo {
                    id: c.id,
                    symbol: c.symbol.clone(),
                    name: c.name.clone(),
                    sector: c.sector.clone(),
                    volatility: c.volatility,
                }).collect();
                let companies_msg = ServerMessage::CompanyList { companies: company_list };
                send_message(sender, &companies_msg).await;
            }

            // Send initial portfolio
            let net_worth = calculate_net_worth(&user, &state.market);
            let portfolio_msg = ServerMessage::PortfolioUpdate {
                money: user.money,
                locked: user.locked_money,
                margin_locked: user.margin_locked,
                net_worth,
                items: user.portfolio.clone(),
            };
            send_message(sender, &portfolio_msg).await;

            // Send market status
            let status_msg = ServerMessage::MarketStatus {
                is_open: state.engine.is_market_open()
            };
            send_message(sender, &status_msg).await;
        }
        Ok(None) => {
            warn!("Login failed: regno {} not found", regno);
            let msg = ServerMessage::AuthFailed { reason: "User not found. Please register first.".to_string() };
            send_message(sender, &msg).await;
        }
        Err(e) => {
            error!("Login error for regno {}: {}", regno, e);
            let msg = ServerMessage::AuthFailed { reason: format!("Login error: {}", e) };
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_register(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    regno: String,
    name: String,
    password: String,
) {
    debug!("Registration attempt for regno: {}, name: {}", regno, name);

    // Check if registration is allowed for this regno
    if let Err(reason) = state.config.is_regno_allowed(&regno) {
        warn!("Registration rejected for regno {}: {}", regno, reason);
        let msg = ServerMessage::RegisterFailed { reason };
        send_message(sender, &msg).await;
        return;
    }

    // Check if regno already exists
    match state.user_repo.find_by_regno(&regno).await {
        Ok(Some(_)) => {
            warn!("Registration failed: regno {} already exists", regno);
            let msg = ServerMessage::RegisterFailed {
                reason: "Registration number already exists".to_string()
            };
            send_message(sender, &msg).await;
            return;
        }
        Err(e) => {
            error!("Registration error for regno {}: {}", regno, e);
            let msg = ServerMessage::RegisterFailed {
                reason: format!("Registration error: {}", e)
            };
            send_message(sender, &msg).await;
            return;
        }
        Ok(None) => {}
    }

    // Get companies for initial share allocation
    let companies = match state.company_repo.all().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to fetch companies for registration: {}", e);
            let msg = ServerMessage::RegisterFailed {
                reason: "Failed to initialize portfolio".to_string()
            };
            send_message(sender, &msg).await;
            return;
        }
    };

    // Create new user with configured starting money
    let mut new_user = User::new(regno.clone(), name.clone(), password);
    let total_starting_value = state.config.default_starting_money(); // Total net worth (e.g., $100,000)

    // Allocate ~50% as shares, ~50% as cash
    // Base price is $100 per share (100 * PRICE_SCALE in internal format)
    let base_price: i64 = 100 * crate::domain::models::PRICE_SCALE;
    let target_portfolio_value = total_starting_value / 2; // Half for shares

    // Calculate shares per company to reach target portfolio value
    let num_companies = companies.len() as i64;
    if num_companies > 0 {
        let value_per_company = target_portfolio_value / num_companies;
        let shares_per_company = (value_per_company / base_price) as u64;

        // Add some randomness (-20% to +20%) to each allocation
        let mut rng = rand::thread_rng();
        let mut total_portfolio_value: i64 = 0;

        for company in &companies {
            // Random variance between -20% and +20%
            let variance: i64 = rng.gen_range(-20..=20);
            let adjusted_shares = ((shares_per_company as i64 * (100 + variance)) / 100) as u64;
            let final_shares = adjusted_shares.max(1); // At least 1 share

            let share_value = (final_shares as i64) * base_price;
            total_portfolio_value += share_value;

            new_user.portfolio.push(crate::domain::models::Portfolio {
                user_id: new_user.id,
                symbol: company.symbol.clone(),
                qty: final_shares,
                short_qty: 0,
                locked_qty: 0,
                average_buy_price: base_price,
            });

            debug!("  {} allocated {} shares = ${}",
                   company.symbol, final_shares, share_value / crate::domain::models::PRICE_SCALE);
        }

        // Calculate cash to reach target net worth
        // Net worth = cash + portfolio_value, so cash = target - portfolio_value
        new_user.money = (total_starting_value - total_portfolio_value).max(0);

        let actual_networth = new_user.money + total_portfolio_value;
        info!("New trader {} allocated: cash=${}, portfolio=${}, networth=${}",
              name,
              new_user.money / crate::domain::models::PRICE_SCALE,
              total_portfolio_value / crate::domain::models::PRICE_SCALE,
              actual_networth / crate::domain::models::PRICE_SCALE);
    } else {
        // No companies, just give all as cash
        new_user.money = total_starting_value;
    }

    let new_user_id = new_user.id;

    match state.user_repo.save(new_user.clone()).await {
        Ok(_) => {
            // Create session
            let (sid, _) = state.sessions.create_session(new_user_id);
            *session_id = Some(sid);
            *user_id = Some(new_user_id);

            info!("New user registered: {} (regno={}, id={}, session={})",
                  name, regno, new_user_id, sid);

            // Log registration event
            let portfolio_value = new_user.portfolio.iter()
                .map(|p| (p.qty as i64) * p.average_buy_price)
                .sum::<i64>();
            state.event_log.log_user_registered(
                new_user_id,
                &regno,
                &name,
                new_user.money,
                portfolio_value,
            );

            let msg = ServerMessage::RegisterSuccess {
                user_id: new_user_id,
                name: name.clone()
            };
            send_message(sender, &msg).await;

            // Send company list
            let company_list: Vec<CompanyInfo> = companies.iter().map(|c| CompanyInfo {
                id: c.id,
                symbol: c.symbol.clone(),
                name: c.name.clone(),
                sector: c.sector.clone(),
                volatility: c.volatility,
            }).collect();
            let companies_msg = ServerMessage::CompanyList { companies: company_list };
            send_message(sender, &companies_msg).await;

            // Send initial portfolio with shares
            let net_worth = calculate_net_worth(&new_user, &state.market);
            let portfolio_msg = ServerMessage::PortfolioUpdate {
                money: new_user.money,
                locked: 0,
                margin_locked: 0,
                net_worth,
                items: new_user.portfolio.clone(),
            };
            send_message(sender, &portfolio_msg).await;

            // Send market status
            let status_msg = ServerMessage::MarketStatus {
                is_open: state.engine.is_market_open()
            };
            send_message(sender, &status_msg).await;

            // Broadcast welcome message
            let portfolio_value = new_user.portfolio.iter()
                .map(|p| (p.qty as i64) * p.average_buy_price)
                .sum::<i64>() / crate::domain::models::PRICE_SCALE;
            let starting_cash = new_user.money / crate::domain::models::PRICE_SCALE;
            let system_msg = ServerMessage::System {
                message: format!("Welcome {}! You start with ${} in cash and ${} in stocks.", name, starting_cash, portfolio_value)
            };
            send_message(sender, &system_msg).await;
        }
        Err(e) => {
            error!("Failed to save user {}: {}", regno, e);
            let msg = ServerMessage::RegisterFailed {
                reason: format!("Failed to save user: {}", e)
            };
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_place_order(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    symbol: String,
    side: String,
    order_type: String,
    time_in_force: Option<String>,
    qty: u64,
    price: i64,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::error("NOT_AUTHENTICATED", "Please authenticate first");
            send_message(sender, &msg).await;
            return;
        }
    };

    // Parse order side
    let side_enum = match side.as_str() {
        "Buy" => OrderSide::Buy,
        "Sell" => OrderSide::Sell,
        "Short" => OrderSide::Short,
        _ => {
            let msg = ServerMessage::OrderRejected { 
                reason: format!("Invalid order side: {}", side),
                error_code: "INVALID_SIDE".to_string(),
            };
            send_message(sender, &msg).await;
            return;
        }
    };

    // Parse order type
    let type_enum = match order_type.as_str() {
        "Market" => OrderType::Market,
        "Limit" => OrderType::Limit,
        _ => {
            let msg = ServerMessage::OrderRejected { 
                reason: format!("Invalid order type: {}", order_type),
                error_code: "INVALID_TYPE".to_string(),
            };
            send_message(sender, &msg).await;
            return;
        }
    };

    // Parse time in force
    let tif_enum = match time_in_force.as_deref() {
        Some("IOC") => TimeInForce::IOC,
        Some("GTC") | None => TimeInForce::GTC,
        Some(other) => {
            let msg = ServerMessage::OrderRejected { 
                reason: format!("Invalid time in force: {}", other),
                error_code: "INVALID_TIF".to_string(),
            };
            send_message(sender, &msg).await;
            return;
        }
    };

    // Validate quantity
    if qty == 0 {
        let msg = ServerMessage::OrderRejected { 
            reason: "Quantity must be greater than 0".to_string(),
            error_code: "INVALID_QTY".to_string(),
        };
        send_message(sender, &msg).await;
        return;
    }

    // Validate price for limit orders
    if type_enum == OrderType::Limit && price <= 0 {
        let msg = ServerMessage::OrderRejected { 
            reason: "Limit order price must be greater than 0".to_string(),
            error_code: "INVALID_PRICE".to_string(),
        };
        send_message(sender, &msg).await;
        return;
    }

    let order = Order {
        id: next_order_id(),
        user_id: uid,
        symbol: symbol.clone(),
        order_type: type_enum,
        side: side_enum,
        qty,
        filled_qty: 0,
        price,
        status: crate::domain::models::OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: tif_enum,
    };

    tracing::info!("Placing order: {:?}", order);

    // Log order placed event
    state.event_log.log_order_placed(
        order.id,
        uid,
        &symbol,
        &side,
        &order_type,
        qty,
        price,
        time_in_force.as_deref().unwrap_or("GTC"),
    );

    match state.engine.place_order(order).await {
        Ok(processed) => {
            tracing::info!("Order {} processed: {:?}", processed.id, processed.status);
            let msg = ServerMessage::OrderAck {
                order_id: processed.id,
                status: format!("{:?}", processed.status),
                filled_qty: processed.filled_qty,
                remaining_qty: processed.qty - processed.filled_qty,
            };
            send_message(sender, &msg).await;

            // Send updated depth after order placement
            if let Some((bids, asks)) = state.engine.get_order_book_depth(&symbol, 10) {
                let spread = match (bids.first(), asks.first()) {
                    (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                    _ => None,
                };
                let depth_msg = ServerMessage::DepthUpdate {
                    symbol: symbol.clone(),
                    bids,
                    asks,
                    spread,
                };
                send_message(sender, &depth_msg).await;
            }
        }
        Err(e) => {
            let error_code = match e.to_string().as_str() {
                s if s.contains("Insufficient funds") => "INSUFFICIENT_FUNDS",
                s if s.contains("Insufficient shares") => "INSUFFICIENT_SHARES",
                s if s.contains("Insufficient margin") => "INSUFFICIENT_MARGIN",
                s if s.contains("Market is closed") => "MARKET_CLOSED",
                s if s.contains("not found") => "NOT_FOUND",
                _ => "ORDER_ERROR",
            };

            // Log order rejected event
            state.event_log.log_order_rejected(uid, &symbol, &side, qty, price, &e.to_string());

            let msg = ServerMessage::OrderRejected {
                reason: e.to_string(),
                error_code: error_code.to_string(),
            };
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_cancel_order(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    symbol: &str,
    order_id: u64,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::error("NOT_AUTHENTICATED", "Please authenticate first");
            send_message(sender, &msg).await;
            return;
        }
    };

    match state.engine.cancel_order(uid, symbol, order_id).await {
        Ok(cancelled) => {
            tracing::info!("Order {} cancelled", cancelled.id);

            // Log order cancelled event
            state.event_log.log_order_cancelled(order_id, uid, symbol, "User requested");

            let msg = ServerMessage::OrderCancelled { order_id: cancelled.id };
            send_message(sender, &msg).await;

            // Send updated depth after cancellation
            if let Some((bids, asks)) = state.engine.get_order_book_depth(symbol, 10) {
                let spread = match (bids.first(), asks.first()) {
                    (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                    _ => None,
                };
                let depth_msg = ServerMessage::DepthUpdate {
                    symbol: symbol.to_string(),
                    bids,
                    asks,
                    spread,
                };
                send_message(sender, &depth_msg).await;
            }

            // Send updated portfolio
            if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                let net_worth = calculate_net_worth(&user, &state.market);
                let portfolio_msg = ServerMessage::PortfolioUpdate {
                    money: user.money,
                    locked: user.locked_money,
                    margin_locked: user.margin_locked,
                    net_worth,
                    items: user.portfolio,
                };
                send_message(sender, &portfolio_msg).await;
            }
        }
        Err(e) => {
            let msg = ServerMessage::error("CANCEL_FAILED", &e.to_string());
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_subscribe(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    subscribed_symbols: &mut Vec<String>,
    symbol: &str,
) {
    if !subscribed_symbols.contains(&symbol.to_string()) {
        subscribed_symbols.push(symbol.to_string());
    }

    // Send historical candles
    let candles = state.market.get_candles(symbol);
    for candle in candles {
        let msg = ServerMessage::CandleUpdate { 
            symbol: symbol.to_string(), 
            candle 
        };
        send_message(sender, &msg).await;
    }

    // Send current depth
    handle_get_depth(sender, state, symbol, 10).await;
}

async fn handle_get_depth(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
    levels: usize,
) {
    match state.engine.get_order_book_depth(symbol, levels) {
        Some((bids, asks)) => {
            let spread = match (bids.first(), asks.first()) {
                (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                _ => None,
            };
            
            let msg = ServerMessage::DepthUpdate {
                symbol: symbol.to_string(),
                bids,
                asks,
                spread,
            };
            send_message(sender, &msg).await;
        }
        None => {
            let msg = ServerMessage::error("SYMBOL_NOT_FOUND", &format!("Symbol {} not found", symbol));
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_admin_action(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    action: &str,
    payload: serde_json::Value,
) {
    let uid = match user_id {
        Some(id) if id == 1 => id, // Simple admin check
        Some(_) => {
            let msg = ServerMessage::error("UNAUTHORIZED", "Admin privileges required");
            send_message(sender, &msg).await;
            return;
        }
        None => {
            let msg = ServerMessage::error("NOT_AUTHENTICATED", "Please authenticate first");
            send_message(sender, &msg).await;
            return;
        }
    };

    match action {
        "ToggleMarket" => {
            if let Some(open) = payload.get("open").and_then(|v| v.as_bool()) {
                state.admin.toggle_market(open);
                let msg = ServerMessage::MarketStatus { is_open: open };
                send_message(sender, &msg).await;
                tracing::info!("Admin {} set market open={}", uid, open);

                // Log market state change
                if open {
                    state.event_log.log_market_opened();
                } else {
                    state.event_log.log_market_closed();
                }
            }
        }
        "SetVolatility" => {
            if let (Some(symbol), Some(vol)) = (
                payload.get("symbol").and_then(|v| v.as_str()),
                payload.get("volatility").and_then(|v| v.as_i64())
            ) {
                // Get old volatility for logging
                let old_vol = state.company_repo.find_by_symbol(symbol).await
                    .ok().flatten().map(|c| c.volatility).unwrap_or(0);

                match state.admin.set_company_volatility(symbol, vol).await {
                    Ok(_) => {
                        // Log volatility change
                        state.event_log.log_volatility_changed(symbol, old_vol, vol);

                        let msg = ServerMessage::System {
                            message: format!("Volatility for {} set to {}", symbol, vol)
                        };
                        send_message(sender, &msg).await;
                    }
                    Err(e) => {
                        let msg = ServerMessage::error("ADMIN_ERROR", &e);
                        send_message(sender, &msg).await;
                    }
                }
            }
        }
        "CreateCompany" => {
            if let (Some(symbol), Some(name), Some(sector), Some(vol)) = (
                payload.get("symbol").and_then(|v| v.as_str()),
                payload.get("name").and_then(|v| v.as_str()),
                payload.get("sector").and_then(|v| v.as_str()),
                payload.get("volatility").and_then(|v| v.as_i64())
            ) {
                match state.admin.create_company(
                    symbol.to_string(),
                    name.to_string(),
                    sector.to_string(),
                    vol
                ).await {
                    Ok(_) => {
                        // Log company creation (initial price is base price $100)
                        let initial_price = 100 * crate::domain::models::PRICE_SCALE;
                        state.event_log.log_company_created(symbol, name, sector, initial_price);

                        let msg = ServerMessage::System {
                            message: format!("Company {} ({}) created", symbol, name)
                        };
                        send_message(sender, &msg).await;
                    }
                    Err(e) => {
                        let msg = ServerMessage::error("ADMIN_ERROR", &e);
                        send_message(sender, &msg).await;
                    }
                }
            }
        }
        "InitGame" => {
            // Initialize/reset the game with equal net worth for all traders
            // Frontend sends dollar amount, we need to scale it
            let starting_cash = payload.get("starting_cash")
                .and_then(|v| v.as_i64())
                .map(|v| v * crate::domain::models::PRICE_SCALE) // Scale to internal format
                .unwrap_or(100_000 * crate::domain::models::PRICE_SCALE); // Default $100,000
            let shares_per_trader = payload.get("shares_per_trader")
                .and_then(|v| v.as_u64())
                .unwrap_or(100); // Default 100 shares per company per trader

            // Get trader count before init for logging
            let num_traders = state.user_repo.all().await.map(|u| u.len()).unwrap_or(0);

            match state.admin.init_game(starting_cash, shares_per_trader).await {
                Ok(summary) => {
                    tracing::info!("Admin {} initialized game: {}", uid, summary);

                    // Log game initialization
                    state.event_log.log_game_initialized(
                        num_traders,
                        starting_cash,
                        shares_per_trader as i64,
                    );

                    let msg = ServerMessage::System { message: summary };
                    send_message(sender, &msg).await;
                    // Also send market status (market is closed after init)
                    let status_msg = ServerMessage::MarketStatus { is_open: false };
                    send_message(sender, &status_msg).await;
                }
                Err(e) => {
                    let msg = ServerMessage::error("INIT_GAME_ERROR", &e);
                    send_message(sender, &msg).await;
                }
            }
        }
        "SetBankrupt" => {
            if let Some(symbol) = payload.get("symbol").and_then(|v| v.as_str()) {
                match state.admin.set_company_bankrupt(symbol, true).await {
                    Ok(_) => {
                        // Log company bankruptcy
                        state.event_log.log_company_bankrupt(symbol);

                        let msg = ServerMessage::System {
                            message: format!("Company {} marked as bankrupt", symbol)
                        };
                        send_message(sender, &msg).await;
                    }
                    Err(e) => {
                        let msg = ServerMessage::error("ADMIN_ERROR", &e);
                        send_message(sender, &msg).await;
                    }
                }
            }
        }
        "BanTrader" => {
            if let (Some(target_user_id), Some(banned)) = (
                payload.get("user_id").and_then(|v| v.as_u64()),
                payload.get("banned").and_then(|v| v.as_bool())
            ) {
                match state.admin.set_trader_banned(target_user_id, banned).await {
                    Ok(_) => {
                        // Log ban/unban action
                        if banned {
                            state.event_log.log_trader_banned(target_user_id, "Admin action");
                        } else {
                            state.event_log.log_trader_unbanned(target_user_id);
                        }

                        let action = if banned { "banned" } else { "unbanned" };
                        let msg = ServerMessage::System {
                            message: format!("Trader {} {}", target_user_id, action)
                        };
                        send_message(sender, &msg).await;
                    }
                    Err(e) => {
                        let msg = ServerMessage::error("ADMIN_ERROR", &e);
                        send_message(sender, &msg).await;
                    }
                }
            }
        }
        "MuteTrader" => {
            if let (Some(target_user_id), Some(muted)) = (
                payload.get("user_id").and_then(|v| v.as_u64()),
                payload.get("muted").and_then(|v| v.as_bool())
            ) {
                match state.admin.set_trader_chat(target_user_id, !muted).await {
                    Ok(_) => {
                        // Log mute/unmute action
                        if muted {
                            state.event_log.log_trader_chat_muted(target_user_id);
                        } else {
                            state.event_log.log_trader_chat_unmuted(target_user_id);
                        }

                        let action = if muted { "muted" } else { "unmuted" };
                        let msg = ServerMessage::System {
                            message: format!("Trader {} chat {}", target_user_id, action)
                        };
                        send_message(sender, &msg).await;
                    }
                    Err(e) => {
                        let msg = ServerMessage::error("ADMIN_ERROR", &e);
                        send_message(sender, &msg).await;
                    }
                }
            }
        }
        "GetAllTrades" => {
            let user_id_filter = payload.get("user_id").and_then(|v| v.as_u64());
            let symbol_filter = payload.get("symbol").and_then(|v| v.as_str());
            let page = payload.get("page").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let page_size = payload.get("page_size").and_then(|v| v.as_u64()).unwrap_or(20) as u32;

            let (trades, total_count, has_more) = state.trade_history.get_all_trades_admin(
                user_id_filter,
                symbol_filter,
                page,
                page_size,
            );

            let msg = ServerMessage::AdminTradeHistory {
                trades,
                total_count,
                page,
                page_size,
                has_more,
            };
            send_message(sender, &msg).await;
        }
        "GetAllOpenOrders" => {
            let symbol_filter = payload.get("symbol").and_then(|v| v.as_str());

            // Build user names map
            let mut user_names = std::collections::HashMap::new();
            if let Ok(users) = state.user_repo.all().await {
                for user in users {
                    user_names.insert(user.id, user.name.clone());
                }
            }

            let orders = state.orders.get_all_orders_admin(symbol_filter, &user_names);
            let total_count = orders.len();

            let msg = ServerMessage::AdminOpenOrders {
                orders,
                total_count,
            };
            send_message(sender, &msg).await;
        }
        "GetOrderbook" => {
            if let Some(symbol) = payload.get("symbol").and_then(|v| v.as_str()) {
                // Build user names map
                let mut user_names = std::collections::HashMap::new();
                if let Ok(users) = state.user_repo.all().await {
                    for user in users {
                        user_names.insert(user.id, user.name.clone());
                    }
                }

                // Get all orders for this symbol, separated into bids and asks
                let orders = state.orders.get_all_orders_admin(Some(symbol), &user_names);
                let (bids, asks): (Vec<_>, Vec<_>) = orders.into_iter().partition(|o| {
                    matches!(o.side, crate::domain::models::OrderSide::Buy)
                });

                let msg = ServerMessage::AdminOrderbook {
                    symbol: symbol.to_string(),
                    bids,
                    asks,
                };
                send_message(sender, &msg).await;
            } else {
                let msg = ServerMessage::error("MISSING_SYMBOL", "Symbol required");
                send_message(sender, &msg).await;
            }
        }
        "GetDashboardMetrics" => {
            // Calculate all metrics
            let users = state.user_repo.all().await.unwrap_or_default();
            let total_traders = users.len();
            let active_traders = state.sessions.active_session_count();
            let total_trades = state.trade_history.get_total_trade_count();
            let total_volume = state.trade_history.get_total_volume();
            let recent_volume = state.trade_history.get_recent_volume(300); // Last 5 minutes
            let halted_symbols_count = state.market.get_halted_symbols().len();
            let open_orders_count = state.orders.get_total_open_orders_count();
            let market_open = state.engine.is_market_open();

            // Calculate total market cap (sum of all users' net worth)
            let mut total_market_cap: i64 = 0;
            for user in &users {
                total_market_cap += calculate_net_worth(user, &state.market);
            }

            let metrics = AdminDashboardMetrics {
                total_traders,
                active_traders,
                total_trades,
                total_volume,
                recent_volume,
                total_market_cap,
                halted_symbols_count,
                open_orders_count,
                market_open,
                timestamp: chrono::Utc::now().timestamp(),
            };

            let msg = ServerMessage::AdminDashboardMetrics { metrics };
            send_message(sender, &msg).await;
        }
        _ => {
            let msg = ServerMessage::error("UNKNOWN_ACTION", &format!("Unknown admin action: {}", action));
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_chat(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    message: String,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::error("NOT_AUTHENTICATED", "Please authenticate to chat");
            send_message(sender, &msg).await;
            return;
        }
    };

    // Validate message
    let message = message.trim();
    if message.is_empty() {
        return;
    }
    if message.len() > 500 {
        let msg = ServerMessage::error("MESSAGE_TOO_LONG", "Message must be under 500 characters");
        send_message(sender, &msg).await;
        return;
    }

    if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
        if !user.chat_enabled {
            let msg = ServerMessage::error("CHAT_DISABLED", "Your chat privileges have been revoked");
            send_message(sender, &msg).await;
            return;
        }

        let chat_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: uid,
            username: user.name.clone(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Log chat message (if chat logging is enabled)
        state.event_log.log_chat_message(uid, &user.name, message);

        state.chat.broadcast_message(chat_msg);
    }
}

async fn handle_get_portfolio(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::error("NOT_AUTHENTICATED", "Please authenticate first");
            send_message(sender, &msg).await;
            return;
        }
    };

    match state.user_repo.find_by_id(uid).await {
        Ok(Some(user)) => {
            let net_worth = calculate_net_worth(&user, &state.market);
            let msg = ServerMessage::PortfolioUpdate { 
                money: user.money, 
                locked: user.locked_money,
                margin_locked: user.margin_locked,
                net_worth,
                items: user.portfolio,
            };
            send_message(sender, &msg).await;
        }
        _ => {
            let msg = ServerMessage::error("USER_NOT_FOUND", "User not found");
            send_message(sender, &msg).await;
        }
    }
}

/// Calculate user's total net worth (cash + portfolio value)
fn calculate_net_worth(user: &User, market: &MarketService) -> i64 {
    let mut portfolio_value: i64 = 0;

    for item in &user.portfolio {
        if let Some(price) = market.get_last_price(&item.symbol) {
            // Long positions add value
            portfolio_value += (item.qty as i64) * price;
            // Short positions subtract value (liability)
            portfolio_value -= (item.short_qty as i64) * price;
        }
    }

    user.money + user.locked_money + user.margin_locked + portfolio_value
}

// ==================== NEW HANDLERS FOR BACKEND-DRIVEN UI ====================

/// Handle full state sync request
async fn handle_request_sync(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    subscribed_symbols: &[String],
    component: Option<String>,
) {
    let sync_id = next_sync_id();

    match component.as_deref() {
        None => {
            // Full state sync
            send_full_state_sync(sender, state, user_id, subscribed_symbols.first().map(|s| s.as_str())).await;
        }
        Some("portfolio") => {
            if let Some(uid) = user_id {
                if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                    let portfolio_ui = compute_portfolio_ui(&user, &state.market);
                    let msg = ServerMessage::PortfolioSync {
                        sync_id,
                        money: portfolio_ui.money,
                        locked_money: portfolio_ui.locked_money,
                        margin_locked: portfolio_ui.margin_locked,
                        portfolio_value: portfolio_ui.portfolio_value,
                        net_worth: portfolio_ui.net_worth,
                        items: portfolio_ui.items,
                    };
                    send_message(sender, &msg).await;
                }
            } else {
                let msg = ServerMessage::error("NOT_AUTHENTICATED", "Authentication required");
                send_message(sender, &msg).await;
            }
        }
        Some("orders") => {
            if let Some(uid) = user_id {
                let orders = state.orders.get_user_orders(uid);
                let msg = ServerMessage::OpenOrdersSync { sync_id, orders };
                send_message(sender, &msg).await;
            }
        }
        Some("leaderboard") => {
            let entries = state.leaderboard.get_current();
            let msg = ServerMessage::LeaderboardSync { sync_id, entries };
            send_message(sender, &msg).await;
        }
        Some("indices") => {
            let indices = state.indices.get_all_indices();
            let msg = ServerMessage::IndicesSync { sync_id, indices };
            send_message(sender, &msg).await;
        }
        Some("news") => {
            let news = state.news.get_recent(20).into_iter().map(|n| NewsItemUI {
                id: n.id.clone(),
                headline: n.headline.clone(),
                symbol: n.symbol.clone(),
                sentiment: n.sentiment.clone(),
                impact: n.impact.clone(),
                timestamp: n.timestamp,
            }).collect();
            let msg = ServerMessage::NewsSync { sync_id, news };
            send_message(sender, &msg).await;
        }
        Some("chat") => {
            let messages = state.chat.get_recent(50);
            let msg = ServerMessage::ChatSync { sync_id, messages };
            send_message(sender, &msg).await;
        }
        Some(comp) if comp.starts_with("orderbook:") => {
            let symbol = &comp[10..];
            if let Some((bids, asks)) = state.engine.get_order_book_depth(symbol, 10) {
                let spread = match (bids.first(), asks.first()) {
                    (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                    _ => None,
                };
                let spread_percent = spread.and_then(|s| {
                    bids.first().map(|(bid_price, _)| {
                        if *bid_price != 0 {
                            (s as f64 / *bid_price as f64) * 100.0
                        } else {
                            0.0
                        }
                    })
                });
                let last_price = state.market.get_last_price(symbol);

                let orderbook = OrderbookUI {
                    symbol: symbol.to_string(),
                    bids: bids.into_iter().scan(0u64, |cum, (price, qty)| {
                        *cum += qty;
                        Some(OrderbookLevelUI { price, qty, order_count: 1, cumulative_qty: *cum })
                    }).collect(),
                    asks: asks.into_iter().scan(0u64, |cum, (price, qty)| {
                        *cum += qty;
                        Some(OrderbookLevelUI { price, qty, order_count: 1, cumulative_qty: *cum })
                    }).collect(),
                    spread,
                    spread_percent,
                    last_price,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                let msg = ServerMessage::OrderbookSync {
                    sync_id,
                    symbol: symbol.to_string(),
                    orderbook,
                };
                send_message(sender, &msg).await;
            }
        }
        Some(comp) if comp.starts_with("candles:") => {
            let symbol = &comp[8..];
            let candles: Vec<CandleUI> = state.market.get_candles(symbol).into_iter().map(|c| CandleUI {
                timestamp: c.timestamp,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume,
            }).collect();
            let msg = ServerMessage::CandlesSync {
                sync_id,
                symbol: symbol.to_string(),
                candles,
            };
            send_message(sender, &msg).await;
        }
        Some(comp) if comp.starts_with("stock_trades:") => {
            let symbol = &comp[13..];
            let trades = state.trade_history.get_symbol_trades(symbol, 20);
            let msg = ServerMessage::StockTradeHistory {
                symbol: symbol.to_string(),
                trades,
            };
            send_message(sender, &msg).await;
        }
        Some("trade_history") => {
            if let Some(uid) = user_id {
                let response = state.trade_history.get_user_trades(uid, 0, 20);
                let msg = ServerMessage::TradeHistory {
                    trades: response.trades,
                    total_count: response.total_count,
                    page: response.page,
                    page_size: response.page_size,
                    has_more: response.has_more,
                };
                send_message(sender, &msg).await;
            } else {
                let msg = ServerMessage::error("NOT_AUTHENTICATED", "Authentication required");
                send_message(sender, &msg).await;
            }
        }
        _ => {
            let msg = ServerMessage::error("UNKNOWN_COMPONENT", "Unknown sync component");
            send_message(sender, &msg).await;
        }
    }
}

/// Send full state sync to client
async fn send_full_state_sync(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    active_symbol: Option<&str>,
) {
    let sync_id = next_sync_id();
    let timestamp = chrono::Utc::now().timestamp();

    // Market state
    let market_open = state.engine.is_market_open();
    let halted_symbols = state.market.get_halted_symbols();

    // Companies with current prices
    let companies: Vec<CompanyUI> = if let Ok(companies) = state.company_repo.all().await {
        companies.iter().map(|c| {
            let candles = state.market.get_candles(&c.symbol);
            let (current_price, price_change, price_change_percent) = if let Some(last) = candles.last() {
                let current = last.close;
                let first = candles.first().map(|c| c.open).unwrap_or(current);
                let change = current - first;
                let change_pct = if first != 0 {
                    (change as f64 / first as f64) * 100.0
                } else {
                    0.0
                };
                (Some(current), Some(change), Some(change_pct))
            } else {
                (None, None, None)
            };

            CompanyUI {
                id: c.id,
                symbol: c.symbol.clone(),
                name: c.name.clone(),
                sector: c.sector.clone(),
                current_price,
                price_change,
                price_change_percent,
                volume: candles.iter().map(|c| c.volume).sum(),
                bankrupt: c.bankrupt,
            }
        }).collect()
    } else {
        vec![]
    };

    // User-specific state
    let (portfolio, open_orders) = if let Some(uid) = user_id {
        let user = state.user_repo.find_by_id(uid).await.ok().flatten();
        let portfolio = user.as_ref().map(|u| compute_portfolio_ui(u, &state.market));
        let orders = state.orders.get_user_orders(uid);
        (portfolio, orders)
    } else {
        (None, vec![])
    };

    // Market data
    let indices = state.indices.get_all_indices();
    let leaderboard = state.leaderboard.get_current();
    let news = state.news.get_recent(20).into_iter().map(|n| NewsItemUI {
        id: n.id.clone(),
        headline: n.headline.clone(),
        symbol: n.symbol.clone(),
        sentiment: n.sentiment.clone(),
        impact: n.impact.clone(),
        timestamp: n.timestamp,
    }).collect();
    let chat_history = state.chat.get_recent(50);

    // Symbol-specific data
    let (orderbook, candles, recent_trades) = if let Some(sym) = active_symbol {
        let ob = state.engine.get_order_book_depth(sym, 10).map(|(bids, asks)| {
            let spread = match (bids.first(), asks.first()) {
                (Some((bid_price, _)), Some((ask_price, _))) => Some(ask_price - bid_price),
                _ => None,
            };
            let spread_percent = spread.and_then(|s| {
                bids.first().map(|(bid_price, _)| {
                    if *bid_price != 0 {
                        (s as f64 / *bid_price as f64) * 100.0
                    } else {
                        0.0
                    }
                })
            });
            let last_price = state.market.get_last_price(sym);

            OrderbookUI {
                symbol: sym.to_string(),
                bids: bids.into_iter().scan(0u64, |cum, (price, qty)| {
                    *cum += qty;
                    Some(OrderbookLevelUI { price, qty, order_count: 1, cumulative_qty: *cum })
                }).collect(),
                asks: asks.into_iter().scan(0u64, |cum, (price, qty)| {
                    *cum += qty;
                    Some(OrderbookLevelUI { price, qty, order_count: 1, cumulative_qty: *cum })
                }).collect(),
                spread,
                spread_percent,
                last_price,
                timestamp,
            }
        });
        let candles_data: Vec<CandleUI> = state.market.get_candles(sym).into_iter().map(|c| CandleUI {
            timestamp: c.timestamp,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }).collect();
        let trades = state.trade_history.get_symbol_trades(sym, 50);
        (ob, Some(candles_data), trades)
    } else {
        (None, None, vec![])
    };

    let payload = FullStateSyncPayload {
        market_open,
        halted_symbols,
        companies,
        portfolio,
        open_orders,
        indices,
        leaderboard,
        news,
        chat_history,
        active_symbol: active_symbol.map(|s| s.to_string()),
        orderbook,
        candles,
        recent_trades,
        sync_id,
        timestamp,
    };

    let msg = ServerMessage::FullStateSync { payload };
    send_message(sender, &msg).await;
    debug!("Sent full state sync (sync_id={})", sync_id);
}

/// Send portfolio sync to client
async fn send_portfolio_sync(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: u64,
) {
    if let Ok(Some(user)) = state.user_repo.find_by_id(user_id).await {
        let portfolio_ui = compute_portfolio_ui(&user, &state.market);
        let msg = ServerMessage::PortfolioUpdateUI {
            money: portfolio_ui.money,
            locked_money: portfolio_ui.locked_money,
            margin_locked: portfolio_ui.margin_locked,
            portfolio_value: portfolio_ui.portfolio_value,
            net_worth: portfolio_ui.net_worth,
            items: portfolio_ui.items,
        };
        send_message(sender, &msg).await;
    }
}

/// Compute UI-ready portfolio from user data
fn compute_portfolio_ui(user: &User, market: &MarketService) -> PortfolioStateUI {
    let mut items: Vec<PortfolioItemUI> = Vec::new();
    let mut portfolio_value: i64 = 0;

    for item in &user.portfolio {
        let current_price = market.get_last_price(&item.symbol)
            .unwrap_or(item.average_buy_price);

        let market_value = (item.qty as i64) * current_price;
        let cost_basis = (item.qty as i64) * item.average_buy_price;
        let unrealized_pnl = market_value - cost_basis;
        let unrealized_pnl_percent = if cost_basis != 0 {
            (unrealized_pnl as f64 / cost_basis as f64) * 100.0
        } else {
            0.0
        };

        // Short position calculations
        let short_market_value = (item.short_qty as i64) * current_price;
        // For shorts, profit is when price goes down
        let short_unrealized_pnl = 0; // Would need short entry price to calculate

        portfolio_value += market_value;
        portfolio_value -= short_market_value; // Shorts are liability

        items.push(PortfolioItemUI {
            symbol: item.symbol.clone(),
            qty: item.qty,
            short_qty: item.short_qty,
            locked_qty: item.locked_qty,
            average_buy_price: item.average_buy_price,
            current_price,
            market_value,
            cost_basis,
            unrealized_pnl,
            unrealized_pnl_percent,
            short_market_value,
            short_unrealized_pnl,
        });
    }

    let net_worth = user.money + user.locked_money + user.margin_locked + portfolio_value;

    PortfolioStateUI {
        money: user.money,
        locked_money: user.locked_money,
        margin_locked: user.margin_locked,
        total_available: user.money,
        portfolio_value,
        net_worth,
        items,
    }
}

/// Handle trade history request
async fn handle_get_trade_history(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    page: Option<u32>,
    page_size: Option<u32>,
    _symbol: Option<String>,
) {
    if let Some(uid) = user_id {
        let response = state.trade_history.get_user_trades(
            uid,
            page.unwrap_or(0),
            page_size.unwrap_or(20),
        );

        let msg = ServerMessage::TradeHistory {
            trades: response.trades,
            total_count: response.total_count,
            page: response.page,
            page_size: response.page_size,
            has_more: response.has_more,
        };
        send_message(sender, &msg).await;
    } else {
        let msg = ServerMessage::error("NOT_AUTHENTICATED", "Authentication required");
        send_message(sender, &msg).await;
    }
}

/// Handle stock trade history request (for orderbook tab)
async fn handle_get_stock_trades(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
    count: Option<usize>,
) {
    let trades = state.trade_history.get_symbol_trades(symbol, count.unwrap_or(50));
    let msg = ServerMessage::StockTradeHistory {
        symbol: symbol.to_string(),
        trades,
    };
    send_message(sender, &msg).await;
}
