use crate::domain::models::{Candle, Trade, Price, Quantity};
use dashmap::DashMap;
use tokio::sync::broadcast;
use chrono::{DateTime, Utc, Timelike, TimeZone};

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
                 tracing::warn!("CIRCUIT BREAKER TRIGGERED for {}: Price {} vs Ref {}", trade.symbol, trade.price, ref_price);
             }
        } else {
            // Initialize reference price
            self.circuit_breakers.insert(trade.symbol.clone(), (0, trade.price));
        }

        if should_halt {
            let _ = self.cb_tx.send((trade.symbol.clone(), halt_until));
        }

        // Aggregate into 1-minute candle
        let timestamp = trade.timestamp;
        // Round down to nearest minute
        let dt = Utc.timestamp_opt(timestamp, 0).unwrap();
        let candle_time = dt.with_second(0).unwrap().with_nanosecond(0).unwrap().timestamp();

        let mut candles = self.candles.entry(trade.symbol.clone()).or_insert_with(Vec::new);
        
        if let Some(last_candle) = candles.last_mut() {
            if last_candle.timestamp == candle_time {
                // Update existing candle
                last_candle.high = std::cmp::max(last_candle.high, trade.price);
                last_candle.low = std::cmp::min(last_candle.low, trade.price);
                last_candle.close = trade.price;
                last_candle.volume += trade.qty;
                
                // Broadcast update
                let _ = self.candle_tx.send(last_candle.clone());
                return;
            }
        }

        // Create new candle
        let new_candle = Candle {
            symbol: trade.symbol.clone(),
            resolution: "1m".to_string(),
            open: trade.price,
            high: trade.price,
            low: trade.price,
            close: trade.price,
            volume: trade.qty,
            timestamp: candle_time,
        };
        
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
}
