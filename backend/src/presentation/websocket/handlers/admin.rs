//! Admin action handlers for WebSocket connections.
//!
//! Handles administrative operations like market control, user management,
//! and dashboard metrics.

use axum::extract::ws::{Message, WebSocket};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::api::ws::AppState;
use crate::domain::error::UserError;
use crate::domain::models::{OrderSide, PRICE_SCALE};
use crate::domain::ui_models::AdminDashboardMetrics;
use crate::domain::user::AdminAction;
use crate::presentation::websocket::messages::ServerMessage;

use super::helpers::calculate_net_worth;
use super::send_message;

/// Handle admin actions
pub async fn handle_admin_action(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    action: &str,
    payload: serde_json::Value,
) {
    // Parse the admin action for RBAC validation
    let admin_action = match AdminAction::from_str(action) {
        Some(a) => a,
        None => {
            let msg = ServerMessage::error(
                "UNKNOWN_ACTION",
                &format!("Unknown admin action: {}", action),
            );
            send_message(sender, &msg).await;
            return;
        }
    };

    // Check admin authorization using RBAC
    let uid = match user_id {
        Some(id) => {
            // Verify user exists and has admin role
            match state.user_repo.find_by_id(id).await {
                Ok(Some(user)) => {
                    // Use granular RBAC permission checks based on action type
                    let has_permission = match &admin_action {
                        AdminAction::ToggleMarket | AdminAction::SetVolatility => {
                            user.role.can_control_market()
                        }
                        AdminAction::CreateCompany | AdminAction::SetBankrupt => {
                            user.role.can_manage_companies()
                        }
                        AdminAction::InitGame => user.role.can_init_game(),
                        AdminAction::BanTrader | AdminAction::MuteTrader => {
                            user.role.can_manage_users()
                        }
                        AdminAction::GetAllTrades => user.role.can_view_all_trades(),
                        AdminAction::GetAllOpenOrders | AdminAction::GetOrderbook => {
                            user.role.can_view_all_orders()
                        }
                        AdminAction::GetDashboardMetrics => user.role.can_view_admin_dashboard(),
                    };

                    if !has_permission {
                        let msg = ServerMessage::from_user_error(UserError::PermissionDenied {
                            action: format!("{:?}", admin_action),
                        });
                        send_message(sender, &msg).await;
                        return;
                    }
                    id
                }
                Ok(None) => {
                    let msg = ServerMessage::from_user_error(UserError::NotFound { user_id: id });
                    send_message(sender, &msg).await;
                    return;
                }
                Err(_) => {
                    let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
                    send_message(sender, &msg).await;
                    return;
                }
            }
        }
        None => {
            let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
            send_message(sender, &msg).await;
            return;
        }
    };

    // Route to appropriate handler based on parsed action
    match admin_action {
        AdminAction::ToggleMarket => handle_toggle_market(sender, state, uid, &payload).await,
        AdminAction::SetVolatility => handle_set_volatility(sender, state, &payload).await,
        AdminAction::CreateCompany => handle_create_company(sender, state, &payload).await,
        AdminAction::InitGame => handle_init_game(sender, state, uid, &payload).await,
        AdminAction::SetBankrupt => handle_set_bankrupt(sender, state, &payload).await,
        AdminAction::BanTrader => handle_ban_trader(sender, state, &payload).await,
        AdminAction::MuteTrader => handle_mute_trader(sender, state, &payload).await,
        AdminAction::GetAllTrades => handle_get_all_trades(sender, state, &payload).await,
        AdminAction::GetAllOpenOrders => handle_get_all_open_orders(sender, state, &payload).await,
        AdminAction::GetOrderbook => handle_get_orderbook(sender, state, &payload).await,
        AdminAction::GetDashboardMetrics => handle_get_dashboard_metrics(sender, state).await,
    }
}

async fn handle_toggle_market(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    uid: u64,
    payload: &serde_json::Value,
) {
    if let Some(open) = payload.get("open").and_then(|v| v.as_bool()) {
        state.admin.toggle_market(open);
        let msg = ServerMessage::MarketStatus { is_open: open };
        send_message(sender, &msg).await;
        info!("Admin {} set market open={}", uid, open);

        if open {
            state.event_log.log_market_opened();
        } else {
            state.event_log.log_market_closed();
        }
    }
}

async fn handle_set_volatility(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let (Some(symbol), Some(vol)) = (
        payload.get("symbol").and_then(|v| v.as_str()),
        payload.get("volatility").and_then(|v| v.as_i64()),
    ) {
        let old_vol = state
            .company_repo
            .find_by_symbol(symbol)
            .await
            .ok()
            .flatten()
            .map(|c| c.volatility)
            .unwrap_or(0);

        match state.admin.set_company_volatility(symbol, vol).await {
            Ok(_) => {
                state.event_log.log_volatility_changed(symbol, old_vol, vol);
                let msg = ServerMessage::System {
                    message: format!("Volatility for {} set to {}", symbol, vol),
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

async fn handle_create_company(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let (Some(symbol), Some(name), Some(sector), Some(vol)) = (
        payload.get("symbol").and_then(|v| v.as_str()),
        payload.get("name").and_then(|v| v.as_str()),
        payload.get("sector").and_then(|v| v.as_str()),
        payload.get("volatility").and_then(|v| v.as_i64()),
    ) {
        match state
            .admin
            .create_company(
                symbol.to_string(),
                name.to_string(),
                sector.to_string(),
                vol,
            )
            .await
        {
            Ok(_) => {
                let initial_price = 100 * PRICE_SCALE;
                state
                    .event_log
                    .log_company_created(symbol, name, sector, initial_price);

                let msg = ServerMessage::System {
                    message: format!("Company {} ({}) created", symbol, name),
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

async fn handle_init_game(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    uid: u64,
    payload: &serde_json::Value,
) {
    let starting_cash = payload
        .get("starting_cash")
        .and_then(|v| v.as_i64())
        .map(|v| v * PRICE_SCALE)
        .unwrap_or(100_000 * PRICE_SCALE);
    let shares_per_trader = payload
        .get("shares_per_trader")
        .and_then(|v| v.as_u64())
        .unwrap_or(100);

    let num_traders = state.user_repo.all().await.map(|u| u.len()).unwrap_or(0);

    match state
        .admin
        .init_game(starting_cash, shares_per_trader)
        .await
    {
        Ok(summary) => {
            info!("Admin {} initialized game: {}", uid, summary);

            // Clear all open orders and trade history as part of game reset
            state.orders.clear_all();
            state.trade_history.clear_all();

            state.event_log.log_game_initialized(
                num_traders,
                starting_cash,
                shares_per_trader as i64,
            );

            let msg = ServerMessage::System { message: summary };
            send_message(sender, &msg).await;

            let status_msg = ServerMessage::MarketStatus { is_open: false };
            send_message(sender, &status_msg).await;
        }
        Err(e) => {
            let msg = ServerMessage::error("INIT_GAME_ERROR", &e);
            send_message(sender, &msg).await;
        }
    }
}

async fn handle_set_bankrupt(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let Some(symbol) = payload.get("symbol").and_then(|v| v.as_str()) {
        match state.admin.set_company_bankrupt(symbol, true).await {
            Ok(_) => {
                state.event_log.log_company_bankrupt(symbol);
                let msg = ServerMessage::System {
                    message: format!("Company {} marked as bankrupt", symbol),
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

async fn handle_ban_trader(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let (Some(target_user_id), Some(banned)) = (
        payload.get("user_id").and_then(|v| v.as_u64()),
        payload.get("banned").and_then(|v| v.as_bool()),
    ) {
        match state.admin.set_trader_banned(target_user_id, banned).await {
            Ok(_) => {
                if banned {
                    state
                        .event_log
                        .log_trader_banned(target_user_id, "Admin action");
                } else {
                    state.event_log.log_trader_unbanned(target_user_id);
                }

                let action = if banned { "banned" } else { "unbanned" };
                let msg = ServerMessage::System {
                    message: format!("Trader {} {}", target_user_id, action),
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

async fn handle_mute_trader(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let (Some(target_user_id), Some(muted)) = (
        payload.get("user_id").and_then(|v| v.as_u64()),
        payload.get("muted").and_then(|v| v.as_bool()),
    ) {
        match state.admin.set_trader_chat(target_user_id, !muted).await {
            Ok(_) => {
                if muted {
                    state.event_log.log_trader_chat_muted(target_user_id);
                } else {
                    state.event_log.log_trader_chat_unmuted(target_user_id);
                }

                let action = if muted { "muted" } else { "unmuted" };
                let msg = ServerMessage::System {
                    message: format!("Trader {} chat {}", target_user_id, action),
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

async fn handle_get_all_trades(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    let user_id_filter = payload.get("user_id").and_then(|v| v.as_u64());
    let symbol_filter = payload.get("symbol").and_then(|v| v.as_str());
    let page = payload.get("page").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let page_size = payload
        .get("page_size")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as u32;

    let (trades, total_count, has_more) =
        state
            .trade_history
            .get_all_trades_admin(user_id_filter, symbol_filter, page, page_size);

    let msg = ServerMessage::AdminTradeHistory {
        trades,
        total_count,
        page,
        page_size,
        has_more,
    };
    send_message(sender, &msg).await;
}

async fn handle_get_all_open_orders(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    let symbol_filter = payload.get("symbol").and_then(|v| v.as_str());

    let mut user_names = HashMap::new();
    if let Ok(users) = state.user_repo.all().await {
        for user in users {
            user_names.insert(user.id, user.name.clone());
        }
    }

    let orders = state
        .orders
        .get_all_orders_admin(symbol_filter, &user_names);
    let total_count = orders.len();

    let msg = ServerMessage::AdminOpenOrders {
        orders,
        total_count,
    };
    send_message(sender, &msg).await;
}

async fn handle_get_orderbook(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    payload: &serde_json::Value,
) {
    if let Some(symbol) = payload.get("symbol").and_then(|v| v.as_str()) {
        let mut user_names = HashMap::new();
        if let Ok(users) = state.user_repo.all().await {
            for user in users {
                user_names.insert(user.id, user.name.clone());
            }
        }

        let orders = state.orders.get_all_orders_admin(Some(symbol), &user_names);
        let (bids, asks): (Vec<_>, Vec<_>) = orders
            .into_iter()
            .partition(|o| matches!(o.side, OrderSide::Buy));

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

async fn handle_get_dashboard_metrics(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
) {
    use crate::domain::ui_models::ActiveSessionInfo;

    let users = state.user_repo.all().await.unwrap_or_default();
    let total_traders = state.user_repo.count().await.unwrap_or(users.len());
    let active_traders = state.sessions.active_session_count();
    let total_trades = state.trade_history.get_total_trade_count();
    let total_volume = state.trade_history.get_total_volume();
    let recent_volume = state.trade_history.get_recent_volume(300);
    let halted_symbols_count = state.market.get_halted_symbols().len();
    let open_orders_count = state.orders.get_total_open_orders_count();
    let market_open = state.engine.is_market_open();

    let mut total_market_cap: i64 = 0;
    for user in &users {
        total_market_cap += calculate_net_worth(user, &state.market);
    }

    // Create a user ID to name lookup
    let user_names: HashMap<u64, String> = users.iter().map(|u| (u.id, u.name.clone())).collect();

    // Build active sessions list with real data
    let all_sessions = state.sessions.get_all_sessions();
    let active_sessions: Vec<ActiveSessionInfo> = all_sessions
        .iter()
        .map(|session| {
            ActiveSessionInfo {
                session_id: session.session_id,
                user_id: session.user_id,
                user_name: user_names
                    .get(&session.user_id)
                    .cloned()
                    .unwrap_or_else(|| format!("User {}", session.user_id)),
                connected_at: session.connected_at,
                last_activity: session.last_activity,
                messages_sent: 0, // TODO: Track message count per session
            }
        })
        .collect();

    // Calculate server uptime
    let now = chrono::Utc::now().timestamp();
    let server_uptime_secs = (now - state.server_start_time).max(0) as u64;

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
        timestamp: now,
        server_uptime_secs,
        active_sessions,
    };

    let msg = ServerMessage::AdminDashboardMetrics { metrics };
    send_message(sender, &msg).await;
}
