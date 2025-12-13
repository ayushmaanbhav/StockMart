//! State synchronization handlers for WebSocket connections.
//!
//! Handles full state sync and component-specific sync requests.

use axum::extract::ws::{Message, WebSocket};
use std::sync::Arc;
use tracing::debug;

use crate::api::ws::AppState;
use crate::domain::error::UserError;
use crate::domain::models::User;
use crate::domain::ui_models::{
    CandleUI, CompanyUI, FullStateSyncPayload, NewsItemUI, OrderbookLevelUI, OrderbookUI,
    PortfolioItemUI, PortfolioStateUI,
};
use crate::infrastructure::id_generator::IdGenerators;
use crate::presentation::websocket::messages::ServerMessage;
use crate::service::market::MarketService;

use super::send_message;

/// Handle state sync request
pub async fn handle_request_sync(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    subscribed_symbols: &[String],
    component: Option<String>,
) {
    let sync_id = IdGenerators::global().next_sync_id();

    match component.as_deref() {
        None => {
            send_full_state_sync(
                sender,
                state,
                user_id,
                subscribed_symbols.first().map(|s| s.as_str()),
            )
            .await;
        }
        Some("portfolio") => {
            sync_portfolio(sender, state, user_id, sync_id).await;
        }
        Some("orders") => {
            sync_open_orders(sender, state, user_id, sync_id).await;
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
            let news = state
                .news
                .get_recent(20)
                .into_iter()
                .map(|n| NewsItemUI {
                    id: n.id.clone(),
                    headline: n.headline.clone(),
                    symbol: n.symbol.clone(),
                    sentiment: n.sentiment.clone(),
                    impact: n.impact.clone(),
                    timestamp: n.timestamp,
                })
                .collect();
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
            sync_orderbook(sender, state, symbol, sync_id).await;
        }
        Some(comp) if comp.starts_with("candles:") => {
            let symbol = &comp[8..];
            sync_candles(sender, state, symbol, sync_id).await;
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
            sync_trade_history(sender, state, user_id, sync_id).await;
        }
        _ => {
            let msg = ServerMessage::error("UNKNOWN_COMPONENT", "Unknown sync component");
            send_message(sender, &msg).await;
        }
    }
}

async fn sync_portfolio(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    sync_id: u64,
) {
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
        let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
        send_message(sender, &msg).await;
    }
}

async fn sync_open_orders(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    sync_id: u64,
) {
    if let Some(uid) = user_id {
        let orders = state.orders.get_user_orders(uid);
        let msg = ServerMessage::OpenOrdersSync { sync_id, orders };
        send_message(sender, &msg).await;
    }
}

async fn sync_orderbook(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
    sync_id: u64,
) {
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
            bids: bids
                .into_iter()
                .scan(0u64, |cum, (price, qty)| {
                    *cum += qty;
                    Some(OrderbookLevelUI {
                        price,
                        qty,
                        order_count: 1,
                        cumulative_qty: *cum,
                    })
                })
                .collect(),
            asks: asks
                .into_iter()
                .scan(0u64, |cum, (price, qty)| {
                    *cum += qty;
                    Some(OrderbookLevelUI {
                        price,
                        qty,
                        order_count: 1,
                        cumulative_qty: *cum,
                    })
                })
                .collect(),
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

async fn sync_candles(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    symbol: &str,
    sync_id: u64,
) {
    let candles: Vec<CandleUI> = state
        .market
        .get_candles(symbol)
        .into_iter()
        .map(|c| CandleUI {
            timestamp: c.timestamp,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        })
        .collect();
    let msg = ServerMessage::CandlesSync {
        sync_id,
        symbol: symbol.to_string(),
        candles,
    };
    send_message(sender, &msg).await;
}

async fn sync_trade_history(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    _sync_id: u64,
) {
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
        let msg = ServerMessage::from_user_error(UserError::NotAuthenticated);
        send_message(sender, &msg).await;
    }
}

/// Send full state sync to client
pub async fn send_full_state_sync(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: Option<u64>,
    active_symbol: Option<&str>,
) {
    let sync_id = IdGenerators::global().next_sync_id();
    let timestamp = chrono::Utc::now().timestamp();

    // Market state
    let market_open = state.engine.is_market_open();
    let halted_symbols = state.market.get_halted_symbols();

    // Companies with current prices
    let companies: Vec<CompanyUI> = if let Ok(companies) = state.company_repo.all().await {
        companies
            .iter()
            .map(|c| {
                let candles = state.market.get_candles(&c.symbol);
                let (current_price, price_change, price_change_percent) =
                    if let Some(last) = candles.last() {
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
            })
            .collect()
    } else {
        vec![]
    };

    // User-specific state
    let (portfolio, open_orders) = if let Some(uid) = user_id {
        let user = state.user_repo.find_by_id(uid).await.ok().flatten();
        let portfolio = user
            .as_ref()
            .map(|u| compute_portfolio_ui(u, &state.market));
        let orders = state.orders.get_user_orders(uid);
        (portfolio, orders)
    } else {
        (None, vec![])
    };

    // Market data
    let indices = state.indices.get_all_indices();
    let leaderboard = state.leaderboard.get_current();
    let news = state
        .news
        .get_recent(20)
        .into_iter()
        .map(|n| NewsItemUI {
            id: n.id.clone(),
            headline: n.headline.clone(),
            symbol: n.symbol.clone(),
            sentiment: n.sentiment.clone(),
            impact: n.impact.clone(),
            timestamp: n.timestamp,
        })
        .collect();
    let chat_history = state.chat.get_recent(50);

    // Symbol-specific data
    let (orderbook, candles, recent_trades) = if let Some(sym) = active_symbol {
        let ob = state
            .engine
            .get_order_book_depth(sym, 10)
            .map(|(bids, asks)| {
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
                    bids: bids
                        .into_iter()
                        .scan(0u64, |cum, (price, qty)| {
                            *cum += qty;
                            Some(OrderbookLevelUI {
                                price,
                                qty,
                                order_count: 1,
                                cumulative_qty: *cum,
                            })
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .scan(0u64, |cum, (price, qty)| {
                            *cum += qty;
                            Some(OrderbookLevelUI {
                                price,
                                qty,
                                order_count: 1,
                                cumulative_qty: *cum,
                            })
                        })
                        .collect(),
                    spread,
                    spread_percent,
                    last_price,
                    timestamp,
                }
            });
        let candles_data: Vec<CandleUI> = state
            .market
            .get_candles(sym)
            .into_iter()
            .map(|c| CandleUI {
                timestamp: c.timestamp,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume,
            })
            .collect();
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

/// Compute UI-ready portfolio from user data
pub fn compute_portfolio_ui(user: &User, market: &MarketService) -> PortfolioStateUI {
    let mut items: Vec<PortfolioItemUI> = Vec::new();
    let mut portfolio_value: i64 = 0;

    for item in &user.portfolio {
        let current_price = market
            .get_last_price(&item.symbol)
            .unwrap_or(item.average_buy_price);

        let market_value = (item.qty as i64) * current_price;
        let cost_basis = (item.qty as i64) * item.average_buy_price;
        let unrealized_pnl = market_value - cost_basis;
        let unrealized_pnl_percent = if cost_basis != 0 {
            (unrealized_pnl as f64 / cost_basis as f64) * 100.0
        } else {
            0.0
        };

        let short_market_value = (item.short_qty as i64) * current_price;
        let short_unrealized_pnl = 0; // Would need short entry price to calculate

        portfolio_value += market_value;
        portfolio_value -= short_market_value;

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
