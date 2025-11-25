use std::collections::{BTreeMap, HashMap, VecDeque};
use crate::domain::models::{Order, OrderSide, OrderStatus, OrderType, Trade, next_trade_id, Price, Quantity};

#[derive(Debug)]
pub struct OrderBook {
    pub symbol: String,
    // Bids: Buy orders, sorted by Price DESC. 
    // We use BTreeMap<Price, VecDeque<Order>>. 
    // Since BTreeMap sorts by Key ASC, we need to reverse the iterator when matching bids (highest price first).
    // Actually, for Bids, we want Highest Price first. 
    // For Asks, we want Lowest Price first.
    pub bids: BTreeMap<Price, VecDeque<Order>>, 
    pub asks: BTreeMap<Price, VecDeque<Order>>,
    
    // Fast lookup for cancellation
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

    pub fn add_order(&mut self, mut order: Order) -> (Order, Vec<Trade>) {
        let mut trades = Vec::new();

        // 1. Try to match immediately
        if order.side == OrderSide::Buy {
            self.match_buy_order(&mut order, &mut trades);
        } else {
            self.match_sell_order(&mut order, &mut trades);
        }

        // 2. If not fully filled, add to book (unless IOC)
        if order.status != OrderStatus::Filled && order.status != OrderStatus::Cancelled {
            // TODO: Handle IOC (Immediate Or Cancel) - if not filled, cancel the rest
            
            self.insert_order(order.clone());
        }

        (order, trades)
    }

    fn match_buy_order(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // Match against Asks (Lowest Price First)
        // BTreeMap iter gives lowest keys first, which is exactly what we want for Asks (Sell orders).
        
        // We need to iterate and mutate. This is tricky with BTreeMap. 
        // We'll loop while the order is active and we have matching asks.
        
        loop {
            if order.status == OrderStatus::Filled {
                break;
            }

            // Find the best ask
            let best_ask_price = if let Some((price, _)) = self.asks.iter().next() {
                *price
            } else {
                break; // No asks
            };

            // Check price condition
            if order.order_type == OrderType::Limit && order.price < best_ask_price {
                break; // Best ask is too expensive
            }

            // We have a match!
            let mut asks_at_price = self.asks.remove(&best_ask_price).unwrap();
            
            while let Some(mut ask) = asks_at_price.pop_front() {
                let trade_qty = std::cmp::min(order.qty - order.filled_qty, ask.qty - ask.filled_qty);
                let trade_price = best_ask_price; // Trade happens at the resting order's price

                // Create Trade
                let trade = Trade {
                    id: next_trade_id(),
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

                // Update Orders
                order.filled_qty += trade_qty;
                ask.filled_qty += trade_qty;

                if order.filled_qty == order.qty {
                    order.status = OrderStatus::Filled;
                } else {
                    order.status = OrderStatus::Partial;
                }

                if ask.filled_qty == ask.qty {
                    ask.status = OrderStatus::Filled;
                    self.order_index.remove(&ask.id);
                } else {
                    ask.status = OrderStatus::Partial;
                    // Push back to front? No, it stays at front if we are processing a queue.
                    // But we popped it. If it's not filled, we need to put it back at the FRONT.
                    // Actually, Price-Time priority means we consume the oldest first. 
                    // pop_front gives the oldest. If partially filled, it remains the oldest.
                    // So we should push_front back.
                    asks_at_price.push_front(ask);
                    break; // Order is filled (since trade_qty = min(rem_order, rem_ask))
                           // Wait, if order is filled, we break the inner loop.
                           // If ask is filled, we continue to next ask.
                           // If both filled? loop continues.
                }
                
                if order.status == OrderStatus::Filled {
                    break;
                }
            }

            // If the price level is empty, remove it (already removed above). 
            // If not empty, put it back.
            if !asks_at_price.is_empty() {
                self.asks.insert(best_ask_price, asks_at_price);
            }
        }
    }

    fn match_sell_order(&mut self, order: &mut Order, trades: &mut Vec<Trade>) {
        // Match against Bids (Highest Price First)
        // BTreeMap iter gives lowest keys first. We need highest.
        // We can use iter().next_back() or similar logic.
        
        loop {
            if order.status == OrderStatus::Filled {
                break;
            }

            // Find the best bid (Highest price)
            let best_bid_price = if let Some((price, _)) = self.bids.iter().next_back() {
                *price
            } else {
                break; // No bids
            };

            // Check price condition
            if order.order_type == OrderType::Limit && order.price > best_bid_price {
                break; // Best bid is too low
            }

            // We have a match!
            let mut bids_at_price = self.bids.remove(&best_bid_price).unwrap();
            
            while let Some(mut bid) = bids_at_price.pop_front() {
                let trade_qty = std::cmp::min(order.qty - order.filled_qty, bid.qty - bid.filled_qty);
                let trade_price = best_bid_price; // Trade happens at the resting order's price

                // Create Trade
                let trade = Trade {
                    id: next_trade_id(),
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

                // Update Orders
                order.filled_qty += trade_qty;
                bid.filled_qty += trade_qty;

                if order.filled_qty == order.qty {
                    order.status = OrderStatus::Filled;
                } else {
                    order.status = OrderStatus::Partial;
                }

                if bid.filled_qty == bid.qty {
                    bid.status = OrderStatus::Filled;
                    self.order_index.remove(&bid.id);
                } else {
                    bid.status = OrderStatus::Partial;
                    bids_at_price.push_front(bid);
                    break; 
                }
                
                if order.status == OrderStatus::Filled {
                    break;
                }
            }

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
            &mut self.asks
        };

        side_map.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

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
}
