use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::domain::models::{Order, OrderSide, OrderType, TimeInForce, next_order_id};
use crate::service::engine::MatchingEngine;
use crate::service::market::MarketService;
use crate::service::admin::AdminService;
use crate::service::indices::IndicesService;
use crate::service::news::NewsService;
use crate::service::leaderboard::LeaderboardService;
use crate::service::chat::ChatService;
use crate::repository::UserRepository;
use crate::domain::models::ChatMessage;

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
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    Auth { token: String }, // For now, token = user_id
    PlaceOrder {
        symbol: String,
        side: String, // "Buy", "Sell", "Short"
        order_type: String, // "Market", "Limit"
        qty: u64,
        price: i64,
    },
    Subscribe { symbol: String },
    AdminAction { action: String, payload: serde_json::Value },
    Chat { message: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    AuthSuccess { user_id: u64, name: String },
    AuthFailed { reason: String },
    OrderAck { order_id: u64, status: String },
    OrderRejected { reason: String },
    TradeUpdate { symbol: String, price: i64, qty: u64 },
    CandleUpdate { symbol: String, candle: crate::domain::models::Candle },
    IndexUpdate { name: String, value: i64 },
    PortfolioUpdate { 
        money: i64, 
        locked: i64, 
        items: Vec<crate::domain::models::Portfolio> 
    },
    CircuitBreaker { symbol: String, halted_until: i64 },
    NewsUpdate { news: crate::service::news::NewsItem },
    LeaderboardUpdate { entries: Vec<crate::service::leaderboard::LeaderboardEntry> },
    ChatUpdate { message: ChatMessage },
    Error { message: String },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    tracing::info!("New WebSocket connection");
    let (mut sender, mut receiver) = socket.split();
    let mut user_id: Option<u64> = None;

    // Subscribe to trades and candles
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
                        tracing::info!("Received message: {}", text);
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                match client_msg {
                                    ClientMessage::Auth { token } => {
                                        // Mock Auth: token is user_id string
                                        if let Ok(uid) = token.parse::<u64>() {
                                            if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                                                user_id = Some(uid);
                                                tracing::info!("User {} authenticated", uid);
                                                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::AuthSuccess { 
                                                    user_id: uid, 
                                                    name: user.name.clone()
                                                }).unwrap())).await;

                                                // Send initial portfolio
                                                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::PortfolioUpdate { 
                                                    money: user.money, 
                                                    locked: user.locked_money, 
                                                    items: user.portfolio.clone() 
                                                }).unwrap())).await;
                                            } else {
                                                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::AuthFailed { 
                                                    reason: "User not found".to_string() 
                                                }).unwrap())).await;
                                            }
                                        } else {
                                            let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::AuthFailed { 
                                                reason: "Invalid token".to_string() 
                                            }).unwrap())).await;
                                        }
                                    },
                                    ClientMessage::PlaceOrder { symbol, side, order_type, qty, price } => {
                                        if let Some(uid) = user_id {
                                            let side_enum = match side.as_str() {
                                                "Buy" => OrderSide::Buy,
                                                "Sell" => OrderSide::Sell,
                                                "Short" => OrderSide::Short,
                                                _ => {
                                                    let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::OrderRejected { reason: "Invalid side".to_string() }).unwrap())).await;
                                                    continue;
                                                }
                                            };
                                            let type_enum = match order_type.as_str() {
                                                "Market" => OrderType::Market,
                                                "Limit" => OrderType::Limit,
                                                _ => {
                                                    let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::OrderRejected { reason: "Invalid type".to_string() }).unwrap())).await;
                                                    continue;
                                                }
                                            };

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
                                                time_in_force: TimeInForce::GTC,
                                            };

                                            tracing::info!("Placing order: {:?}", order);
                                            match state.engine.place_order(order).await {
                                                Ok(processed) => {
                                                    tracing::info!("Order placed successfully: {}", processed.id);
                                                    let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::OrderAck { 
                                                        order_id: processed.id, 
                                                        status: format!("{:?}", processed.status) 
                                                    }).unwrap())).await;
                                                },
                                                Err(e) => {
                                                    let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::OrderRejected { reason: e }).unwrap())).await;
                                                }
                                            }
                                        } else {
                                            let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::Error { message: "Not authenticated".to_string() }).unwrap())).await;
                                        }
                                    },
                                    ClientMessage::Subscribe { symbol } => {
                                        // Send historical candles
                                        let candles = state.market.get_candles(&symbol);
                                        for candle in candles {
                                            let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::CandleUpdate { 
                                                symbol: symbol.clone(), 
                                                candle 
                                            }).unwrap())).await;
                                        }
                                    },
                                    ClientMessage::AdminAction { action, payload } => {
                                        if let Some(uid) = user_id {
                                            if uid == 1 { // Simple Admin Check
                                                match action.as_str() {
                                                    "ToggleMarket" => {
                                                        if let Some(open) = payload.get("open").and_then(|v| v.as_bool()) {
                                                            state.admin.toggle_market(open);
                                                            // Broadcast system message?
                                                        }
                                                    },
                                                    "SetVolatility" => {
                                                        if let (Some(symbol), Some(vol)) = (payload.get("symbol").and_then(|v| v.as_str()), payload.get("volatility").and_then(|v| v.as_i64())) {
                                                            let _ = state.admin.set_company_volatility(symbol, vol).await;
                                                        }
                                                    },
                                                    "CreateCompany" => {
                                                        if let (Some(symbol), Some(name), Some(sector), Some(vol)) = (
                                                            payload.get("symbol").and_then(|v| v.as_str()),
                                                            payload.get("name").and_then(|v| v.as_str()),
                                                            payload.get("sector").and_then(|v| v.as_str()),
                                                            payload.get("volatility").and_then(|v| v.as_i64())
                                                        ) {
                                                            let _ = state.admin.create_company(symbol.to_string(), name.to_string(), sector.to_string(), vol).await;
                                                        }
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    ClientMessage::Chat { message } => {
                                        if let Some(uid) = user_id {
                                            if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                                                let chat_msg = ChatMessage {
                                                    id: uuid::Uuid::new_v4().to_string(),
                                                    user_id: uid,
                                                    username: user.name,
                                                    message,
                                                    timestamp: chrono::Utc::now().timestamp(),
                                                };
                                                state.chat.broadcast_message(chat_msg);
                                            }
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::Error { message: format!("Invalid JSON: {}", e) }).unwrap())).await;
                            }
                        }
                    },
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
            // Handle outgoing trade updates
            Ok(trade) = trade_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::TradeUpdate { 
                    symbol: trade.symbol.clone(), 
                    price: trade.price, 
                    qty: trade.qty 
                }).unwrap())).await;

                // If this user was involved, send portfolio update
                if let Some(uid) = user_id {
                    if trade.maker_user_id == uid || trade.taker_user_id == uid {
                        if let Ok(Some(user)) = state.user_repo.find_by_id(uid).await {
                             let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::PortfolioUpdate { 
                                money: user.money, 
                                locked: user.locked_money, 
                                items: user.portfolio 
                            }).unwrap())).await;
                        }
                    }
                }
            }
            // Handle outgoing candle updates
            Ok(candle) = candle_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::CandleUpdate { 
                    symbol: candle.symbol.clone(), 
                    candle 
                }).unwrap())).await;
            }
            // Handle circuit breaker updates
            Ok((symbol, halted_until)) = cb_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::CircuitBreaker { 
                    symbol, 
                    halted_until 
                }).unwrap())).await;
            }
            // Handle outgoing index updates
            Ok(index) = index_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::IndexUpdate { 
                    name: index.name, 
                    value: index.value 
                }).unwrap())).await;
            }
            // Handle outgoing news updates
            Ok(news) = news_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::NewsUpdate { 
                    news 
                }).unwrap())).await;
            }
            // Handle outgoing leaderboard updates
            Ok(entries) = lb_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::LeaderboardUpdate { 
                    entries 
                }).unwrap())).await;
            }
            // Handle outgoing chat messages
            Ok(message) = chat_rx.recv() => {
                let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::ChatUpdate { 
                    message 
                }).unwrap())).await;
            }
            else => break,
        }
    }
}
