use crate::domain::constants::trading::{SHORT_MARGIN_PERCENT, TRADE_CHANNEL_SIZE};
use crate::domain::error::TradingError;
use crate::domain::models::{
    Order, OrderSide, OrderStatus, OrderType, Portfolio, Price, Quantity, TimeInForce, Trade, User,
    PRICE_SCALE,
};
use crate::domain::trading::OrderBook;
use crate::domain::UserRepository;
use crate::service::orders::OrdersService;
use crate::service::trade_history::TradeHistoryService;
use dashmap::DashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Error types for matching engine operations
#[derive(Debug, Clone)]
pub enum EngineError {
    MarketClosed,
    UserNotFound,
    SymbolNotFound,
    InsufficientFunds {
        required: Price,
        available: Price,
    },
    InsufficientShares {
        required: Quantity,
        available: Quantity,
    },
    InsufficientMargin {
        required: Price,
        available: Price,
    },
    OrderNotFound,
    InternalError(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::MarketClosed => write!(f, "Market is currently closed"),
            EngineError::UserNotFound => write!(f, "User not found"),
            EngineError::SymbolNotFound => write!(f, "Trading symbol not found"),
            EngineError::InsufficientFunds {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient funds: need {}, have {}",
                    required, available
                )
            }
            EngineError::InsufficientShares {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient shares: need {}, have {}",
                    required, available
                )
            }
            EngineError::InsufficientMargin {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient margin for short: need {}, have {}",
                    required, available
                )
            }
            EngineError::OrderNotFound => write!(f, "Order not found"),
            EngineError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl EngineError {
    /// Get an error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            EngineError::MarketClosed => "MARKET_CLOSED",
            EngineError::UserNotFound => "USER_NOT_FOUND",
            EngineError::SymbolNotFound => "SYMBOL_NOT_FOUND",
            EngineError::InsufficientFunds { .. } => "INSUFFICIENT_FUNDS",
            EngineError::InsufficientShares { .. } => "INSUFFICIENT_SHARES",
            EngineError::InsufficientMargin { .. } => "INSUFFICIENT_MARGIN",
            EngineError::OrderNotFound => "ORDER_NOT_FOUND",
            EngineError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    /// Convert to TradingError for typed error handling
    pub fn to_trading_error(&self) -> TradingError {
        match self {
            EngineError::MarketClosed => TradingError::MarketClosed,
            EngineError::UserNotFound => TradingError::InvalidOrder {
                reason: "User not found".to_string(),
            },
            EngineError::SymbolNotFound => TradingError::SymbolNotFound {
                symbol: "unknown".to_string(),
            },
            EngineError::InsufficientFunds {
                required,
                available,
            } => TradingError::InsufficientFunds {
                required: *required,
                available: *available,
            },
            EngineError::InsufficientShares {
                required,
                available,
            } => TradingError::InsufficientShares {
                required: *required,
                available: *available,
            },
            EngineError::InsufficientMargin {
                required,
                available,
            } => TradingError::InsufficientMargin {
                required: *required,
                available: *available,
            },
            EngineError::OrderNotFound => TradingError::OrderNotFound { order_id: 0 },
            EngineError::InternalError(msg) => TradingError::InvalidOrder {
                reason: msg.clone(),
            },
        }
    }
}

impl From<EngineError> for String {
    fn from(e: EngineError) -> Self {
        e.to_string()
    }
}

impl From<EngineError> for TradingError {
    fn from(e: EngineError) -> Self {
        e.to_trading_error()
    }
}

pub struct MatchingEngine {
    orderbooks: DashMap<String, OrderBook>,
    user_repo: Arc<dyn UserRepository>,
    orders_service: Arc<OrdersService>,
    trade_history: Arc<TradeHistoryService>,
    trade_sender: broadcast::Sender<Trade>,
    is_open: AtomicBool,
}

impl MatchingEngine {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        orders_service: Arc<OrdersService>,
        trade_history: Arc<TradeHistoryService>,
    ) -> Self {
        let (tx, _) = broadcast::channel(TRADE_CHANNEL_SIZE);
        Self {
            orderbooks: DashMap::new(),
            user_repo,
            orders_service,
            trade_history,
            trade_sender: tx,
            is_open: AtomicBool::new(true),
        }
    }

    pub fn create_orderbook(&self, symbol: String) {
        self.orderbooks
            .insert(symbol.clone(), OrderBook::new(symbol));
    }

    /// Clear all orders from an orderbook (for game reset)
    pub fn clear_orderbook(&self, symbol: &str) {
        if let Some(mut ob) = self.orderbooks.get_mut(symbol) {
            ob.clear();
        }
    }

    /// Seed an order directly into the orderbook (for initial game state)
    /// This bypasses validation and matching - use only for initial liquidity
    pub fn seed_order(&self, order: Order) {
        let symbol = order.symbol.clone();
        if let Some(mut ob) = self.orderbooks.get_mut(&symbol) {
            tracing::info!(
                "Seeding order {} for {} at price {}",
                order.id,
                symbol,
                order.price
            );
            ob.seed_order(order);
        } else {
            tracing::warn!(
                "Cannot seed order - orderbook not found for symbol: {}",
                symbol
            );
        }
    }

    pub fn subscribe_trades(&self) -> broadcast::Receiver<Trade> {
        self.trade_sender.subscribe()
    }

    pub fn set_market_open(&self, open: bool) {
        self.is_open.store(open, Ordering::SeqCst);
        tracing::info!("Market status changed: open={}", open);
    }

    pub fn is_market_open(&self) -> bool {
        self.is_open.load(Ordering::SeqCst)
    }

    /// Get order book depth for a symbol
    pub fn get_order_book_depth(
        &self,
        symbol: &str,
        levels: usize,
    ) -> Option<(Vec<(Price, Quantity)>, Vec<(Price, Quantity)>)> {
        self.orderbooks.get(symbol).map(|ob| ob.get_depth(levels))
    }

    /// Cancel an order by ID
    pub async fn cancel_order(
        &self,
        user_id: u64,
        symbol: &str,
        order_id: u64,
    ) -> Result<Order, EngineError> {
        let mut orderbook = self
            .orderbooks
            .get_mut(symbol)
            .ok_or(EngineError::SymbolNotFound)?;

        let cancelled_order = orderbook
            .cancel_order(order_id)
            .ok_or(EngineError::OrderNotFound)?;

        // Verify ownership
        if cancelled_order.user_id != user_id {
            // Re-insert the order since we can't cancel someone else's order
            // Use the order's time_in_force when re-inserting
            orderbook.add_order(cancelled_order.clone(), cancelled_order.time_in_force);
            return Err(EngineError::OrderNotFound);
        }

        drop(orderbook); // Release lock before async operations

        // Release locked funds/shares
        self.release_locks(&cancelled_order).await?;

        // Remove from orders tracking
        self.orders_service.remove_order(order_id);

        Ok(cancelled_order)
    }

    /// Release locked funds or shares when an order is cancelled
    async fn release_locks(&self, order: &Order) -> Result<(), EngineError> {
        let mut user = self
            .user_repo
            .find_by_id(order.user_id)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?
            .ok_or(EngineError::UserNotFound)?;

        let remaining_qty = order.qty - order.filled_qty;

        match order.side {
            OrderSide::Buy => {
                // Release locked money
                let locked_amount = order.price * remaining_qty as i64;
                user.money += locked_amount;
                user.locked_money = user.locked_money.saturating_sub(locked_amount);
            }
            OrderSide::Sell => {
                // Release locked shares
                if let Some(pos) = user.portfolio.iter_mut().find(|p| p.symbol == order.symbol) {
                    pos.locked_qty = pos.locked_qty.saturating_sub(remaining_qty);
                }
            }
            OrderSide::Short => {
                // Release locked margin
                let margin_amount =
                    (order.price * remaining_qty as i64 * SHORT_MARGIN_PERCENT) / 100;
                user.money += margin_amount;
                user.margin_locked = user.margin_locked.saturating_sub(margin_amount);
            }
        }

        self.user_repo
            .save(user)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?;

        Ok(())
    }

    /// Place a new order with full validation
    pub async fn place_order(&self, mut order: Order) -> Result<Order, EngineError> {
        // Check market status
        if !self.is_market_open() {
            return Err(EngineError::MarketClosed);
        }

        // Verify symbol exists
        if !self.orderbooks.contains_key(&order.symbol) {
            return Err(EngineError::SymbolNotFound);
        }

        // Fetch user
        let mut user = self
            .user_repo
            .find_by_id(order.user_id)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?
            .ok_or(EngineError::UserNotFound)?;

        // For market orders, get the current best price for validation/locking
        // We'll use a reasonable estimate, then the actual matching will happen at market price
        let market_order_price = if order.order_type == OrderType::Market {
            let orderbook = self
                .orderbooks
                .get(&order.symbol)
                .ok_or(EngineError::SymbolNotFound)?;

            match order.side {
                OrderSide::Buy => {
                    // For market buy, use best ask + 10% buffer to ensure we can cover price movement
                    orderbook
                        .best_ask()
                        .map(|p| (p * 110) / 100) // 10% buffer
                        .unwrap_or(100 * PRICE_SCALE) // Default if no asks
                }
                OrderSide::Sell | OrderSide::Short => {
                    // For market sell/short, use best bid - doesn't affect fund locking
                    orderbook.best_bid().unwrap_or(1)
                }
            }
        } else {
            order.price
        };

        // Adjust price for market orders (for matching, use extreme price to ensure matching)
        if order.order_type == OrderType::Market {
            order.price = match order.side {
                OrderSide::Buy => i64::MAX / 2, // Very high price to match any ask
                OrderSide::Sell | OrderSide::Short => 1, // Very low price to match any bid
            };
        }

        // Validate and lock based on order side (use market_order_price for market orders)
        match order.side {
            OrderSide::Buy => {
                self.validate_and_lock_buy(&mut user, &order, market_order_price)?;
            }
            OrderSide::Sell => {
                self.validate_and_lock_sell(&mut user, &order)?;
            }
            OrderSide::Short => {
                self.validate_and_lock_short(&mut user, &order, market_order_price)?;
            }
        }

        // Save user with locked funds/shares
        self.user_repo
            .save(user.clone())
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?;

        // Process in order book
        let order_side = order.side;
        let time_in_force = order.time_in_force;
        // For settlement: use lock_price (what we actually locked) for price improvement calc
        let locked_price = market_order_price;

        let mut orderbook = self
            .orderbooks
            .get_mut(&order.symbol)
            .ok_or(EngineError::SymbolNotFound)?;

        let (mut processed_order, trades) = orderbook.add_order(order, time_in_force);
        drop(orderbook); // Release lock before async operations

        // Handle trades (settlement)
        for trade in &trades {
            if let Err(e) = self.settle_trade(trade, order_side, locked_price).await {
                tracing::error!("Trade settlement error: {}", e);
                // In production, this would need careful handling
            }

            // Update maker order in OrdersService
            // The maker order was resting in the book; check if it's now filled
            if let Some(maker_order) = self.orders_service.get_order(trade.maker_order_id) {
                let new_filled = maker_order.filled_qty + trade.qty;
                if new_filled >= maker_order.qty {
                    // Fully filled - remove from tracking
                    self.orders_service.remove_order(trade.maker_order_id);
                } else {
                    // Partially filled - update
                    self.orders_service.update_order(
                        trade.maker_order_id,
                        new_filled,
                        OrderStatus::Partial,
                    );
                }
            }

            let _ = self.trade_sender.send(trade.clone());
        }

        // Handle IOC orders that weren't fully filled
        if time_in_force == TimeInForce::IOC && processed_order.status != OrderStatus::Filled {
            processed_order.status = OrderStatus::Cancelled;
            // Release remaining locks
            self.release_locks(&processed_order).await?;
        }

        // Track order in OrdersService if it's still active (resting in book)
        if processed_order.status == OrderStatus::Open
            || processed_order.status == OrderStatus::Partial
        {
            self.orders_service.add_order(processed_order.clone());
        }

        Ok(processed_order)
    }

    /// Validate and lock funds for buy orders
    /// lock_price is the price to use for fund locking (different from order.price for market orders)
    fn validate_and_lock_buy(
        &self,
        user: &mut User,
        order: &Order,
        lock_price: Price,
    ) -> Result<(), EngineError> {
        let required_amount = lock_price * order.qty as i64;

        if user.money < required_amount {
            return Err(EngineError::InsufficientFunds {
                required: required_amount,
                available: user.money,
            });
        }

        user.money -= required_amount;
        user.locked_money += required_amount;

        tracing::debug!(
            "Locked {} for buy order {} by user {} (lock_price: {})",
            required_amount,
            order.id,
            order.user_id,
            lock_price
        );

        Ok(())
    }

    /// Validate and lock shares for sell orders
    fn validate_and_lock_sell(&self, user: &mut User, order: &Order) -> Result<(), EngineError> {
        // Find portfolio position
        let position = user.portfolio.iter_mut().find(|p| p.symbol == order.symbol);

        match position {
            Some(pos) => {
                let available = pos.qty.saturating_sub(pos.locked_qty);

                if available < order.qty {
                    return Err(EngineError::InsufficientShares {
                        required: order.qty,
                        available,
                    });
                }

                pos.locked_qty += order.qty;

                tracing::debug!(
                    "Locked {} shares of {} for sell order {} by user {}",
                    order.qty,
                    order.symbol,
                    order.id,
                    order.user_id
                );

                Ok(())
            }
            None => Err(EngineError::InsufficientShares {
                required: order.qty,
                available: 0,
            }),
        }
    }

    /// Validate and lock margin for short orders
    /// lock_price is the price to use for margin calculation (different from order.price for market orders)
    fn validate_and_lock_short(
        &self,
        user: &mut User,
        order: &Order,
        lock_price: Price,
    ) -> Result<(), EngineError> {
        // Short selling requires 150% margin
        let order_value = lock_price * order.qty as i64;
        let required_margin = (order_value * SHORT_MARGIN_PERCENT) / 100;

        if user.money < required_margin {
            return Err(EngineError::InsufficientMargin {
                required: required_margin,
                available: user.money,
            });
        }

        user.money -= required_margin;
        user.margin_locked += required_margin;

        tracing::debug!(
            "Locked {} margin for short order {} by user {} (lock_price: {})",
            required_margin,
            order.id,
            order.user_id,
            lock_price
        );

        Ok(())
    }

    /// Settle a trade between buyer and seller
    async fn settle_trade(
        &self,
        trade: &Trade,
        taker_side: OrderSide,
        taker_limit_price: Price,
    ) -> Result<(), EngineError> {
        // Identify buyer and seller
        let (buyer_id, seller_id, is_taker_buyer) = if taker_side == OrderSide::Buy {
            (trade.taker_user_id, trade.maker_user_id, true)
        } else {
            (trade.maker_user_id, trade.taker_user_id, false)
        };

        let is_short_sale = taker_side == OrderSide::Short;

        // Fetch both users
        let mut buyer = self
            .user_repo
            .find_by_id(buyer_id)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?
            .ok_or(EngineError::UserNotFound)?;

        let mut seller = self
            .user_repo
            .find_by_id(seller_id)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?
            .ok_or(EngineError::UserNotFound)?;

        // Capture names for trade history before modifying users
        let buyer_name = buyer.name.clone();
        let seller_name = seller.name.clone();

        let trade_cost = trade.price * trade.qty as i64;

        // --- Update Buyer ---
        // Release locked money (was locked at limit price) and handle price improvement
        if is_taker_buyer {
            // Taker buyer: We locked at taker_limit_price
            let locked_per_share = taker_limit_price;
            let locked_amount = locked_per_share * trade.qty as i64;
            let price_improvement = locked_amount - trade_cost;

            // Release the locked amount, refund price improvement
            buyer.locked_money = buyer.locked_money.saturating_sub(locked_amount);
            buyer.money += price_improvement; // Refund if we bought cheaper
        } else {
            // Maker buyer: Just release what was spent
            buyer.locked_money = buyer.locked_money.saturating_sub(trade_cost);
        }

        // Add shares to buyer's portfolio
        if let Some(pos) = buyer
            .portfolio
            .iter_mut()
            .find(|p| p.symbol == trade.symbol)
        {
            let old_cost = pos.average_buy_price * pos.qty as i64;
            let new_qty = pos.qty + trade.qty;
            pos.qty = new_qty;
            if new_qty > 0 {
                pos.average_buy_price = (old_cost + trade_cost) / new_qty as i64;
            }
        } else {
            buyer.portfolio.push(Portfolio {
                user_id: buyer.id,
                symbol: trade.symbol.clone(),
                qty: trade.qty,
                short_qty: 0,
                locked_qty: 0,
                average_buy_price: trade.price,
            });
        }

        // --- Update Seller ---
        if is_short_sale {
            // Short sale: Add to short position, release margin proportionally
            let margin_per_share = (taker_limit_price * SHORT_MARGIN_PERCENT) / 100;
            let margin_released = margin_per_share * trade.qty as i64;

            seller.margin_locked = seller.margin_locked.saturating_sub(margin_released);
            seller.money += trade_cost; // Receive sale proceeds

            // Track short position
            if let Some(pos) = seller
                .portfolio
                .iter_mut()
                .find(|p| p.symbol == trade.symbol)
            {
                pos.short_qty += trade.qty;
            } else {
                seller.portfolio.push(Portfolio {
                    user_id: seller.id,
                    symbol: trade.symbol.clone(),
                    qty: 0,
                    short_qty: trade.qty,
                    locked_qty: 0,
                    average_buy_price: trade.price, // For short, this tracks entry price
                });
            }
        } else {
            // Regular sell: Remove from portfolio, credit money
            seller.money += trade_cost;

            if let Some(pos) = seller
                .portfolio
                .iter_mut()
                .find(|p| p.symbol == trade.symbol)
            {
                pos.locked_qty = pos.locked_qty.saturating_sub(trade.qty);
                pos.qty = pos.qty.saturating_sub(trade.qty);
            }
        }

        // Save both users
        self.user_repo
            .save(buyer)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?;
        self.user_repo
            .save(seller)
            .await
            .map_err(|e| EngineError::InternalError(e.to_string()))?;

        // Record trade in history
        let buyer_side = OrderSide::Buy;
        let seller_side = if is_short_sale {
            OrderSide::Short
        } else {
            OrderSide::Sell
        };
        self.trade_history.record_trade(
            trade.clone(),
            buyer_name,
            seller_name,
            buyer_side,
            seller_side,
        );

        tracing::info!(
            "Settled trade {}: {} {} @ {} between buyer {} and seller {}",
            trade.id,
            trade.qty,
            trade.symbol,
            trade.price,
            buyer_id,
            seller_id
        );

        Ok(())
    }
}
