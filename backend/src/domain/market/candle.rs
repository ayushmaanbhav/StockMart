//! Candle (OHLCV) entity for the market bounded context.
//!
//! Candlestick data for price charts.

use crate::domain::common::{Price, Quantity};
use serde::{Deserialize, Serialize};

/// OHLCV candlestick data for charting.
///
/// Represents price movement over a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Trading symbol
    pub symbol: String,
    /// Time resolution (e.g., "1m", "5m", "1h")
    pub resolution: String,
    /// Opening price
    pub open: Price,
    /// Highest price
    pub high: Price,
    /// Lowest price
    pub low: Price,
    /// Closing price
    pub close: Price,
    /// Total volume traded
    pub volume: Quantity,
    /// Unix timestamp of the candle start
    pub timestamp: i64,
}

impl Candle {
    /// Create a new candle with the given opening price.
    pub fn new(symbol: String, resolution: String, price: Price, timestamp: i64) -> Self {
        Self {
            symbol,
            resolution,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0,
            timestamp,
        }
    }

    /// Update the candle with a new trade.
    pub fn update(&mut self, price: Price, volume: Quantity) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
    }
}
