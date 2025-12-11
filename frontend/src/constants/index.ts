// ============================================
// Application Constants
// Centralized fallback configuration values
// ============================================
//
// IMPORTANT: These are FALLBACK values only!
// The actual values should be fetched from the backend via FrontendConstants.
// Use useConfigStore() to get the server-provided values:
//
//   import { useConfigStore } from '../store/configStore';
//   const { constants } = useConfigStore();
//   const depth = constants.ui.orderbook_depth;
//
// These constants are kept for:
// 1. Type-safe fallback values before server connection
// 2. Components that cannot use hooks (non-React code)
// ============================================

// === Data Limits (fallback) ===
// Maximum number of items to keep in local state
export const LIMITS = {
    /** Maximum trades to keep in state */
    TRADES_HISTORY: 100,
    /** Maximum candles to keep per symbol */
    CANDLES_PER_SYMBOL: 200,
    /** Maximum chat messages to keep */
    CHAT_MESSAGES: 50,
    /** Maximum news items to keep */
    NEWS_ITEMS: 10,
    /** Default page size for trade history */
    TRADE_HISTORY_PAGE_SIZE: 50,
    /** Default stock trades count */
    STOCK_TRADES_COUNT: 50,
} as const;

// === UI Constants (fallback) ===
export const UI = {
    /** Default orderbook depth levels to show */
    ORDERBOOK_DEPTH: 8,
    /** Default leaderboard entries to show */
    LEADERBOARD_ENTRIES: 10,
    /** Default chat messages to show */
    CHAT_MESSAGES_VISIBLE: 20,
    /** Trade history entries to show in widget */
    TRADE_HISTORY_WIDGET: 20,
    /** Stock trades to show in orderbook */
    STOCK_TRADES_WIDGET: 15,
} as const;

// === Animation (fallback) ===
export const ANIMATION = {
    /** Base duration for news ticker (seconds) */
    NEWS_TICKER_BASE_DURATION: 20,
    /** Additional duration per news item (seconds) */
    NEWS_TICKER_PER_ITEM: 5,
} as const;

// === Trading (fallback) ===
export const TRADING = {
    /** Default order quantity */
    DEFAULT_ORDER_QTY: 10,
    /** Short selling margin requirement (percentage) */
    SHORT_MARGIN_PERCENT: 150,
} as const;

// === Game Initialization Defaults (fallback) ===
export const GAME_DEFAULTS = {
    /** Default target net worth for new traders */
    TARGET_NETWORTH: 200000,
    /** Default base shares per company */
    SHARES_PER_TRADER: 100,
    /** Default trading start time */
    TRADING_START_TIME: '09:00',
    /** Default trading end time */
    TRADING_END_TIME: '16:00',
    /** Default circuit breaker threshold (%) */
    CIRCUIT_BREAKER_THRESHOLD: 10,
    /** Default circuit breaker duration (seconds) */
    CIRCUIT_BREAKER_DURATION: 300,
} as const;

// === Polling & Refresh (fallback) ===
export const POLLING = {
    /** Dashboard metrics refresh interval (ms) */
    DASHBOARD_METRICS_INTERVAL: 10000,
} as const;

// === Company Form (fallback) ===
export const COMPANY_FORM = {
    /** Maximum symbol length */
    SYMBOL_MAX_LENGTH: 5,
    /** Volatility range */
    VOLATILITY_MIN: 0.1,
    VOLATILITY_MAX: 0.5,
    VOLATILITY_STEP: 0.05,
    /** Default total shares for new company */
    DEFAULT_TOTAL_SHARES: 1_000_000,
    /** Default initial price (scaled) */
    DEFAULT_INITIAL_PRICE: 1_000_000,
} as const;

// === Sectors (fallback) ===
export const SECTORS = [
    'Tech',
    'Finance',
    'Healthcare',
    'Energy',
    'Consumer',
    'Industrial',
] as const;

export type Sector = typeof SECTORS[number];
