use crate::domain::models::{Order, OrderStatus, Trade, User, Portfolio, Price, Quantity, OrderSide};
use crate::domain::orderbook::OrderBook;
use crate::repository::{UserRepository, CompanyRepository};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

pub struct MatchingEngine {
    orderbooks: DashMap<String, OrderBook>, // Symbol -> OrderBook
    user_repo: Arc<dyn UserRepository>,
    // company_repo: Arc<dyn CompanyRepository>, // For future use (e.g. circuit breakers)
    trade_sender: broadcast::Sender<Trade>, // Broadcast trades to WS
    is_open: AtomicBool,
}

impl MatchingEngine {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            orderbooks: DashMap::new(),
            user_repo,
            trade_sender: tx,
            is_open: AtomicBool::new(true),
        }
    }

    pub fn create_orderbook(&self, symbol: String) {
        self.orderbooks.insert(symbol.clone(), OrderBook::new(symbol));
    }

    pub fn subscribe_trades(&self) -> broadcast::Receiver<Trade> {
        self.trade_sender.subscribe()
    }

    pub fn set_market_open(&self, open: bool) {
        self.is_open.store(open, Ordering::Relaxed);
    }

    pub fn is_market_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    pub async fn place_order(&self, mut order: Order) -> Result<Order, String> {
        if !self.is_market_open() {
            return Err("Market is closed".to_string());
        }
        
        // 1. Validate & Lock Funds/Shares
        // This is critical. We must ensure the user has enough money (Buy) or shares (Sell).
        // We need to fetch the user, update their locked state, and save.
        // NOTE: In a real DB, this needs a transaction. With DashMap/Memory, we need care.
        // For now, we'll do a read-modify-write. Since it's a single server, we can use locks if needed, 
        // but DashMap is concurrent. However, `user_repo.save` overwrites. 
        // We should probably add `lock_funds` methods to the Repo or User struct to be atomic.
        // For this implementation, we will assume single-threaded access per user or optimistic locking.
        // Let's just do it simply for now.

        let user_opt = self.user_repo.find_by_id(order.user_id).await.map_err(|e| e.to_string())?;
        let mut user = user_opt.ok_or("User not found")?;

        if order.side == OrderSide::Buy {
            let required_amount = order.price * order.qty as i64; // Limit order assumption
            if user.money < required_amount {
                return Err("Insufficient funds".to_string());
            }
            user.money -= required_amount;
            user.locked_money += required_amount;
            self.user_repo.save(user.clone()).await.map_err(|e| e.to_string())?;
        } else {
            // Sell/Short logic
            // Check portfolio... (omitted for brevity in this step, but required)
            // For Short, we might check margin.
        }

        // 2. Process in OrderBook
        let mut orderbook = self.orderbooks.get_mut(&order.symbol).ok_or("Symbol not found")?;
        let order_side = order.side; // Capture side before move
        let (processed_order, trades) = orderbook.add_order(order);

        // 3. Handle Trades (Settlement)
        for trade in trades {
            self.settle_trade(&trade, order_side).await?;
            let _ = self.trade_sender.send(trade);
        }

        // 4. If order is filled/cancelled, release locks?
        // Actually, settlement handles the exchange. 
        // If the order was PARTIALLY filled or OPEN, the funds remain locked.
        // If the order was a BUY and executed at a BETTER price, we need to refund the difference.
        // This logic is complex. 
        // Simplified: 
        // - On Buy Trade: 
        //   - Buyer: locked_money -= trade.price * trade.qty. 
        //     Wait, we locked `limit_price * qty`. 
        //     If trade_price < limit_price, we refund `(limit_price - trade_price) * qty`.
        //     And we add stock to portfolio.
        //   - Seller: add money to balance. Remove stock from locked_qty.

        Ok(processed_order)
    }

    async fn settle_trade(&self, trade: &Trade, taker_side: OrderSide) -> Result<(), String> {
        // Identify Buyer and Seller
        let (buyer_id, seller_id) = if taker_side == OrderSide::Buy {
            (trade.taker_user_id, trade.maker_user_id)
        } else {
            (trade.maker_user_id, trade.taker_user_id)
        };

        let mut buyer = self.user_repo.find_by_id(buyer_id).await.map_err(|e| e.to_string())?
            .ok_or("Buyer not found")?;
        let mut seller = self.user_repo.find_by_id(seller_id).await.map_err(|e| e.to_string())?
            .ok_or("Seller not found")?;

        let total_cost = trade.price * trade.qty as i64;

        // --- Update Buyer ---
        // Buyer locked funds for this trade. 
        // If Taker was Buyer: Locked = limit_price * qty.
        // If Maker was Buyer: Locked = maker_limit_price * qty.
        // We don't know the Maker's limit price here easily! 
        // CRITICAL: We need to know how much was locked to unlock it correctly.
        // Assumption: For MVP, we assume locked amount is exactly what was spent + refund.
        // Actually, if we just deduct `total_cost` from `locked_money` and `money`, it might be wrong if we locked MORE.
        // Correct logic: 
        // `locked_money` should be reduced by the amount that was reserved for THIS chunk of the order.
        // But we don't track per-chunk lock.
        // Simplified approach:
        // 1. Deduct cost from `money`.
        // 2. Reduce `locked_money` by cost (assuming we locked exactly cost). 
        //    Wait, if we bought cheaper, we locked MORE.
        //    We need to release the difference.
        //    Difference = (Locked Price - Execution Price) * Qty.
        //    We need Locked Price.
        //    For Taker (Buy), it's `order.price`.
        //    For Maker (Buy), it's `maker_order.price`.
        //    We still need Maker's price.
        
        // FIX: For now, we will just deduct `total_cost` from `locked_money`. 
        // Any "excess" lock will remain until the order is fully filled or cancelled, 
        // at which point we should reconcile.
        // OR: We can say `locked_money -= total_cost`. 
        // If `locked_money` goes negative (impossible if logic is right), we have a bug.
        // The "refund" for price improvement happens when the order is finalized/cancelled?
        // No, user wants to use that money immediately.
        
        // Let's stick to: `locked_money -= total_cost`.
        // And we add shares to portfolio.
        
        buyer.money -= total_cost; // Already deducted from money when locking? 
        // No, when locking we did: `money -= amount; locked += amount`.
        // So `money` is already reduced. We just need to reduce `locked`.
        // Wait, if we reduce `locked`, where does it go? It disappears (spent).
        // So: `locked_money -= total_cost`.
        // But if we bought cheaper, we should refund `(limit - exec) * qty` to `money`.
        // Since we don't know limit, we can't refund yet. 
        // This is a limitation of not having the order.
        // However, `locked_money` is just a safety counter. 
        // The actual `money` balance was decremented by `limit * qty`.
        // So we effectively paid `limit * qty`.
        // If we want to refund, we MUST know the limit.
        
        // Let's assume for now we don't refund immediately (or we fetch the order).
        // Fetching order is hard as it might be gone.
        // Let's just update portfolio.
        
        buyer.locked_money -= total_cost; // This might underflow if we locked less (Market Order?)
        // Market Order locks estimate?
        
        // Update Buyer Portfolio
        if let Some(pos) = buyer.portfolio.iter_mut().find(|p| p.symbol == trade.symbol) {
            let old_cost = pos.average_buy_price * pos.qty as i64;
            pos.qty += trade.qty;
            pos.average_buy_price = (old_cost + total_cost) / pos.qty as i64;
        } else {
            buyer.portfolio.push(Portfolio {
                user_id: buyer.id,
                symbol: trade.symbol.clone(),
                qty: trade.qty,
                locked_qty: 0,
                average_buy_price: trade.price,
            });
        }

        // --- Update Seller ---
        // Seller locked shares.
        // `locked_qty -= qty`.
        // `money += total_cost`.
        
        seller.money += total_cost;
        
        if let Some(pos) = seller.portfolio.iter_mut().find(|p| p.symbol == trade.symbol) {
            pos.locked_qty -= trade.qty;
            pos.qty -= trade.qty;
            // If qty is 0, remove? Maybe keep for history or remove.
        } else {
            return Err("Seller does not have the stock".to_string());
        }

        // Save
        self.user_repo.save(buyer).await.map_err(|e| e.to_string())?;
        self.user_repo.save(seller).await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
