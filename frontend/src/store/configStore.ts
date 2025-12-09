// ============================================
// Config Store
// Manages application configuration from server
// including currency formatting settings
// ============================================

import { create } from 'zustand';
import websocketService from '../services/websocket';

export interface CurrencyConfig {
    symbol: string;
    code: string;
    locale: string;
    decimals: number;
    symbol_position: 'before' | 'after';
}

interface ConfigState {
    // Server config
    registrationMode: string;
    chatEnabled: boolean;
    currency: CurrencyConfig;
    configLoaded: boolean;

    // Cached formatter
    _formatter: Intl.NumberFormat | null;

    // Actions
    setConfig: (config: {
        registration_mode: string;
        chat_enabled: boolean;
        currency: CurrencyConfig;
    }) => void;

    // Currency formatting
    formatCurrency: (value: number) => string;
    formatNumber: (value: number) => string;
}

// Default currency config (USD)
const defaultCurrency: CurrencyConfig = {
    symbol: '$',
    code: 'USD',
    locale: 'en-US',
    decimals: 2,
    symbol_position: 'before',
};

export const useConfigStore = create<ConfigState>((set, get) => ({
    registrationMode: 'Free',
    chatEnabled: true,
    currency: defaultCurrency,
    configLoaded: false,
    _formatter: null,

    setConfig: (config) => {
        console.log('[ConfigStore] Setting config:', config);

        // Create a new formatter based on the config
        let formatter: Intl.NumberFormat;
        try {
            // Try to use Intl.NumberFormat with full currency styling
            formatter = new Intl.NumberFormat(config.currency.locale, {
                style: 'decimal',
                minimumFractionDigits: config.currency.decimals,
                maximumFractionDigits: config.currency.decimals,
            });
        } catch (e) {
            console.warn('[ConfigStore] Failed to create formatter, using fallback:', e);
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

    formatCurrency: (value: number) => {
        const state = get();
        const { currency, _formatter } = state;

        // Use cached formatter or create default
        let formatted: string;
        if (_formatter) {
            formatted = _formatter.format(value);
        } else {
            formatted = value.toFixed(currency.decimals);
        }

        // Add currency symbol
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
}));

// Listen for Config messages from server
websocketService.on('Config', (payload: {
    registration_mode: string;
    chat_enabled: boolean;
    currency: CurrencyConfig;
}) => {
    useConfigStore.getState().setConfig(payload);
});

export default useConfigStore;
