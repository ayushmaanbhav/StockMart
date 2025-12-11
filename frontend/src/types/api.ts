// ============================================
// WebSocket API Types
// ============================================

import type {
    OrderSide,
    OrderType,
    TimeInForce,
    RawPrice,
    Quantity,
    OrderId,
    UserId,
} from './models';

// === Client to Server Messages ===

export interface ClientAuth {
    type: 'Auth';
    payload: {
        token: string;
    };
}

export interface ClientLogin {
    type: 'Login';
    payload: {
        regno: string;
        password: string;
    };
}

export interface ClientRegister {
    type: 'Register';
    payload: {
        regno: string;
        name: string;
        password: string;
    };
}

export interface ClientPlaceOrder {
    type: 'PlaceOrder';
    payload: {
        symbol: string;
        side: OrderSide;
        order_type: OrderType;
        time_in_force?: TimeInForce;
        qty: Quantity;
        price: RawPrice;
    };
}

export interface ClientCancelOrder {
    type: 'CancelOrder';
    payload: {
        symbol: string;
        order_id: OrderId;
    };
}

export interface ClientSubscribe {
    type: 'Subscribe';
    payload: {
        symbol: string;
    };
}

export interface ClientGetDepth {
    type: 'GetDepth';
    payload: {
        symbol: string;
        levels?: number;
    };
}

export interface ClientAdminAction {
    type: 'AdminAction';
    payload: {
        action: string;
        payload: Record<string, unknown>;
    };
}

export interface ClientChat {
    type: 'Chat';
    payload: {
        message: string;
    };
}

export interface ClientGetPortfolio {
    type: 'GetPortfolio';
    payload: Record<string, never>;
}

export interface ClientPing {
    type: 'Ping';
    payload: Record<string, never>;
}

export interface ClientRequestSync {
    type: 'RequestSync';
    payload: {
        component?: string;  // Optional: "portfolio", "orders", "leaderboard", "indices"
    };
}

export interface ClientGetTradeHistory {
    type: 'GetTradeHistory';
    payload: {
        page?: number;
        page_size?: number;
        symbol?: string;
    };
}

export interface ClientGetStockTrades {
    type: 'GetStockTrades';
    payload: {
        symbol: string;
        count?: number;
    };
}

export type ClientMessage =
    | ClientAuth
    | ClientLogin
    | ClientRegister
    | ClientPlaceOrder
    | ClientCancelOrder
    | ClientSubscribe
    | ClientGetDepth
    | ClientAdminAction
    | ClientChat
    | ClientGetPortfolio
    | ClientPing
    | ClientRequestSync
    | ClientGetTradeHistory
    | ClientGetStockTrades;

// === Server to Client Messages ===

export interface ServerAuthSuccess {
    type: 'AuthSuccess';
    payload: {
        user_id: UserId;
        name: string;
    };
}

export interface ServerAuthFailed {
    type: 'AuthFailed';
    payload: {
        reason: string;
    };
}

export interface ServerRegisterSuccess {
    type: 'RegisterSuccess';
    payload: {
        user_id: UserId;
        name: string;
    };
}

export interface ServerRegisterFailed {
    type: 'RegisterFailed';
    payload: {
        reason: string;
    };
}

export interface ServerOrderAck {
    type: 'OrderAck';
    payload: {
        order_id: OrderId;
        status: string;
        filled_qty: Quantity;
        remaining_qty: Quantity;
    };
}

export interface ServerOrderRejected {
    type: 'OrderRejected';
    payload: {
        reason: string;
        error_code: string;
    };
}

export interface ServerOrderCancelled {
    type: 'OrderCancelled';
    payload: {
        order_id: OrderId;
    };
}

export interface ServerTradeUpdate {
    type: 'TradeUpdate';
    payload: {
        symbol: string;
        price: RawPrice;
        qty: Quantity;
        timestamp: number;
    };
}

export interface ServerCandleUpdate {
    type: 'CandleUpdate';
    payload: {
        symbol: string;
        candle: {
            symbol: string;
            resolution: string;
            open: RawPrice;
            high: RawPrice;
            low: RawPrice;
            close: RawPrice;
            volume: Quantity;
            timestamp: number;
        };
    };
}

export interface ServerDepthUpdate {
    type: 'DepthUpdate';
    payload: {
        symbol: string;
        bids: Array<[RawPrice, Quantity]>;
        asks: Array<[RawPrice, Quantity]>;
        spread: RawPrice | null;
    };
}

export interface ServerIndexUpdate {
    type: 'IndexUpdate';
    payload: {
        name: string;
        value: RawPrice;
    };
}

export interface ServerPortfolioUpdate {
    type: 'PortfolioUpdate';
    payload: {
        money: RawPrice;
        locked: RawPrice;
        margin_locked: RawPrice;
        net_worth: RawPrice;
        items: Array<{
            user_id: UserId;
            symbol: string;
            qty: Quantity;
            short_qty: Quantity;
            locked_qty: Quantity;
            average_buy_price: RawPrice;
        }>;
    };
}

export interface ServerCircuitBreaker {
    type: 'CircuitBreaker';
    payload: {
        symbol: string;
        halted_until: number;
        reason: string;
    };
}

export interface ServerMarketStatus {
    type: 'MarketStatus';
    payload: {
        is_open: boolean;
    };
}

export interface ServerNewsUpdate {
    type: 'NewsUpdate';
    payload: {
        news: {
            id: number;
            headline: string;
            sentiment: 'Bullish' | 'Bearish' | 'Neutral';
            symbol: string | null;
            timestamp: number;
        };
    };
}

export interface ServerLeaderboardUpdate {
    type: 'LeaderboardUpdate';
    payload: {
        entries: Array<{
            rank: number;
            name: string;
            net_worth: RawPrice;
        }>;
    };
}

export interface ServerChatUpdate {
    type: 'ChatUpdate';
    payload: {
        message: {
            id: string;
            user_id: UserId;
            username: string;
            message: string;
            timestamp: number;
        };
    };
}

export interface ServerError {
    type: 'Error';
    payload: {
        code: string;
        message: string;
    };
}

export interface ServerPong {
    type: 'Pong';
    payload: {
        timestamp: number;
    };
}

export interface ServerSystem {
    type: 'System';
    payload: {
        message: string;
    };
}

export interface ServerCompanyList {
    type: 'CompanyList';
    payload: {
        companies: Array<{
            id: number;
            symbol: string;
            name: string;
            sector: string;
            volatility: number;
        }>;
    };
}

export interface ServerConfig {
    type: 'Config';
    payload: {
        registration_mode: string;
        chat_enabled: boolean;
        currency: {
            symbol: string;
            code: string;
            locale: string;
            decimals: number;
            symbol_position: 'before' | 'after';
        };
    };
}

// Frontend constants from server
export interface FrontendConstants {
    limits: {
        trades_history: number;
        candles_per_symbol: number;
        chat_messages: number;
        news_items: number;
        trade_history_page_size: number;
        stock_trades_count: number;
    };
    ui: {
        orderbook_depth: number;
        leaderboard_entries: number;
        chat_messages_visible: number;
        trade_history_widget: number;
        stock_trades_widget: number;
    };
    animation: {
        news_ticker_base_duration: number;
        news_ticker_per_item: number;
    };
    trading: {
        default_order_qty: number;
        short_margin_percent: number;
    };
    game_defaults: {
        target_networth: number;
        shares_per_trader: number;
        trading_start_time: string;
        trading_end_time: string;
        circuit_breaker_threshold: number;
        circuit_breaker_duration: number;
    };
    polling: {
        dashboard_metrics_interval: number;
    };
    company_form: {
        symbol_max_length: number;
        volatility_min: number;
        volatility_max: number;
        volatility_step: number;
        default_total_shares: number;
        default_initial_price: number;
    };
    sectors: string[];
    labels: FrontendLabels;
    validation: ValidationRules;
}

// UI Labels from server
export interface FrontendLabels {
    app_name: string;
    app_tagline: string;
    auth: AuthLabels;
    trading: TradingLabels;
    admin: AdminLabels;
    common: CommonLabels;
}

export interface AuthLabels {
    login_title: string;
    login_subtitle: string;
    register_title: string;
    register_subtitle: string;
    regno_label: string;
    regno_placeholder: string;
    password_label: string;
    password_placeholder: string;
    confirm_password_label: string;
    name_label: string;
    name_placeholder: string;
    login_button: string;
    register_button: string;
    no_account_text: string;
    has_account_text: string;
    starting_balance_info: string;
}

export interface TradingLabels {
    order_book: string;
    portfolio: string;
    open_orders: string;
    trade_history: string;
    buy: string;
    sell: string;
    short: string;
    market: string;
    limit: string;
    quantity: string;
    price: string;
    total: string;
    bids: string;
    asks: string;
    no_bids: string;
    no_asks: string;
    no_trades: string;
    no_orders: string;
    cancel_order: string;
    cancel_all: string;
    confirm_order: string;
    gtc: string;
    gtc_full: string;
    ioc: string;
    ioc_full: string;
    short_margin_warning: string;
    market_order_info: string;
    no_liquidity: string;
    positions: string;
    holdings: string;
    cash: string;
    net_worth: string;
}

export interface AdminLabels {
    dashboard: string;
    game_control: string;
    traders: string;
    companies: string;
    diagnostics: string;
    market_open: string;
    market_closed: string;
    open_market: string;
    close_market: string;
    initialize_game: string;
    ban_trader: string;
    unban_trader: string;
    mute_trader: string;
    unmute_trader: string;
    create_company: string;
    mark_bankrupt: string;
}

export interface CommonLabels {
    loading: string;
    error: string;
    success: string;
    cancel: string;
    confirm: string;
    save: string;
    refresh: string;
    search: string;
    no_results: string;
    connected: string;
    disconnected: string;
    reconnecting: string;
    live: string;
    offline: string;
}

export interface ValidationRules {
    regno_min_length: number;
    regno_max_length: number;
    password_min_length: number;
    name_min_length: number;
    name_max_length: number;
    chat_message_max_length: number;
}

export interface ServerFrontendConstants {
    type: 'FrontendConstants';
    payload: {
        constants: FrontendConstants;
    };
}

// ==================== NEW UI-READY MESSAGE TYPES ====================

// UI-ready portfolio item with pre-computed values
export interface PortfolioItemUI {
    symbol: string;
    qty: Quantity;
    short_qty: Quantity;
    locked_qty: Quantity;
    average_buy_price: RawPrice;
    current_price: RawPrice;
    market_value: RawPrice;
    cost_basis: RawPrice;
    unrealized_pnl: RawPrice;
    unrealized_pnl_percent: number;
    short_market_value: RawPrice;
    short_unrealized_pnl: RawPrice;
}

// UI-ready open order
export interface OpenOrderUI {
    order_id: OrderId;
    symbol: string;
    side: OrderSide;
    order_type: OrderType;
    qty: Quantity;
    filled_qty: Quantity;
    remaining_qty: Quantity;
    price: RawPrice;
    status: string;
    timestamp: number;
    time_in_force: TimeInForce;
}

// UI-ready market index with change data
export interface MarketIndexUI {
    name: string;
    value: RawPrice;
    previous_value: RawPrice;
    change: RawPrice;
    change_percent: number;
    timestamp: number;
}

// UI-ready leaderboard entry with user_id and rank change
export interface LeaderboardEntryUI {
    rank: number;
    user_id: UserId;
    name: string;
    net_worth: RawPrice;
    change_rank: number;  // Positive = moved up, negative = moved down
}

// Trade history item
export interface TradeHistoryItem {
    trade_id: number;
    symbol: string;
    side: string;
    qty: Quantity;
    price: RawPrice;
    total_value: RawPrice;
    counterparty_id: UserId | null;
    counterparty_name: string | null;
    timestamp: number;
}

// UI-ready company info
export interface CompanyUI {
    id: number;
    symbol: string;
    name: string;
    sector: string;
    current_price: RawPrice | null;
    price_change: RawPrice | null;
    price_change_percent: number | null;
    volume: Quantity;
    bankrupt: boolean;
}

// UI-ready news item
export interface NewsItemUI {
    id: string;
    headline: string;
    symbol: string | null;
    sentiment: string;
    impact: string;
    timestamp: number;
}

// UI-ready orderbook level
export interface OrderbookLevelUI {
    price: RawPrice;
    qty: Quantity;
    order_count: number;
    cumulative_qty: Quantity;
}

// UI-ready orderbook
export interface OrderbookUI {
    symbol: string;
    bids: OrderbookLevelUI[];
    asks: OrderbookLevelUI[];
    spread: RawPrice | null;
    spread_percent: number | null;
    last_price: RawPrice | null;
    timestamp: number;
}

// UI-ready candle
export interface CandleUI {
    timestamp: number;
    open: RawPrice;
    high: RawPrice;
    low: RawPrice;
    close: RawPrice;
    volume: Quantity;
}

// Portfolio state for full sync
export interface PortfolioStateUI {
    money: RawPrice;
    locked_money: RawPrice;
    margin_locked: RawPrice;
    total_available: RawPrice;
    portfolio_value: RawPrice;
    net_worth: RawPrice;
    items: PortfolioItemUI[];
}

// Chat message (reusing existing structure)
export interface ChatMessageUI {
    id: string;
    user_id: UserId;
    username: string;
    message: string;
    timestamp: number;
}

// Full state sync payload
export interface FullStateSyncPayload {
    market_open: boolean;
    halted_symbols: Array<[string, number]>;
    companies: CompanyUI[];
    portfolio: PortfolioStateUI | null;
    open_orders: OpenOrderUI[];
    indices: MarketIndexUI[];
    leaderboard: LeaderboardEntryUI[];
    news: NewsItemUI[];
    chat_history: ChatMessageUI[];
    active_symbol: string | null;
    orderbook: OrderbookUI | null;
    candles: CandleUI[] | null;
    recent_trades: TradeHistoryItem[];
    sync_id: number;
    timestamp: number;
}

// Full state sync message
export interface ServerFullStateSync {
    type: 'FullStateSync';
    payload: {
        payload: FullStateSyncPayload;
    };
}

// Enhanced portfolio with pre-computed values
export interface ServerPortfolioUpdateUI {
    type: 'PortfolioUpdateUI';
    payload: {
        money: RawPrice;
        locked_money: RawPrice;
        margin_locked: RawPrice;
        portfolio_value: RawPrice;
        net_worth: RawPrice;
        items: PortfolioItemUI[];
    };
}

// Open orders list update
export interface ServerOpenOrdersUpdate {
    type: 'OpenOrdersUpdate';
    payload: {
        orders: OpenOrderUI[];
    };
}

// UI-ready index update
export interface ServerIndexUpdateUI {
    type: 'IndexUpdateUI';
    payload: {
        index: MarketIndexUI;
    };
}

// UI-ready leaderboard update
export interface ServerLeaderboardUpdateUI {
    type: 'LeaderboardUpdateUI';
    payload: {
        entries: LeaderboardEntryUI[];
    };
}

// Trade history response
export interface ServerTradeHistory {
    type: 'TradeHistory';
    payload: {
        trades: TradeHistoryItem[];
        total_count: number;
        page: number;
        page_size: number;
        has_more: boolean;
    };
}

// Stock trade history response
export interface ServerStockTradeHistory {
    type: 'StockTradeHistory';
    payload: {
        symbol: string;
        trades: TradeHistoryItem[];
    };
}

// Admin trade history item with both parties visible
export interface AdminTradeHistoryItem {
    trade_id: number;
    symbol: string;
    buyer_id: UserId;
    buyer_name: string;
    buyer_side: string;
    seller_id: UserId;
    seller_name: string;
    seller_side: string;
    qty: Quantity;
    price: RawPrice;
    total_value: RawPrice;
    timestamp: number;
}

// Admin open order with user info
export interface AdminOpenOrderUI {
    order_id: OrderId;
    user_id: UserId;
    user_name: string;
    symbol: string;
    side: OrderSide;
    order_type: OrderType;
    qty: Quantity;
    filled_qty: Quantity;
    remaining_qty: Quantity;
    price: RawPrice;
    status: string;
    timestamp: number;
    time_in_force: TimeInForce;
}

// Admin dashboard metrics
export interface AdminDashboardMetrics {
    total_traders: number;
    active_traders: number;
    total_trades: number;
    total_volume: RawPrice;
    recent_volume: RawPrice;
    total_market_cap: RawPrice;
    halted_symbols_count: number;
    open_orders_count: number;
    market_open: boolean;
    timestamp: number;
    // Server metrics
    server_uptime_secs: number;
    active_sessions: ActiveSessionInfo[];
}

// Active session info for admin dashboard
export interface ActiveSessionInfo {
    session_id: number;
    user_id: number;
    user_name: string;
    connected_at: number;  // Unix timestamp
    last_activity: number; // Unix timestamp
    messages_sent: number;
}

// Admin trade history response
export interface ServerAdminTradeHistory {
    type: 'AdminTradeHistory';
    payload: {
        trades: AdminTradeHistoryItem[];
        total_count: number;
        page: number;
        page_size: number;
        has_more: boolean;
    };
}

// Admin open orders response
export interface ServerAdminOpenOrders {
    type: 'AdminOpenOrders';
    payload: {
        orders: AdminOpenOrderUI[];
        total_count: number;
    };
}

// Admin dashboard metrics response
export interface ServerAdminDashboardMetrics {
    type: 'AdminDashboardMetrics';
    payload: {
        metrics: AdminDashboardMetrics;
    };
}

// Admin orderbook response
export interface ServerAdminOrderbook {
    type: 'AdminOrderbook';
    payload: {
        symbol: string;
        bids: AdminOpenOrderUI[];
        asks: AdminOpenOrderUI[];
    };
}

// Component sync responses
export interface ServerPortfolioSync {
    type: 'PortfolioSync';
    payload: {
        sync_id: number;
        money: RawPrice;
        locked_money: RawPrice;
        margin_locked: RawPrice;
        portfolio_value: RawPrice;
        net_worth: RawPrice;
        items: PortfolioItemUI[];
    };
}

export interface ServerOpenOrdersSync {
    type: 'OpenOrdersSync';
    payload: {
        sync_id: number;
        orders: OpenOrderUI[];
    };
}

export interface ServerLeaderboardSync {
    type: 'LeaderboardSync';
    payload: {
        sync_id: number;
        entries: LeaderboardEntryUI[];
    };
}

export interface ServerIndicesSync {
    type: 'IndicesSync';
    payload: {
        sync_id: number;
        indices: MarketIndexUI[];
    };
}

export interface ServerOrderbookSync {
    type: 'OrderbookSync';
    payload: {
        sync_id: number;
        symbol: string;
        orderbook: OrderbookUI;
    };
}

export interface ServerCandlesSync {
    type: 'CandlesSync';
    payload: {
        sync_id: number;
        symbol: string;
        candles: CandleUI[];
    };
}

export interface ServerNewsSync {
    type: 'NewsSync';
    payload: {
        sync_id: number;
        news: NewsItemUI[];
    };
}

export interface ServerChatSync {
    type: 'ChatSync';
    payload: {
        sync_id: number;
        messages: ChatMessageUI[];
    };
}

export type ServerMessage =
    | ServerAuthSuccess
    | ServerAuthFailed
    | ServerRegisterSuccess
    | ServerRegisterFailed
    | ServerOrderAck
    | ServerOrderRejected
    | ServerOrderCancelled
    | ServerTradeUpdate
    | ServerCandleUpdate
    | ServerDepthUpdate
    | ServerIndexUpdate
    | ServerPortfolioUpdate
    | ServerCircuitBreaker
    | ServerMarketStatus
    | ServerNewsUpdate
    | ServerLeaderboardUpdate
    | ServerChatUpdate
    | ServerError
    | ServerPong
    | ServerSystem
    | ServerCompanyList
    | ServerConfig
    | ServerFrontendConstants
    // New UI-ready messages
    | ServerFullStateSync
    | ServerPortfolioUpdateUI
    | ServerOpenOrdersUpdate
    | ServerIndexUpdateUI
    | ServerLeaderboardUpdateUI
    | ServerTradeHistory
    | ServerStockTradeHistory
    | ServerAdminTradeHistory
    | ServerAdminOpenOrders
    | ServerAdminDashboardMetrics
    | ServerAdminOrderbook
    // Component sync messages
    | ServerPortfolioSync
    | ServerOpenOrdersSync
    | ServerLeaderboardSync
    | ServerIndicesSync
    | ServerOrderbookSync
    | ServerCandlesSync
    | ServerNewsSync
    | ServerChatSync;

// === Message Type Extractors ===
export type ServerMessageType = ServerMessage['type'];
export type ClientMessageType = ClientMessage['type'];

// === Admin Actions ===
export type AdminAction =
    | 'ToggleMarket'
    | 'SetVolatility'
    | 'CreateCompany'
    | 'SetBankrupt'
    | 'BanUser'
    | 'UnbanUser'
    | 'DisableChat'
    | 'EnableChat'
    | 'InitializeGame'
    | 'EndGame';
