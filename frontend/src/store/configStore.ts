// ============================================
// Config Store
// Manages application configuration from server
// including currency formatting and UI constants
// ============================================

import { create } from 'zustand';
import websocketService from '../services/websocket';
import type { FrontendConstants } from '../types/api';
import { logger } from '../utils';

export interface CurrencyConfig {
    symbol: string;
    code: string;
    locale: string;
    decimals: number;
    symbol_position: 'before' | 'after';
}

// Default frontend constants (fallback values)
const defaultFrontendConstants: FrontendConstants = {
    limits: {
        trades_history: 100,
        candles_per_symbol: 200,
        chat_messages: 50,
        news_items: 10,
        trade_history_page_size: 50,
        stock_trades_count: 50,
    },
    ui: {
        orderbook_depth: 8,
        leaderboard_entries: 10,
        chat_messages_visible: 20,
        trade_history_widget: 20,
        stock_trades_widget: 15,
    },
    animation: {
        news_ticker_base_duration: 20,
        news_ticker_per_item: 5,
    },
    trading: {
        default_order_qty: 10,
        short_margin_percent: 150,
    },
    game_defaults: {
        target_networth: 200000,
        shares_per_trader: 100,
        trading_start_time: '09:00',
        trading_end_time: '16:00',
        circuit_breaker_threshold: 10,
        circuit_breaker_duration: 300,
    },
    polling: {
        dashboard_metrics_interval: 10000,
    },
    company_form: {
        symbol_max_length: 5,
        volatility_min: 0.1,
        volatility_max: 0.5,
        volatility_step: 0.05,
        default_total_shares: 1000000,
        default_initial_price: 1000000, // scaled price
    },
    sectors: ['Tech', 'Finance', 'Healthcare', 'Energy', 'Consumer', 'Industrial'],
    labels: {
        app_name: 'StockMart',
        app_tagline: 'Virtual Trading Simulation',
        auth: {
            login_title: 'Welcome Back',
            login_subtitle: 'Sign in to continue trading',
            register_title: 'Create Account',
            register_subtitle: 'Join the trading simulation',
            regno_label: 'Registration Number',
            regno_placeholder: 'Enter your registration ID',
            password_label: 'Password',
            password_placeholder: 'Enter your password',
            confirm_password_label: 'Confirm Password',
            name_label: 'Display Name',
            name_placeholder: "How you'll appear to others",
            login_button: 'Sign In',
            register_button: 'Create Account',
            no_account_text: "Don't have an account?",
            has_account_text: 'Already have an account?',
            starting_balance_info: "You'll receive $100,000 in virtual cash to start trading!",
        },
        trading: {
            order_book: 'Order Book',
            portfolio: 'Portfolio',
            open_orders: 'Open Orders',
            trade_history: 'Trade History',
            buy: 'Buy',
            sell: 'Sell',
            short: 'Short',
            market: 'Market',
            limit: 'Limit',
            quantity: 'Quantity',
            price: 'Price',
            total: 'Total',
            bids: 'Bids',
            asks: 'Asks',
            no_bids: 'No bids',
            no_asks: 'No asks',
            no_trades: 'No trades yet',
            no_orders: 'No open orders',
            cancel_order: 'Cancel',
            cancel_all: 'Cancel All',
            confirm_order: 'Confirm Order',
            gtc: 'GTC',
            gtc_full: "Good 'Til Cancelled",
            ioc: 'IOC',
            ioc_full: 'Immediate or Cancel',
            short_margin_warning: 'Short selling requires 150% margin coverage',
            market_order_info: 'Market orders execute immediately at best available price',
            no_liquidity: 'No Market Liquidity',
            positions: 'Positions',
            holdings: 'Holdings',
            cash: 'Cash',
            net_worth: 'Net Worth',
        },
        admin: {
            dashboard: 'Dashboard',
            game_control: 'Game Control',
            traders: 'Traders',
            companies: 'Companies',
            diagnostics: 'Diagnostics',
            market_open: 'Market Open',
            market_closed: 'Market Closed',
            open_market: 'Open Market',
            close_market: 'Close Market',
            initialize_game: 'Initialize Game',
            ban_trader: 'Ban Trader',
            unban_trader: 'Unban Trader',
            mute_trader: 'Mute',
            unmute_trader: 'Unmute',
            create_company: 'Create Company',
            mark_bankrupt: 'Mark Bankrupt',
        },
        common: {
            loading: 'Loading...',
            error: 'Error',
            success: 'Success',
            cancel: 'Cancel',
            confirm: 'Confirm',
            save: 'Save',
            refresh: 'Refresh',
            search: 'Search',
            no_results: 'No results found',
            connected: 'Connected',
            disconnected: 'Disconnected',
            reconnecting: 'Reconnecting...',
            live: 'Live',
            offline: 'Offline',
        },
    },
    validation: {
        regno_min_length: 3,
        regno_max_length: 20,
        password_min_length: 4,
        name_min_length: 2,
        name_max_length: 50,
        chat_message_max_length: 500,
    },
};

// Default currency config (USD)
const defaultCurrency: CurrencyConfig = {
    symbol: '$',
    code: 'USD',
    locale: 'en-US',
    decimals: 2,
    symbol_position: 'before',
};

interface ConfigState {
    // Server config
    registrationMode: string;
    chatEnabled: boolean;
    currency: CurrencyConfig;
    configLoaded: boolean;

    // Frontend constants from server
    constants: FrontendConstants;
    constantsLoaded: boolean;

    // Cached formatter
    _formatter: Intl.NumberFormat | null;

    // Actions
    setConfig: (config: {
        registration_mode: string;
        chat_enabled: boolean;
        currency: CurrencyConfig;
    }) => void;
    setConstants: (constants: FrontendConstants) => void;

    // Currency formatting
    formatCurrency: (value: number) => string;
    formatNumber: (value: number) => string;

    // Getters for constants (with proper typing)
    getLimits: () => FrontendConstants['limits'];
    getUI: () => FrontendConstants['ui'];
    getAnimation: () => FrontendConstants['animation'];
    getTrading: () => FrontendConstants['trading'];
    getGameDefaults: () => FrontendConstants['game_defaults'];
    getPolling: () => FrontendConstants['polling'];
    getCompanyForm: () => FrontendConstants['company_form'];
    getSectors: () => string[];
    getLabels: () => FrontendConstants['labels'];
    getValidation: () => FrontendConstants['validation'];
}

export const useConfigStore = create<ConfigState>((set, get) => ({
    registrationMode: 'Free',
    chatEnabled: true,
    currency: defaultCurrency,
    configLoaded: false,
    constants: defaultFrontendConstants,
    constantsLoaded: false,
    _formatter: null,

    setConfig: (config) => {
        logger.debug('ConfigStore', 'Setting config', config);

        // Create a new formatter based on the config
        let formatter: Intl.NumberFormat;
        try {
            formatter = new Intl.NumberFormat(config.currency.locale, {
                style: 'decimal',
                minimumFractionDigits: config.currency.decimals,
                maximumFractionDigits: config.currency.decimals,
            });
        } catch (e) {
            logger.warn('ConfigStore', 'Failed to create formatter, using fallback', e);
            formatter = new Intl.NumberFormat('en-US', {
                style: 'decimal',
                minimumFractionDigits: 2,
                maximumFractionDigits: 2,
            });
        }

        set({
            registrationMode: config.registration_mode,
            chatEnabled: config.chat_enabled,
            currency: config.currency,
            configLoaded: true,
            _formatter: formatter,
        });
    },

    setConstants: (constants) => {
        logger.debug('ConfigStore', 'Setting frontend constants', constants);
        set({
            constants,
            constantsLoaded: true,
        });
    },

    formatCurrency: (value: number) => {
        const state = get();
        const { currency, _formatter } = state;

        let formatted: string;
        if (_formatter) {
            formatted = _formatter.format(value);
        } else {
            formatted = value.toFixed(currency.decimals);
        }

        if (currency.symbol_position === 'before') {
            return `${currency.symbol}${formatted}`;
        } else {
            return `${formatted}${currency.symbol}`;
        }
    },

    formatNumber: (value: number) => {
        const state = get();
        const { _formatter } = state;

        if (_formatter) {
            return _formatter.format(value);
        }
        return value.toFixed(2);
    },

    // Getters for constants
    getLimits: () => get().constants.limits,
    getUI: () => get().constants.ui,
    getAnimation: () => get().constants.animation,
    getTrading: () => get().constants.trading,
    getGameDefaults: () => get().constants.game_defaults,
    getPolling: () => get().constants.polling,
    getCompanyForm: () => get().constants.company_form,
    getSectors: () => get().constants.sectors,
    getLabels: () => get().constants.labels,
    getValidation: () => get().constants.validation,
}));

// Listen for Config messages from server
websocketService.on('Config', (payload: {
    registration_mode: string;
    chat_enabled: boolean;
    currency: CurrencyConfig;
}) => {
    useConfigStore.getState().setConfig(payload);
});

// Listen for FrontendConstants messages from server
websocketService.on('FrontendConstants', (payload: {
    constants: FrontendConstants;
}) => {
    useConfigStore.getState().setConstants(payload.constants);
});

export default useConfigStore;
