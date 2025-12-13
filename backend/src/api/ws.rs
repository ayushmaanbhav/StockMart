//! WebSocket API handler.
//!
//! This module contains the main WebSocket upgrade handler and connection loop.
//! The implementation delegates to the presentation layer for:
//! - State management (AppState)
//! - Connection lifecycle (ConnectionState, BroadcastSubscriptions)
//! - Message handling (handlers module)

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
};
use futures::stream::StreamExt;
use std::sync::Arc;
use tracing::{debug, error};

// Re-export AppState for use in main.rs
pub use crate::presentation::websocket::state::AppState;

use crate::presentation::websocket::{
    connection::{
        self, cleanup_connection, handle_candle_broadcast, handle_chat_broadcast,
        handle_circuit_breaker_broadcast, handle_index_broadcast, handle_leaderboard_broadcast,
        handle_news_broadcast, handle_trade_broadcast, log_connection_closed, log_connection_error,
        log_connection_established, send_initial_config, BroadcastSubscriptions, ConnectionState,
    },
    handlers::{
        handle_admin_action, handle_auth, handle_cancel_order, handle_chat, handle_get_depth,
        handle_get_portfolio, handle_get_stock_trades, handle_get_trade_history, handle_login,
        handle_place_order, handle_register, handle_request_sync, handle_subscribe,
    },
    messages::{ClientMessage, CurrencyConfigPayload, ServerMessage},
};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Main WebSocket connection handler
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    log_connection_established();

    let (mut sender, mut receiver) = socket.split();
    let mut conn_state = ConnectionState::new();

    // Send initial config to the client
    send_initial_config(&mut sender, &state).await;

    // Subscribe to broadcast channels
    let mut subs = BroadcastSubscriptions::from_state(&state);

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
                            &mut conn_state,
                        ).await {
                            error!("Error handling client message: {}", e);
                        }
                    },
                    Ok(Message::Ping(data)) => {
                        use futures::SinkExt;
                        let _ = sender.send(Message::Pong(data)).await;
                    },
                    Ok(Message::Close(_)) => {
                        log_connection_closed(&conn_state);
                        break;
                    },
                    Err(e) => {
                        log_connection_error(&conn_state, &e.to_string());
                        break;
                    },
                    _ => {}
                }
            }

            // Handle trade updates
            Ok(trade) = subs.trades.recv() => {
                handle_trade_broadcast(&mut sender, &state, &conn_state, trade).await;
            }

            // Handle candle updates
            Ok(candle) = subs.candles.recv() => {
                handle_candle_broadcast(&mut sender, candle).await;
            }

            // Handle circuit breaker updates
            Ok((symbol, halted_until)) = subs.circuit_breakers.recv() => {
                handle_circuit_breaker_broadcast(&mut sender, symbol, halted_until).await;
            }

            // Handle index updates
            Ok(index) = subs.indices.recv() => {
                handle_index_broadcast(&mut sender, index).await;
            }

            // Handle news updates
            Ok(news) = subs.news.recv() => {
                handle_news_broadcast(&mut sender, news).await;
            }

            // Handle leaderboard updates
            Ok(entries) = subs.leaderboard.recv() => {
                handle_leaderboard_broadcast(&mut sender, entries).await;
            }

            // Handle chat messages
            Ok(message) = subs.chat.recv() => {
                handle_chat_broadcast(&mut sender, message).await;
            }

            else => break,
        }
    }

    // Cleanup session on disconnect
    cleanup_connection(&state, &conn_state);
}

/// Route client messages to appropriate handlers
async fn handle_client_message(
    text: &str,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    conn_state: &mut ConnectionState,
) -> Result<(), String> {
    let client_msg: ClientMessage = serde_json::from_str(text).map_err(|e| {
        let msg = ServerMessage::error("PARSE_ERROR", &format!("Invalid JSON: {}", e));
        let _ = futures::executor::block_on(connection::send_message(sender, &msg));
        e.to_string()
    })?;

    debug!(
        "Handling client message: {:?}",
        std::mem::discriminant(&client_msg)
    );

    match client_msg {
        ClientMessage::Auth { token } => {
            handle_auth(
                sender,
                state,
                &mut conn_state.user_id,
                &mut conn_state.session_id,
                &token,
            )
            .await;
        }

        ClientMessage::Login { regno, password } => {
            handle_login(
                sender,
                state,
                &mut conn_state.user_id,
                &mut conn_state.session_id,
                regno,
                password,
            )
            .await;
        }

        ClientMessage::Register {
            regno,
            name,
            password,
        } => {
            handle_register(
                sender,
                state,
                &mut conn_state.user_id,
                &mut conn_state.session_id,
                regno,
                name,
                password,
            )
            .await;
        }

        ClientMessage::PlaceOrder {
            symbol,
            side,
            order_type,
            time_in_force,
            qty,
            price,
        } => {
            handle_place_order(
                sender,
                state,
                conn_state.user_id,
                symbol,
                side,
                order_type,
                time_in_force,
                qty,
                price,
            )
            .await;
        }

        ClientMessage::CancelOrder { symbol, order_id } => {
            handle_cancel_order(sender, state, conn_state.user_id, &symbol, order_id).await;
        }

        ClientMessage::Subscribe { symbol } => {
            handle_subscribe(sender, state, &mut conn_state.subscribed_symbols, &symbol).await;
        }

        ClientMessage::GetDepth { symbol, levels } => {
            handle_get_depth(sender, state, &symbol, levels.unwrap_or(10)).await;
        }

        ClientMessage::AdminAction { action, payload } => {
            handle_admin_action(sender, state, conn_state.user_id, &action, payload).await;
        }

        ClientMessage::Chat { message } => {
            handle_chat(sender, state, conn_state.user_id, message).await;
        }

        ClientMessage::GetPortfolio => {
            handle_get_portfolio(sender, state, conn_state.user_id).await;
        }

        ClientMessage::Ping {} => {
            let msg = ServerMessage::Pong {
                timestamp: chrono::Utc::now().timestamp(),
            };
            connection::send_message(sender, &msg).await;
        }

        ClientMessage::GetConfig {} => {
            let public_config = state.config.get_public_config();
            let config_msg = ServerMessage::Config {
                registration_mode: format!("{:?}", public_config.registration_mode),
                chat_enabled: public_config.chat_enabled,
                currency: CurrencyConfigPayload::from(&public_config.currency),
            };
            connection::send_message(sender, &config_msg).await;
            debug!("Sent config response");
        }

        ClientMessage::RequestSync { component } => {
            handle_request_sync(
                sender,
                state,
                conn_state.user_id,
                &conn_state.subscribed_symbols,
                component,
            )
            .await;
        }

        ClientMessage::GetTradeHistory {
            page,
            page_size,
            symbol,
        } => {
            handle_get_trade_history(sender, state, conn_state.user_id, page, page_size, symbol)
                .await;
        }

        ClientMessage::GetStockTrades { symbol, count } => {
            handle_get_stock_trades(sender, state, &symbol, count).await;
        }
    }

    Ok(())
}
