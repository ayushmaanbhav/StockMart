//! Trade history service for tracking and querying executed trades.

use crate::domain::models::{OrderSide, Trade, UserId};
use crate::domain::ui_models::{TradeHistoryItem, TradeHistoryResponse};
use dashmap::DashMap;
use std::sync::RwLock;

/// Extended trade record with additional context for history display
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub trade: Trade,
    pub buyer_name: String,
    pub seller_name: String,
    pub buyer_side: String,  // "Buy" or "Cover" (covering a short)
    pub seller_side: String, // "Sell" or "Short"
}

/// Service for storing and querying trade history.
/// Keeps all trades indefinitely for the game session.
pub struct TradeHistoryService {
    /// All trades in chronological order
    trades: RwLock<Vec<TradeRecord>>,
    /// user_id -> Vec<trade_index> for user's trades
    user_trades: DashMap<UserId, Vec<usize>>,
    /// symbol -> Vec<trade_index> for symbol trades
    symbol_trades: DashMap<String, Vec<usize>>,
}

impl TradeHistoryService {
    pub fn new() -> Self {
        Self {
            trades: RwLock::new(Vec::new()),
            user_trades: DashMap::new(),
            symbol_trades: DashMap::new(),
        }
    }

    /// Record a new trade
    pub fn record_trade(
        &self,
        trade: Trade,
        buyer_name: String,
        seller_name: String,
        buyer_side: OrderSide,
        seller_side: OrderSide,
    ) {
        let buyer_side_str = match buyer_side {
            OrderSide::Buy => "Buy".to_string(),
            OrderSide::Short => "Cover".to_string(), // Covering a short
            OrderSide::Sell => "Buy".to_string(),    // Shouldn't happen but fallback
        };

        let seller_side_str = match seller_side {
            OrderSide::Sell => "Sell".to_string(),
            OrderSide::Short => "Short".to_string(),
            OrderSide::Buy => "Sell".to_string(), // Shouldn't happen but fallback
        };

        let record = TradeRecord {
            trade: trade.clone(),
            buyer_name,
            seller_name,
            buyer_side: buyer_side_str,
            seller_side: seller_side_str,
        };

        let mut trades = self.trades.write().unwrap();
        let index = trades.len();

        // Index by buyer
        self.user_trades
            .entry(trade.taker_user_id)
            .or_insert_with(Vec::new)
            .push(index);

        // Index by seller (maker)
        if trade.maker_user_id != trade.taker_user_id {
            self.user_trades
                .entry(trade.maker_user_id)
                .or_insert_with(Vec::new)
                .push(index);
        }

        // Index by symbol
        self.symbol_trades
            .entry(trade.symbol.clone())
            .or_insert_with(Vec::new)
            .push(index);

        trades.push(record);
    }

    /// Get trades for a specific user with pagination
    pub fn get_user_trades(
        &self,
        user_id: UserId,
        page: u32,
        page_size: u32,
    ) -> TradeHistoryResponse {
        let trades = self.trades.read().unwrap();
        let indices = self.user_trades.get(&user_id);

        match indices {
            Some(indices) => {
                let total_count = indices.len() as u64;
                let start = (page * page_size) as usize;
                let end = std::cmp::min(start + page_size as usize, indices.len());

                if start >= indices.len() {
                    return TradeHistoryResponse {
                        trades: vec![],
                        total_count,
                        page,
                        page_size,
                        has_more: false,
                    };
                }

                // Get trades in reverse order (newest first)
                let mut items: Vec<TradeHistoryItem> = indices[start..end]
                    .iter()
                    .rev()
                    .filter_map(|&idx| trades.get(idx))
                    .map(|record| self.record_to_item(record, user_id))
                    .collect();

                // Actually reverse to get newest first properly
                items.reverse();

                TradeHistoryResponse {
                    trades: items,
                    total_count,
                    page,
                    page_size,
                    has_more: end < indices.len(),
                }
            }
            None => TradeHistoryResponse {
                trades: vec![],
                total_count: 0,
                page,
                page_size,
                has_more: false,
            },
        }
    }

    /// Get trades for a specific symbol (for orderbook tab)
    pub fn get_symbol_trades(&self, symbol: &str, count: usize) -> Vec<TradeHistoryItem> {
        let trades = self.trades.read().unwrap();
        let indices = self.symbol_trades.get(symbol);

        match indices {
            Some(indices) => {
                // Get most recent trades
                indices
                    .iter()
                    .rev()
                    .take(count)
                    .filter_map(|&idx| trades.get(idx))
                    .map(|record| self.record_to_item_public(record))
                    .collect()
            }
            None => vec![],
        }
    }

    /// Get all trades with optional filters (for admin)
    #[allow(dead_code)] // API method - use get_all_trades_admin for admin panel
    pub fn get_all_trades(
        &self,
        user_id_filter: Option<UserId>,
        symbol_filter: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> TradeHistoryResponse {
        let trades = self.trades.read().unwrap();

        // Apply filters
        let filtered: Vec<(usize, &TradeRecord)> = trades
            .iter()
            .enumerate()
            .filter(|(_, record)| {
                let user_match = user_id_filter
                    .map(|uid| {
                        record.trade.maker_user_id == uid || record.trade.taker_user_id == uid
                    })
                    .unwrap_or(true);

                let symbol_match = symbol_filter
                    .map(|sym| record.trade.symbol == sym)
                    .unwrap_or(true);

                user_match && symbol_match
            })
            .collect();

        let total_count = filtered.len() as u64;
        let start = (page * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, filtered.len());

        if start >= filtered.len() {
            return TradeHistoryResponse {
                trades: vec![],
                total_count,
                page,
                page_size,
                has_more: false,
            };
        }

        // Get page in reverse order (newest first)
        let items: Vec<TradeHistoryItem> = filtered
            .into_iter()
            .rev()
            .skip(start)
            .take(page_size as usize)
            .map(|(_, record)| self.record_to_item_admin(record))
            .collect();

        TradeHistoryResponse {
            trades: items,
            total_count,
            page,
            page_size,
            has_more: end < total_count as usize,
        }
    }

    /// Get total trade count for a user
    #[allow(dead_code)] // API method for user statistics
    pub fn get_user_trade_count(&self, user_id: UserId) -> u64 {
        self.user_trades
            .get(&user_id)
            .map(|indices| indices.len() as u64)
            .unwrap_or(0)
    }

    /// Get recent trades for a symbol (limited count)
    #[allow(dead_code)] // API alias for get_symbol_trades
    pub fn get_recent_symbol_trades(&self, symbol: &str, count: usize) -> Vec<TradeHistoryItem> {
        self.get_symbol_trades(symbol, count)
    }

    /// Clear all trade history (for game reset)
    pub fn clear_all(&self) {
        let mut trades = self.trades.write().unwrap();
        trades.clear();
        self.user_trades.clear();
        self.symbol_trades.clear();
    }

    /// Convert record to TradeHistoryItem for a specific user's perspective
    fn record_to_item(&self, record: &TradeRecord, user_id: UserId) -> TradeHistoryItem {
        let is_buyer = record.trade.taker_user_id == user_id;

        TradeHistoryItem {
            trade_id: record.trade.id,
            symbol: record.trade.symbol.clone(),
            side: if is_buyer {
                record.buyer_side.clone()
            } else {
                record.seller_side.clone()
            },
            qty: record.trade.qty,
            price: record.trade.price,
            total_value: (record.trade.qty as i64) * record.trade.price,
            counterparty_id: None, // Don't expose counterparty to traders
            counterparty_name: None,
            timestamp: record.trade.timestamp,
        }
    }

    /// Convert record to TradeHistoryItem for public view (symbol trades)
    fn record_to_item_public(&self, record: &TradeRecord) -> TradeHistoryItem {
        TradeHistoryItem {
            trade_id: record.trade.id,
            symbol: record.trade.symbol.clone(),
            side: record.buyer_side.clone(), // Show as "Buy" for public
            qty: record.trade.qty,
            price: record.trade.price,
            total_value: (record.trade.qty as i64) * record.trade.price,
            counterparty_id: None,
            counterparty_name: None,
            timestamp: record.trade.timestamp,
        }
    }

    /// Convert record to TradeHistoryItem for admin view (shows both parties)
    fn record_to_item_admin(&self, record: &TradeRecord) -> TradeHistoryItem {
        TradeHistoryItem {
            trade_id: record.trade.id,
            symbol: record.trade.symbol.clone(),
            side: format!("{} / {}", record.buyer_side, record.seller_side),
            qty: record.trade.qty,
            price: record.trade.price,
            total_value: (record.trade.qty as i64) * record.trade.price,
            counterparty_id: Some(record.trade.maker_user_id), // For linking
            counterparty_name: Some(format!("{} <-> {}", record.buyer_name, record.seller_name)),
            timestamp: record.trade.timestamp,
        }
    }

    /// Get total trade count across all trades
    pub fn get_total_trade_count(&self) -> u64 {
        self.trades.read().unwrap().len() as u64
    }

    /// Get total volume across all trades
    pub fn get_total_volume(&self) -> i64 {
        self.trades
            .read()
            .unwrap()
            .iter()
            .map(|r| (r.trade.qty as i64) * r.trade.price)
            .sum()
    }

    /// Get recent volume (trades within last N seconds)
    pub fn get_recent_volume(&self, seconds: i64) -> i64 {
        let cutoff = chrono::Utc::now().timestamp() - seconds;
        self.trades
            .read()
            .unwrap()
            .iter()
            .filter(|r| r.trade.timestamp >= cutoff)
            .map(|r| (r.trade.qty as i64) * r.trade.price)
            .sum()
    }

    /// Get all trades for admin (extended version with both parties)
    pub fn get_all_trades_admin(
        &self,
        user_id_filter: Option<UserId>,
        symbol_filter: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> (
        Vec<crate::domain::ui_models::AdminTradeHistoryItem>,
        u64,
        bool,
    ) {
        let trades = self.trades.read().unwrap();

        // Apply filters
        let filtered: Vec<&TradeRecord> = trades
            .iter()
            .filter(|record| {
                let user_match = user_id_filter
                    .map(|uid| {
                        record.trade.maker_user_id == uid || record.trade.taker_user_id == uid
                    })
                    .unwrap_or(true);

                let symbol_match = symbol_filter
                    .map(|sym| record.trade.symbol == sym)
                    .unwrap_or(true);

                user_match && symbol_match
            })
            .collect();

        let total_count = filtered.len() as u64;
        let start = (page * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, filtered.len());

        if start >= filtered.len() {
            return (vec![], total_count, false);
        }

        // Get page in reverse order (newest first)
        let items: Vec<crate::domain::ui_models::AdminTradeHistoryItem> = filtered
            .into_iter()
            .rev()
            .skip(start)
            .take(page_size as usize)
            .map(|record| crate::domain::ui_models::AdminTradeHistoryItem {
                trade_id: record.trade.id,
                symbol: record.trade.symbol.clone(),
                buyer_id: record.trade.taker_user_id,
                buyer_name: record.buyer_name.clone(),
                buyer_side: record.buyer_side.clone(),
                seller_id: record.trade.maker_user_id,
                seller_name: record.seller_name.clone(),
                seller_side: record.seller_side.clone(),
                qty: record.trade.qty,
                price: record.trade.price,
                total_value: (record.trade.qty as i64) * record.trade.price,
                timestamp: record.trade.timestamp,
            })
            .collect();

        let has_more = end < total_count as usize;
        (items, total_count, has_more)
    }
}

impl Default for TradeHistoryService {
    fn default() -> Self {
        Self::new()
    }
}
