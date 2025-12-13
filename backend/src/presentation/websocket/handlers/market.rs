//! Market data handlers for WebSocket connections.
//!
//! Handles market subscriptions, depth queries, and trade history.

use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;

use crate::api::ws::AppState;
use crate::domain::error::{MarketError, UserError};
use crate::presentation::websocket::messages::ServerMessage;

use super::send_message;

/// Handle market subscription for a symbol
pub async fn handle_subscribe(
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
            candle,
        };
        send_message(sender, &msg).await;
    }

    // Send current depth
    handle_get_depth(sender, state, symbol, 10).await;
}

/// Handle order book depth query
pub async fn handle_get_depth(
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
            let msg = ServerMessage::from_market_error(MarketError::CompanyNotFound {
                symbol: symbol.to_string(),
            });
            send_message(sender, &msg).await;
        }
    }
}

/// Handle user trade history request
pub async fn handle_get_trade_history(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    page: Option<u32>,
    page_size: Option<u32>,
    _symbol: Option<String>,
) {
    if let Some(uid) = user_id {
        let response =
            state
                .trade_history
                .get_user_trades(uid, page.unwrap_or(0), page_size.unwrap_or(20));

        let msg = ServerMessage::TradeHistory {
            trades: response.trades,
            total_count: response.total_count,
            page: response.page,
            page_size: response.page_size,
            has_more: response.has_more,
        };
        send_message(sender, &msg).await;
    } else {
        let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
        send_message(sender, &msg).await;
    }
}

/// Handle stock trade history request (for orderbook tab)
pub async fn handle_get_stock_trades(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
    count: Option<usize>,
) {
    let trades = state
        .trade_history
        .get_symbol_trades(symbol, count.unwrap_or(50));
    let msg = ServerMessage::StockTradeHistory {
        symbol: symbol.to_string(),
        trades,
    };
    send_message(sender, &msg).await;
}
