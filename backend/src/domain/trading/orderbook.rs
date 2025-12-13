//! Order book implementation for price-time priority matching.

use crate::domain::common::types::{Price, Quantity};
use crate::domain::trading::{Order, OrderSide, OrderStatus, OrderType, TimeInForce, Trade};
use crate::infrastructure::id_generator::IdGenerators;
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    /// Bids: Buy orders, BTreeMap gives ASC order, we use iter().rev() for DESC (highest first)
    pub bids: BTreeMap<Price, VecDeque<Order>>,
    /// Asks: Sell orders, BTreeMap gives ASC order (lowest first) which is what we want
    pub asks: BTreeMap<Price, VecDeque<Order>>,
    /// Fast lookup for cancellation: order_id -> (side, price)
    pub order_index: HashMap<u64, (OrderSide, Price)>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            order_index: HashMap::new(),
        }
    }

    /// Get order book depth (top N price levels for bids and asks)
    /// Returns (bids, asks) where each is a vec of (price, total_qty)
    pub fn get_depth(&self, levels: usize) -> (Vec<(Price, Quantity)>, Vec<(Price, Quantity)>) {
        // Bids: highest price first
        let bids: Vec<(Price, Quantity)> = self
            .bids
            .iter()
            .rev()
            .take(levels)
            .map(|(price, orders)| {
                let total_qty: Quantity = orders.iter().map(|o| o.qty - o.filled_qty).sum();
                (*price, total_qty)
            })
            .collect();

        // Asks: lowest price first
        let asks: Vec<(Price, Quantity)> = self
            .asks
            .iter()
            .take(levels)
            .map(|(price, orders)| {
                let total_qty: Quantity = orders.iter().map(|o| o.qty - o.filled_qty).sum();
                (*price, total_qty)
            })
            .collect();

        (bids, asks)
    }

    /// Get best bid price
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.iter().next_back().map(|(p, _)| *p)
    }

    /// Get best ask price
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.iter().next().map(|(p, _)| *p)
    }

    /// Get spread
    #[allow(dead_code)] // Market data API - spread calculated inline from depth data
    pub fn spread(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Add an order to the book with matching, respecting TimeInForce
    pub fn add_order(
        &mut self,
        mut order: Order,
        time_in_force: TimeInForce,
    ) -> (Order, Vec<Trade>) {
        let mut trades = Vec::new();

        // 1. Try to match immediately
        if order.side == OrderSide::Buy {
            self.match_buy_order(&mut order, &mut trades);
        } else {
            // Both Sell and Short match against bids
            self.match_sell_order(&mut order, &mut trades);
        }

        // 2. Handle remaining quantity based on TimeInForce
        if order.status != OrderStatus::Filled {
            match time_in_force {
                TimeInForce::IOC => {
                    // Immediate or Cancel: don't add to book, mark as cancelled
                    order.status = OrderStatus::Cancelled;
                    tracing::debug!(
                        "IOC order {} cancelled: {} of {} filled",
                        order.id,
                        order.filled_qty,
                        order.qty
                    );
                }
                TimeInForce::GTC => {
                    // Good Till Cancelled: add remaining to book
                    self.insert_order(order.clone());
                    tracing::debug!(
                        "GTC order {} added to book: {} remaining at {}",
                        order.id,
                        order.qty - order.filled_qty,
                        order.price
                    );
                }
            }
        }

        (order, trades)
    }

    fn match_buy_order(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // Match against Asks (Lowest Price First)
        loop {
            if order.status == OrderStatus::Filled {
                break;
            }

            // Find the best ask
            let best_ask_price = match self.asks.iter().next() {
                Some((price, _)) => *price,
                None => break, // No asks available
            };

            // Check price condition for limit orders
            if order.order_type == OrderType::Limit && order.price < best_ask_price {
                break; // Best ask is too expensive
            }

            // We have a match! Process the price level
            let mut asks_at_price = self.asks.remove(&best_ask_price).unwrap();

            while let Some(mut ask) = asks_at_price.pop_front() {
                let remaining_order = order.qty - order.filled_qty;
                let remaining_ask = ask.qty - ask.filled_qty;
                let trade_qty = std::cmp::min(remaining_order, remaining_ask);
                let trade_price = best_ask_price; // Trade at resting order's price

                // Create Trade
                let trade = Trade {
                    id: IdGenerators::global().next_trade_id(),
                    maker_order_id: ask.id,
                    taker_order_id: order.id,
                    maker_user_id: ask.user_id,
                    taker_user_id: order.user_id,
                    symbol: self.symbol.clone(),
                    qty: trade_qty,
                    price: trade_price,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                trades.push(trade);

                // Update Order fills
                order.filled_qty += trade_qty;
                ask.filled_qty += trade_qty;

                // Update Order statuses
                order.status = if order.filled_qty == order.qty {
                    OrderStatus::Filled
                } else {
                    OrderStatus::Partial
                };

                if ask.filled_qty == ask.qty {
                    ask.status = OrderStatus::Filled;
                    self.order_index.remove(&ask.id);
                } else {
                    ask.status = OrderStatus::Partial;
                    // Push back to front (preserve time priority)
                    asks_at_price.push_front(ask);
                    break;
                }

                if order.status == OrderStatus::Filled {
                    break;
                }
            }

            // Put remaining orders back at this price level
            if !asks_at_price.is_empty() {
                self.asks.insert(best_ask_price, asks_at_price);
            }
        }
    }

    fn match_sell_order(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // Match against Bids (Highest Price First)
        loop {
            if order.status == OrderStatus::Filled {
                break;
            }

            // Find the best bid (Highest price)
            let best_bid_price = match self.bids.iter().next_back() {
                Some((price, _)) => *price,
                None => break, // No bids available
            };

            // Check price condition for limit orders
            if order.order_type == OrderType::Limit && order.price > best_bid_price {
                break; // Best bid is too low
            }

            // We have a match! Process the price level
            let mut bids_at_price = self.bids.remove(&best_bid_price).unwrap();

            while let Some(mut bid) = bids_at_price.pop_front() {
                let remaining_order = order.qty - order.filled_qty;
                let remaining_bid = bid.qty - bid.filled_qty;
                let trade_qty = std::cmp::min(remaining_order, remaining_bid);
                let trade_price = best_bid_price; // Trade at resting order's price

                // Create Trade
                let trade = Trade {
                    id: IdGenerators::global().next_trade_id(),
                    maker_order_id: bid.id,
                    taker_order_id: order.id,
                    maker_user_id: bid.user_id,
                    taker_user_id: order.user_id,
                    symbol: self.symbol.clone(),
                    qty: trade_qty,
                    price: trade_price,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                trades.push(trade);

                // Update Order fills
                order.filled_qty += trade_qty;
                bid.filled_qty += trade_qty;

                // Update Order statuses
                order.status = if order.filled_qty == order.qty {
                    OrderStatus::Filled
                } else {
                    OrderStatus::Partial
                };

                if bid.filled_qty == bid.qty {
                    bid.status = OrderStatus::Filled;
                    self.order_index.remove(&bid.id);
                } else {
                    bid.status = OrderStatus::Partial;
                    // Push back to front (preserve time priority)
                    bids_at_price.push_front(bid);
                    break;
                }

                if order.status == OrderStatus::Filled {
                    break;
                }
            }

            // Put remaining orders back at this price level
            if !bids_at_price.is_empty() {
                self.bids.insert(best_bid_price, bids_at_price);
            }
        }
    }

    fn insert_order(&mut self, order: Order) {
        self.order_index.insert(order.id, (order.side, order.price));

        let side_map = if order.side == OrderSide::Buy {
            &mut self.bids
        } else {
            // Both Sell and Short go on the ask side
            &mut self.asks
        };

        side_map
            .entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    /// Cancel an order by ID, returns the cancelled order if found
    pub fn cancel_order(&mut self, order_id: u64) -> Option<Order> {
        if let Some((side, price)) = self.order_index.remove(&order_id) {
            let side_map = if side == OrderSide::Buy {
                &mut self.bids
            } else {
                &mut self.asks
            };

            if let Some(queue) = side_map.get_mut(&price) {
                // Find and remove the order
                if let Some(idx) = queue.iter().position(|o| o.id == order_id) {
                    let mut order = queue.remove(idx).unwrap();
                    order.status = OrderStatus::Cancelled;

                    // Cleanup empty price levels
                    if queue.is_empty() {
                        side_map.remove(&price);
                    }

                    return Some(order);
                }
            }
        }
        None
    }

    /// Get total volume at all price levels
    #[allow(dead_code)] // Market data API for volume statistics
    pub fn total_volume(&self) -> (Quantity, Quantity) {
        let bid_volume: Quantity = self
            .bids
            .values()
            .flat_map(|orders| orders.iter())
            .map(|o| o.qty - o.filled_qty)
            .sum();

        let ask_volume: Quantity = self
            .asks
            .values()
            .flat_map(|orders| orders.iter())
            .map(|o| o.qty - o.filled_qty)
            .sum();

        (bid_volume, ask_volume)
    }

    /// Clear all orders from the book (for game reset)
    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.order_index.clear();
    }

    /// Seed an order directly into the book without matching (for initial game state)
    /// Used by admin to create initial liquidity
    pub fn seed_order(&mut self, order: Order) {
        self.order_index.insert(order.id, (order.side, order.price));

        let side_map = if order.side == OrderSide::Buy {
            &mut self.bids
        } else {
            // Both Sell and Short go on the ask side
            &mut self.asks
        };

        side_map
            .entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }
}
