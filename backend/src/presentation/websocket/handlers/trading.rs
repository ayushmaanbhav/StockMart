//! Trading handlers for WebSocket connections.
//!
//! Handles order placement, cancellation, and portfolio queries.

use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use tracing::info;

use crate::api::ws::AppState;
use crate::domain::error::UserError;
use crate::domain::models::{Order, OrderSide, OrderStatus, OrderType, TimeInForce};
use crate::infrastructure::id_generator::IdGenerators;
use crate::presentation::websocket::messages::ServerMessage;

use super::helpers::calculate_net_worth;
use super::send_message;

/// Handle placing a new order
pub async fn handle_place_order(
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
            let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
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
        id: IdGenerators::global().next_order_id(),
        user_id: uid,
        symbol: symbol.clone(),
        order_type: type_enum,
        side: side_enum,
        qty,
        filled_qty: 0,
        price,
        status: OrderStatus::Open,
        timestamp: chrono::Utc::now().timestamp(),
        time_in_force: tif_enum,
    };

    info!("Placing order: {:?}", order);

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
            info!("Order {} processed: {:?}", processed.id, processed.status);

            // Track open/partial orders in OrdersService
            if matches!(processed.status, OrderStatus::Open | OrderStatus::Partial) {
                state.orders.add_order(processed.clone());
            }

            let msg = ServerMessage::OrderAck {
                order_id: processed.id,
                status: format!("{:?}", processed.status),
                filled_qty: processed.filled_qty,
                remaining_qty: processed.qty - processed.filled_qty,
            };
            send_message(sender, &msg).await;

            // Send updated depth after order placement
            send_depth_update(sender, state, &symbol).await;
        }
        Err(e) => {
            // Log order rejected event
            state
                .event_log
                .log_order_rejected(uid, &symbol, &side, qty, price, &e.to_string());

            // Use typed error code from EngineError
            let msg = ServerMessage::OrderRejected {
                reason: e.to_string(),
                error_code: e.error_code().to_string(),
            };
            send_message(sender, &msg).await;
        }
    }
}

/// Handle order cancellation
pub async fn handle_cancel_order(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    symbol: &str,
    order_id: u64,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
            send_message(sender, &msg).await;
            return;
        }
    };

    match state.engine.cancel_order(uid, symbol, order_id).await {
        Ok(cancelled) => {
            info!("Order {} cancelled", cancelled.id);

            // Remove from OrdersService tracking
            state.orders.remove_order(order_id);

            // Log order cancelled event
            state
                .event_log
                .log_order_cancelled(order_id, uid, symbol, "User requested");

            let msg = ServerMessage::OrderCancelled {
                order_id: cancelled.id,
            };
            send_message(sender, &msg).await;

            // Send updated depth after cancellation
            send_depth_update(sender, state, symbol).await;

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
            // Use typed error code from EngineError
            let msg = ServerMessage::error(e.error_code(), &e.to_string());
            send_message(sender, &msg).await;
        }
    }
}

/// Handle portfolio query
pub async fn handle_get_portfolio(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
) {
    let uid = match user_id {
        Some(id) => id,
        None => {
            let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
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
            let msg = ServerMessage::from_user_error(UserError::NotFound { user_id: uid });
            send_message(sender, &msg).await;
        }
    }
}

/// Send depth update for a symbol
async fn send_depth_update(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
) {
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
}
