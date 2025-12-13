use crate::domain::models::{Candle, Trade};
use chrono::{TimeZone, Timelike, Utc};
use dashmap::DashMap;
use tokio::sync::broadcast;

pub struct MarketService {
    // Symbol -> Resolution -> Vec<Candle>
    candles: DashMap<String, Vec<Candle>>,
    candle_tx: broadcast::Sender<Candle>,
    // Symbol -> (Halted Until Timestamp, Reference Price)
    circuit_breakers: DashMap<String, (i64, i64)>,
    cb_tx: broadcast::Sender<(String, i64)>, // Symbol, Halted Until
}

impl MarketService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        let (cb_tx, _) = broadcast::channel(100);
        Self {
            candles: DashMap::new(),
            candle_tx: tx,
            circuit_breakers: DashMap::new(),
            cb_tx,
        }
    }

    pub fn subscribe_candles(&self) -> broadcast::Receiver<Candle> {
        self.candle_tx.subscribe()
    }

    pub fn subscribe_circuit_breakers(&self) -> broadcast::Receiver<(String, i64)> {
        self.cb_tx.subscribe()
    }

    #[allow(dead_code)] // API method for checking individual symbol halt status
    pub fn is_halted(&self, symbol: &str) -> bool {
        if let Some(cb) = self.circuit_breakers.get(symbol) {
            let (halted_until, _) = *cb;
            if Utc::now().timestamp() < halted_until {
                return true;
            }
        }
        false
    }

    pub fn get_last_price(&self, symbol: &str) -> Option<i64> {
        if let Some(candles) = self.candles.get(symbol) {
            if let Some(last) = candles.last() {
                return Some(last.close);
            }
        }
        None
    }

    pub async fn run(&self, mut trade_rx: broadcast::Receiver<Trade>) {
        loop {
            match trade_rx.recv().await {
                Ok(trade) => {
                    self.process_trade(trade);
                }
                Err(e) => {
                    tracing::error!("MarketService trade receive error: {}", e);
                    break;
                }
            }
        }
    }

    fn process_trade(&self, trade: Trade) {
        // Check Circuit Breaker
        // For simplicity, we'll set the reference price as the Open price of the current candle
        // If price moves > 10% from Open, we halt for 1 minute

        let mut should_halt = false;
        let mut halt_until = 0;

        if let Some(mut cb) = self.circuit_breakers.get_mut(&trade.symbol) {
            let (halted_until, ref_price) = *cb;

            // If currently halted, ignore (should be blocked by engine, but double check)
            if Utc::now().timestamp() < halted_until {
                return;
            }

            // Check 10% move
            let diff = (trade.price - ref_price).abs();
            let threshold = ref_price / 10; // 10%

            if diff > threshold {
                should_halt = true;
                halt_until = Utc::now().timestamp() + 60; // Halt for 1 minute
                cb.0 = halt_until;
                // Reset reference price to current price after halt
                cb.1 = trade.price;
                tracing::warn!(
                    "CIRCUIT BREAKER TRIGGERED for {}: Price {} vs Ref {}",
                    trade.symbol,
                    trade.price,
                    ref_price
                );
            }
        } else {
            // Initialize reference price
            self.circuit_breakers
                .insert(trade.symbol.clone(), (0, trade.price));
        }

        if should_halt {
            let _ = self.cb_tx.send((trade.symbol.clone(), halt_until));
        }

        // Aggregate into 1-minute candle
        let timestamp = trade.timestamp;
        // Round down to nearest minute
        let dt = Utc.timestamp_opt(timestamp, 0).unwrap();
        let candle_time = dt
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            .timestamp();

        let mut candles = self
            .candles
            .entry(trade.symbol.clone())
            .or_insert_with(Vec::new);

        if let Some(last_candle) = candles.last_mut() {
            if last_candle.timestamp == candle_time {
                // Update existing candle
                last_candle.update(trade.price, trade.qty);

                // Broadcast update
                let _ = self.candle_tx.send(last_candle.clone());
                return;
            }
        }

        // Create new candle
        let mut new_candle = Candle::new(
            trade.symbol.clone(),
            "1m".to_string(),
            trade.price,
            candle_time,
        );
        new_candle.volume = trade.qty;

        // Broadcast new candle
        let _ = self.candle_tx.send(new_candle.clone());
        candles.push(new_candle);
    }

    pub fn get_candles(&self, symbol: &str) -> Vec<Candle> {
        if let Some(c) = self.candles.get(symbol) {
            c.clone()
        } else {
            Vec::new()
        }
    }

    /// Get all currently halted symbols with their halt-until timestamps
    pub fn get_halted_symbols(&self) -> Vec<(String, i64)> {
        let now = Utc::now().timestamp();
        self.circuit_breakers
            .iter()
            .filter_map(|entry| {
                let (halted_until, _) = *entry.value();
                if halted_until > now {
                    Some((entry.key().clone(), halted_until))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Process a single trade - exposed for testing
    #[cfg(test)]
    pub fn test_process_trade(&self, trade: Trade) {
        self.process_trade(trade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_service_new() {
        let svc = MarketService::new();
        // Should be able to subscribe
        let _rx = svc.subscribe_candles();
        let _cb_rx = svc.subscribe_circuit_breakers();
    }

    #[test]
    fn test_get_last_price_empty() {
        let svc = MarketService::new();
        assert!(svc.get_last_price("AAPL").is_none());
    }

    #[test]
    fn test_get_candles_empty() {
        let svc = MarketService::new();
        let candles = svc.get_candles("AAPL");
        assert!(candles.is_empty());
    }

    #[test]
    fn test_is_halted_not_halted() {
        let svc = MarketService::new();
        assert!(!svc.is_halted("AAPL"));
    }

    #[test]
    fn test_is_halted_expired() {
        let svc = MarketService::new();
        // Insert an expired halt
        svc.circuit_breakers
            .insert("AAPL".to_string(), (1, 1000000));
        assert!(!svc.is_halted("AAPL"));
    }

    #[test]
    fn test_is_halted_active() {
        let svc = MarketService::new();
        // Insert a future halt
        let future = Utc::now().timestamp() + 3600;
        svc.circuit_breakers
            .insert("AAPL".to_string(), (future, 1000000));
        assert!(svc.is_halted("AAPL"));
    }

    #[test]
    fn test_get_halted_symbols_none() {
        let svc = MarketService::new();
        assert!(svc.get_halted_symbols().is_empty());
    }

    #[test]
    fn test_get_halted_symbols_expired() {
        let svc = MarketService::new();
        // Insert an expired halt
        svc.circuit_breakers
            .insert("AAPL".to_string(), (1, 1000000));
        assert!(svc.get_halted_symbols().is_empty());
    }

    #[test]
    fn test_get_halted_symbols_active() {
        let svc = MarketService::new();
        let future = Utc::now().timestamp() + 3600;
        svc.circuit_breakers
            .insert("AAPL".to_string(), (future, 1000000));
        let halted = svc.get_halted_symbols();
        assert_eq!(halted.len(), 1);
        assert_eq!(halted[0].0, "AAPL");
        assert_eq!(halted[0].1, future);
    }

    #[test]
    fn test_process_trade_creates_candle() {
        let svc = MarketService::new();
        let trade = Trade {
            id: 1,
            symbol: "AAPL".to_string(),
            price: 1500000, // $150.00 scaled
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade);

        let candles = svc.get_candles("AAPL");
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].symbol, "AAPL");
        assert_eq!(candles[0].close, 1500000);
    }

    #[test]
    fn test_process_trade_updates_existing_candle() {
        let svc = MarketService::new();
        let now = Utc::now().timestamp();

        let trade1 = Trade {
            id: 1,
            symbol: "AAPL".to_string(),
            price: 1500000,
            qty: 10,
            timestamp: now,
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade1);

        let trade2 = Trade {
            id: 2,
            symbol: "AAPL".to_string(),
            price: 1550000, // Higher price
            qty: 5,
            timestamp: now, // Same minute
            taker_user_id: 3,
            maker_user_id: 4,
            taker_order_id: 101,
            maker_order_id: 201,
        };
        svc.test_process_trade(trade2);

        let candles = svc.get_candles("AAPL");
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].close, 1550000); // Updated to latest price
        assert_eq!(candles[0].high, 1550000); // High should be updated
        assert_eq!(candles[0].volume, 15); // Volume accumulated
    }

    #[test]
    fn test_get_last_price_after_trade() {
        let svc = MarketService::new();
        let trade = Trade {
            id: 1,
            symbol: "GOOGL".to_string(),
            price: 2800000,
            qty: 5,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade);

        assert_eq!(svc.get_last_price("GOOGL"), Some(2800000));
    }

    #[test]
    fn test_circuit_breaker_initialization() {
        let svc = MarketService::new();
        let trade = Trade {
            id: 1,
            symbol: "TEST".to_string(),
            price: 1000000,
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade);

        // Circuit breaker should be initialized with reference price
        assert!(svc.circuit_breakers.contains_key("TEST"));
        let cb = svc.circuit_breakers.get("TEST").unwrap();
        assert_eq!(cb.1, 1000000); // Reference price set
    }

    #[test]
    fn test_circuit_breaker_triggers_on_large_move() {
        let svc = MarketService::new();

        // First trade establishes reference price
        let trade1 = Trade {
            id: 1,
            symbol: "VOLATILE".to_string(),
            price: 1000000, // $100.00
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade1);

        // Second trade with >10% move should trigger circuit breaker
        let trade2 = Trade {
            id: 2,
            symbol: "VOLATILE".to_string(),
            price: 1150000, // $115.00 - 15% move
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 3,
            maker_user_id: 4,
            taker_order_id: 101,
            maker_order_id: 201,
        };
        svc.test_process_trade(trade2);

        // Should be halted
        assert!(svc.is_halted("VOLATILE"));
    }

    #[test]
    fn test_multiple_symbols_independent() {
        let svc = MarketService::new();

        let trade1 = Trade {
            id: 1,
            symbol: "AAPL".to_string(),
            price: 1500000,
            qty: 10,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 100,
            maker_order_id: 200,
        };
        svc.test_process_trade(trade1);

        let trade2 = Trade {
            id: 2,
            symbol: "GOOGL".to_string(),
            price: 2800000,
            qty: 5,
            timestamp: Utc::now().timestamp(),
            taker_user_id: 1,
            maker_user_id: 2,
            taker_order_id: 101,
            maker_order_id: 201,
        };
        svc.test_process_trade(trade2);

        assert_eq!(svc.get_last_price("AAPL"), Some(1500000));
        assert_eq!(svc.get_last_price("GOOGL"), Some(2800000));
        assert_eq!(svc.get_candles("AAPL").len(), 1);
        assert_eq!(svc.get_candles("GOOGL").len(), 1);
    }
}
