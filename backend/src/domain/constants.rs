//! Centralized constants for the StockMart domain.
//!
//! All magic numbers and configuration constants are defined here
//! to ensure consistency and easy modification across the codebase.
//!
//! Note: Many constants are defined for completeness and future use.
//! The dead_code warnings are suppressed for unused constants.

#![allow(dead_code)]

/// Price scaling factor - all prices are stored as integers multiplied by this value.
/// Example: $100.00 is stored as 1_000_000 (100 * 10_000)
pub const PRICE_SCALE: i64 = 10_000;

// =============================================================================
// TRADING CONSTANTS
// =============================================================================

pub mod trading {
    use super::PRICE_SCALE;

    /// Margin requirement for short selling (150% = 1.5x the position value)
    pub const SHORT_MARGIN_PERCENT: i64 = 150;

    /// Buffer percentage added to market order price estimates
    pub const MARKET_ORDER_BUFFER_PERCENT: i64 = 10;

    /// Price used for market buy orders (very high to match any ask)
    pub const MARKET_BUY_PRICE: i64 = i64::MAX / 2;

    /// Price used for market sell orders (very low to match any bid)
    pub const MARKET_SELL_PRICE: i64 = 1;

    /// Default price when no market data available
    pub const DEFAULT_PRICE: i64 = 100 * PRICE_SCALE;

    /// Size of the trade broadcast channel
    pub const TRADE_CHANNEL_SIZE: usize = 1000;
}

// =============================================================================
// CIRCUIT BREAKER CONSTANTS
// =============================================================================

pub mod circuit_breaker {
    use std::time::Duration;

    /// Price change threshold to trigger circuit breaker (10%)
    pub const THRESHOLD_PERCENT: i64 = 10;

    /// Duration to halt trading when circuit breaker triggers
    pub const HALT_DURATION: Duration = Duration::from_secs(60);

    /// Halt duration in seconds (for serialization)
    pub const HALT_DURATION_SECS: u64 = 60;

    /// Size of circuit breaker broadcast channel
    pub const CHANNEL_SIZE: usize = 100;
}

// =============================================================================
// USER CONSTANTS
// =============================================================================

pub mod user {
    use super::PRICE_SCALE;

    /// Default starting money for new users ($100,000)
    pub const DEFAULT_STARTING_MONEY: i64 = 100_000 * PRICE_SCALE;

    /// Default number of shares per company for new users
    pub const DEFAULT_SHARES_PER_COMPANY: u64 = 100;

    /// Variance percentage for share allocation (±20%)
    pub const SHARE_ALLOCATION_VARIANCE_PERCENT: i64 = 20;

    /// Base share price for calculations ($100)
    pub const BASE_SHARE_PRICE: i64 = 100 * PRICE_SCALE;

    /// Target portfolio percentage (50% of net worth in stocks)
    pub const TARGET_PORTFOLIO_PERCENT: i64 = 50;

    /// Minimum shares per company
    pub const MIN_SHARES_PER_COMPANY: u64 = 1;
}

// =============================================================================
// CHAT CONSTANTS
// =============================================================================

pub mod chat {
    /// Maximum length of a chat message
    pub const MAX_MESSAGE_LENGTH: usize = 500;

    /// Number of chat messages to keep in history
    pub const HISTORY_SIZE: usize = 100;

    /// Number of messages to send on sync
    pub const SYNC_MESSAGE_COUNT: usize = 50;
}

// =============================================================================
// SESSION CONSTANTS
// =============================================================================

pub mod session {
    use std::time::Duration;

    /// Default maximum sessions per user (1 = single session)
    pub const DEFAULT_MAX_SESSIONS_PER_USER: u32 = 1;

    /// Session timeout duration
    pub const SESSION_TIMEOUT: Duration = Duration::from_secs(3600);
}

// =============================================================================
// PERSISTENCE CONSTANTS
// =============================================================================

pub mod persistence {
    use std::time::Duration;

    /// Interval between auto-saves
    pub const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(60);

    /// Auto-save interval in seconds (for configuration)
    pub const AUTO_SAVE_INTERVAL_SECS: u64 = 60;

    /// Default data directory
    pub const DEFAULT_DATA_DIR: &str = "./data";

    /// Users data file name
    pub const USERS_FILE: &str = "users.json";

    /// Companies data file name
    pub const COMPANIES_FILE: &str = "companies.json";

    /// Event log file name
    pub const EVENT_LOG_FILE: &str = "game_events.jsonl";

    /// Config file name
    pub const CONFIG_FILE: &str = "config.json";
}

// =============================================================================
// WEBSOCKET CONSTANTS
// =============================================================================

pub mod websocket {
    /// Size of broadcast channels
    pub const BROADCAST_CHANNEL_SIZE: usize = 1000;

    /// Maximum orderbook depth levels to return
    pub const MAX_DEPTH_LEVELS: usize = 20;

    /// Default orderbook depth levels
    pub const DEFAULT_DEPTH_LEVELS: usize = 10;

    /// Ping interval in seconds
    pub const PING_INTERVAL_SECS: u64 = 30;

    /// Maximum WebSocket frame size
    pub const MAX_FRAME_SIZE: usize = 1024 * 1024; // 1MB
}

// =============================================================================
// MARKET DATA CONSTANTS
// =============================================================================

pub mod market {
    /// Default candle resolution
    pub const DEFAULT_CANDLE_RESOLUTION: &str = "1m";

    /// Maximum candles to keep in history per symbol
    pub const MAX_CANDLES_HISTORY: usize = 1000;

    /// Size of candle broadcast channel
    pub const CANDLE_CHANNEL_SIZE: usize = 100;

    /// Default number of recent trades to return
    pub const DEFAULT_RECENT_TRADES_COUNT: usize = 50;

    /// Number of trades to send on sync
    pub const SYNC_TRADES_COUNT: usize = 20;
}

// =============================================================================
// LEADERBOARD CONSTANTS
// =============================================================================

pub mod leaderboard {
    use std::time::Duration;

    /// Interval between leaderboard calculations
    pub const CALCULATION_INTERVAL: Duration = Duration::from_secs(5);

    /// Calculation interval in seconds
    pub const CALCULATION_INTERVAL_SECS: u64 = 5;
}

// =============================================================================
// NEWS CONSTANTS
// =============================================================================

pub mod news {
    /// Number of news items to keep in history
    pub const HISTORY_SIZE: usize = 100;

    /// Number of news items to send on sync
    pub const SYNC_NEWS_COUNT: usize = 20;
}

// =============================================================================
// ADMIN CONSTANTS
// =============================================================================

pub mod admin {
    use super::PRICE_SCALE;

    /// Admin user ID (legacy - prefer using Role::Admin)
    pub const ADMIN_USER_ID: u64 = 1;

    /// Default IPO shares
    pub const DEFAULT_IPO_SHARES: u64 = 1_000_000;

    /// Default price precision (decimal places)
    pub const DEFAULT_PRICE_PRECISION: u8 = 2;

    /// Default volatility factor
    pub const DEFAULT_VOLATILITY: i64 = 100;

    /// Base stock price for new companies
    pub const BASE_STOCK_PRICE: i64 = 100 * PRICE_SCALE;
}

// =============================================================================
// PAGINATION CONSTANTS
// =============================================================================

pub mod pagination {
    /// Default page size for paginated queries
    pub const DEFAULT_PAGE_SIZE: u32 = 20;

    /// Maximum page size allowed
    pub const MAX_PAGE_SIZE: u32 = 100;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Convert dollars to scaled price
/// Example: dollars_to_scaled(100) -> 1_000_000
pub const fn dollars_to_scaled(dollars: i64) -> i64 {
    dollars * PRICE_SCALE
}

/// Convert scaled price to dollars
/// Example: scaled_to_dollars(1_000_000) -> 100
pub const fn scaled_to_dollars(scaled: i64) -> i64 {
    scaled / PRICE_SCALE
}

/// Convert scaled price to dollars with decimal precision
pub fn scaled_to_dollars_f64(scaled: i64) -> f64 {
    scaled as f64 / PRICE_SCALE as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_scale() {
        assert_eq!(PRICE_SCALE, 10_000);
    }

    #[test]
    fn test_dollars_to_scaled() {
        assert_eq!(dollars_to_scaled(100), 1_000_000);
        assert_eq!(dollars_to_scaled(1), 10_000);
        assert_eq!(dollars_to_scaled(0), 0);
        assert_eq!(dollars_to_scaled(-50), -500_000);
    }

    #[test]
    fn test_scaled_to_dollars() {
        assert_eq!(scaled_to_dollars(1_000_000), 100);
        assert_eq!(scaled_to_dollars(10_000), 1);
        assert_eq!(scaled_to_dollars(0), 0);
        assert_eq!(scaled_to_dollars(-500_000), -50);
    }

    #[test]
    fn test_scaled_to_dollars_f64() {
        assert!((scaled_to_dollars_f64(1_000_000) - 100.0).abs() < 0.001);
        assert!((scaled_to_dollars_f64(10_000) - 1.0).abs() < 0.001);
        assert!((scaled_to_dollars_f64(5_000) - 0.5).abs() < 0.001);
        assert!((scaled_to_dollars_f64(0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let dollars = 12345;
        let scaled = dollars_to_scaled(dollars);
        let back = scaled_to_dollars(scaled);
        assert_eq!(back, dollars);
    }

    // Test constant values are reasonable
    #[test]
    fn test_trading_constants() {
        assert!(trading::SHORT_MARGIN_PERCENT > 100); // Must be > 100%
        assert!(trading::MARKET_ORDER_BUFFER_PERCENT > 0);
        assert!(trading::MARKET_BUY_PRICE > trading::MARKET_SELL_PRICE);
        assert!(trading::DEFAULT_PRICE > 0);
        assert!(trading::TRADE_CHANNEL_SIZE > 0);
    }

    #[test]
    fn test_circuit_breaker_constants() {
        assert!(circuit_breaker::THRESHOLD_PERCENT > 0);
        assert!(circuit_breaker::HALT_DURATION_SECS > 0);
        assert!(circuit_breaker::CHANNEL_SIZE > 0);
    }

    #[test]
    fn test_user_constants() {
        assert!(user::DEFAULT_STARTING_MONEY > 0);
        assert!(user::DEFAULT_SHARES_PER_COMPANY > 0);
        assert!(user::SHARE_ALLOCATION_VARIANCE_PERCENT >= 0);
        assert!(user::BASE_SHARE_PRICE > 0);
        assert!(user::TARGET_PORTFOLIO_PERCENT > 0 && user::TARGET_PORTFOLIO_PERCENT <= 100);
        assert!(user::MIN_SHARES_PER_COMPANY >= 1);
    }

    #[test]
    fn test_chat_constants() {
        assert!(chat::MAX_MESSAGE_LENGTH > 0);
        assert!(chat::HISTORY_SIZE > 0);
        assert!(chat::SYNC_MESSAGE_COUNT > 0);
        assert!(chat::SYNC_MESSAGE_COUNT <= chat::HISTORY_SIZE);
    }

    #[test]
    fn test_session_constants() {
        assert!(session::DEFAULT_MAX_SESSIONS_PER_USER >= 1);
    }

    #[test]
    fn test_persistence_constants() {
        assert!(persistence::AUTO_SAVE_INTERVAL_SECS > 0);
        assert!(!persistence::DEFAULT_DATA_DIR.is_empty());
        assert!(!persistence::USERS_FILE.is_empty());
        assert!(!persistence::COMPANIES_FILE.is_empty());
        assert!(!persistence::EVENT_LOG_FILE.is_empty());
        assert!(!persistence::CONFIG_FILE.is_empty());
    }

    #[test]
    fn test_websocket_constants() {
        assert!(websocket::BROADCAST_CHANNEL_SIZE > 0);
        assert!(websocket::MAX_DEPTH_LEVELS >= websocket::DEFAULT_DEPTH_LEVELS);
        assert!(websocket::DEFAULT_DEPTH_LEVELS > 0);
        assert!(websocket::PING_INTERVAL_SECS > 0);
        assert!(websocket::MAX_FRAME_SIZE > 0);
    }

    #[test]
    fn test_market_constants() {
        assert!(!market::DEFAULT_CANDLE_RESOLUTION.is_empty());
        assert!(market::MAX_CANDLES_HISTORY > 0);
        assert!(market::CANDLE_CHANNEL_SIZE > 0);
        assert!(market::DEFAULT_RECENT_TRADES_COUNT > 0);
        assert!(market::SYNC_TRADES_COUNT > 0);
    }

    #[test]
    fn test_leaderboard_constants() {
        assert!(leaderboard::CALCULATION_INTERVAL_SECS > 0);
    }

    #[test]
    fn test_news_constants() {
        assert!(news::HISTORY_SIZE > 0);
        assert!(news::SYNC_NEWS_COUNT > 0);
        assert!(news::SYNC_NEWS_COUNT <= news::HISTORY_SIZE);
    }

    #[test]
    fn test_admin_constants() {
        assert!(admin::DEFAULT_IPO_SHARES > 0);
        assert!(admin::DEFAULT_VOLATILITY > 0);
        assert!(admin::BASE_STOCK_PRICE > 0);
    }

    #[test]
    fn test_pagination_constants() {
        assert!(pagination::DEFAULT_PAGE_SIZE > 0);
        assert!(pagination::MAX_PAGE_SIZE >= pagination::DEFAULT_PAGE_SIZE);
    }
}
