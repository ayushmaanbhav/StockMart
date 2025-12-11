// ============================================
// Game State Store
// Manages market data, portfolio, orders, etc.
// ============================================

import { create } from 'zustand';
import type {
    Trade,
    Candle,
    OrderBookDepth,
    MarketIndex,
    NewsItem,
    LeaderboardEntry,
    ChatMessage,
    PortfolioItem,
    Order,
    Price,
} from '../types/models';
import { PRICE_SCALE } from '../types/models';
import { LIMITS } from '../constants';
import type {
    ServerPortfolioUpdate,
    ServerTradeUpdate,
    ServerCandleUpdate,
    ServerDepthUpdate,
    ServerIndexUpdate,
    ServerNewsUpdate,
    ServerLeaderboardUpdate,
    ServerChatUpdate,
    ServerCircuitBreaker,
    ServerMarketStatus,
    ServerOrderAck,
    ServerOrderCancelled,
    ServerOrderRejected,
    ServerError,
    // New UI-ready message types
    ServerPortfolioUpdateUI,
    ServerOpenOrdersUpdate,
    ServerIndexUpdateUI,
    ServerLeaderboardUpdateUI,
    ServerTradeHistory,
    ServerStockTradeHistory,
    TradeHistoryItem,
    FullStateSyncPayload,
    // Component sync types
    PortfolioItemUI,
    OpenOrderUI,
    MarketIndexUI,
    LeaderboardEntryUI,
} from '../types/api';
import websocketService from '../services/websocket';
import { useAuthStore } from './authStore';
import { useUIStore } from './uiStore';
import { loggers } from '../utils';

const log = loggers.gameStore;

interface Company {
    id: number;
    symbol: string;
    name: string;
    sector: string;
    volatility: number;
}

interface GameState {
    // Connection
    isConnected: boolean;

    // Market
    marketOpen: boolean;
    haltedSymbols: Record<string, number>; // symbol -> halted_until timestamp

    // Companies
    companies: Company[];

    // Portfolio
    money: Price;
    lockedMoney: Price;
    marginLocked: Price;
    netWorth: Price;
    portfolio: PortfolioItem[];

    // Orders
    openOrders: Order[];
    pendingOrder: {
        symbol: string;
        side: 'Buy' | 'Sell' | 'Short';
        orderType: 'Market' | 'Limit';
        qty: number;
        price: number;
    } | null;

    // Market Data
    trades: Trade[];
    candles: Record<string, Candle[]>;
    orderBooks: Record<string, OrderBookDepth>;
    indices: Record<string, MarketIndex>;

    // Social
    leaderboard: LeaderboardEntry[];
    news: NewsItem[];
    chatMessages: ChatMessage[];

    // Active Symbol
    activeSymbol: string;
    subscribedSymbols: string[];

    // Trade History
    tradeHistory: TradeHistoryItem[];
    tradeHistoryTotal: number;
    tradeHistoryPage: number;
    tradeHistoryHasMore: boolean;
    stockTradeHistory: Record<string, TradeHistoryItem[]>; // symbol -> trades

    // Loading/Sync State
    syncId: number;
    lastSyncTimestamp: number;
    isFullSyncComplete: boolean;
    loading: {
        portfolio: boolean;
        orders: boolean;
        leaderboard: boolean;
        indices: boolean;
        orderbook: boolean;
        candles: boolean;
        news: boolean;
        chat: boolean;
        tradeHistory: boolean;
    };

    // Actions
    setConnected: (connected: boolean) => void;
    setActiveSymbol: (symbol: string) => void;
    subscribeSymbol: (symbol: string) => void;
    sendChatMessage: (message: string) => void;

    // Order actions
    placeOrder: (order: {
        symbol: string;
        side: 'Buy' | 'Sell' | 'Short';
        orderType: 'Market' | 'Limit';
        qty: number;
        price: number;
        timeInForce?: 'GTC' | 'IOC';
    }) => void;
    cancelOrder: (symbol: string, orderId: number) => void;

    // Sync actions
    requestSync: (component?: string) => void;
    requestTradeHistory: (page?: number, symbol?: string) => void;
    requestStockTrades: (symbol: string, count?: number) => void;
    setLoading: (component: keyof GameState['loading'], isLoading: boolean) => void;

    // Internal handlers
    _setCompanies: (companies: Company[]) => void;
    _updatePortfolio: (payload: ServerPortfolioUpdate['payload']) => void;
    _addTrade: (payload: ServerTradeUpdate['payload']) => void;
    _updateCandle: (payload: ServerCandleUpdate['payload']) => void;
    _updateDepth: (payload: ServerDepthUpdate['payload']) => void;
    _updateIndex: (payload: ServerIndexUpdate['payload']) => void;
    _addNews: (payload: ServerNewsUpdate['payload']) => void;
    _updateLeaderboard: (payload: ServerLeaderboardUpdate['payload']) => void;
    _addChatMessage: (payload: ServerChatUpdate['payload']) => void;
    _setCircuitBreaker: (payload: ServerCircuitBreaker['payload']) => void;
    _setMarketStatus: (payload: ServerMarketStatus['payload']) => void;
    _handleOrderAck: (payload: ServerOrderAck['payload']) => void;
    _handleOrderCancelled: (payload: ServerOrderCancelled['payload']) => void;
    _handleOrderRejected: (payload: ServerOrderRejected['payload']) => void;
    _handleError: (payload: ServerError['payload']) => void;

    // New UI-ready handlers
    _handleFullStateSync: (payload: FullStateSyncPayload) => void;
    _handlePortfolioUpdateUI: (payload: ServerPortfolioUpdateUI['payload']) => void;
    _handleOpenOrdersUpdate: (payload: ServerOpenOrdersUpdate['payload']) => void;
    _handleIndexUpdateUI: (payload: ServerIndexUpdateUI['payload']) => void;
    _handleLeaderboardUpdateUI: (payload: ServerLeaderboardUpdateUI['payload']) => void;
    _handleTradeHistory: (payload: ServerTradeHistory['payload']) => void;
    _handleStockTradeHistory: (payload: ServerStockTradeHistory['payload']) => void;
}

export const useGameStore = create<GameState>((set, get) => ({
    // Initial state
    isConnected: false,
    marketOpen: true,
    haltedSymbols: {},

    companies: [],

    money: 0,
    lockedMoney: 0,
    marginLocked: 0,
    netWorth: 0,
    portfolio: [],
    openOrders: [],
    pendingOrder: null,

    trades: [],
    candles: {},
    orderBooks: {},
    indices: {},

    leaderboard: [],
    news: [],
    chatMessages: [],

    activeSymbol: '',  // Empty until companies are loaded
    subscribedSymbols: [],

    // Trade History
    tradeHistory: [],
    tradeHistoryTotal: 0,
    tradeHistoryPage: 0,
    tradeHistoryHasMore: false,
    stockTradeHistory: {},

    // Loading/Sync State
    syncId: 0,
    lastSyncTimestamp: 0,
    isFullSyncComplete: false,
    loading: {
        portfolio: false,
        orders: false,
        leaderboard: false,
        indices: false,
        orderbook: false,
        candles: false,
        news: false,
        chat: false,
        tradeHistory: false,
    },

    // === Actions ===

    setConnected: (connected) => {
        log.debug('Connection status:', connected);
        set({ isConnected: connected });
    },

    setActiveSymbol: (symbol) => {
        const current = get().activeSymbol;
        if (current !== symbol) {
            log.debug('Active symbol changed:', current, '->', symbol);
            set({ activeSymbol: symbol });
            get().subscribeSymbol(symbol);
        }
    },

    subscribeSymbol: (symbol) => {
        const subscribed = get().subscribedSymbols;
        if (!subscribed.includes(symbol)) {
            log.debug('Subscribing to symbol:', symbol);
            websocketService.send({ type: 'Subscribe', payload: { symbol } });
            set({ subscribedSymbols: [...subscribed, symbol] });
        }
    },

    sendChatMessage: (message) => {
        log.debug('Sending chat message:', message);
        websocketService.send({ type: 'Chat', payload: { message } });
    },

    placeOrder: (order) => {
        log.debug('Placing order:', order);
        // Store pending order to merge with OrderAck
        set({ pendingOrder: order });

        websocketService.send({
            type: 'PlaceOrder',
            payload: {
                symbol: order.symbol,
                side: order.side,
                order_type: order.orderType,
                qty: order.qty,
                price: Math.round(order.price * PRICE_SCALE),
                time_in_force: order.timeInForce || 'GTC',
            }
        });
    },

    cancelOrder: (symbol, orderId) => {
        log.debug('Cancelling order:', orderId, 'for', symbol);
        websocketService.send({
            type: 'CancelOrder',
            payload: { symbol, order_id: orderId }
        });
    },

    // === Sync Actions ===

    requestSync: (component) => {
        log.debug('Requesting sync:', component || 'full');

        // Set loading state for the component being synced
        if (component) {
            const componentMap: Record<string, keyof GameState['loading']> = {
                'portfolio': 'portfolio',
                'orders': 'orders',
                'leaderboard': 'leaderboard',
                'indices': 'indices',
                'news': 'news',
                'chat': 'chat',
                'trade_history': 'tradeHistory',
            };
            // Handle symbol-specific components
            if (component.startsWith('orderbook:')) {
                get().setLoading('orderbook', true);
            } else if (component.startsWith('candles:')) {
                get().setLoading('candles', true);
            } else if (componentMap[component]) {
                get().setLoading(componentMap[component], true);
            }
        } else {
            // Full sync - set all loading states
            set({
                loading: {
                    portfolio: true,
                    orders: true,
                    leaderboard: true,
                    indices: true,
                    orderbook: true,
                    candles: true,
                    news: true,
                    chat: true,
                    tradeHistory: true,
                }
            });
        }

        websocketService.send({
            type: 'RequestSync',
            payload: { component }
        });
    },

    requestTradeHistory: (page, symbol) => {
        log.debug('Requesting trade history:', { page, symbol });
        websocketService.send({
            type: 'GetTradeHistory',
            payload: { page, symbol, page_size: 50 }
        });
    },

    requestStockTrades: (symbol, count) => {
        log.debug('Requesting stock trades:', { symbol, count });
        websocketService.send({
            type: 'GetStockTrades',
            payload: { symbol, count: count || 50 }
        });
    },

    setLoading: (component, isLoading) => {
        set((state) => ({
            loading: { ...state.loading, [component]: isLoading }
        }));
    },

    // === Internal Handlers ===

    _setCompanies: (companies) => {
        log.debug('Setting companies:', companies.length, companies.map(c => c.symbol));
        const currentSymbol = get().activeSymbol;
        // If no active symbol yet, set to first company
        const newActiveSymbol = currentSymbol || (companies.length > 0 ? companies[0].symbol : '');
        set({ companies });
        if (newActiveSymbol && newActiveSymbol !== currentSymbol) {
            get().setActiveSymbol(newActiveSymbol);
        }
    },

    _updatePortfolio: (payload) => {
        log.debug('Portfolio update:', {
            money: payload.money / PRICE_SCALE,
            locked: payload.locked / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
            positions: payload.items.length
        });
        set({
            money: payload.money / PRICE_SCALE,
            lockedMoney: payload.locked / PRICE_SCALE,
            marginLocked: payload.margin_locked / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
            portfolio: payload.items.map(item => ({
                userId: item.user_id,
                symbol: item.symbol,
                qty: item.qty,
                shortQty: item.short_qty,
                lockedQty: item.locked_qty,
                averageBuyPrice: item.average_buy_price / PRICE_SCALE,
            }))
        });

        // Also update auth store user
        useAuthStore.getState().setUser({
            money: payload.money / PRICE_SCALE,
            lockedMoney: payload.locked / PRICE_SCALE,
            marginLocked: payload.margin_locked / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
        });
    },

    _addTrade: (payload) => {
        const trade: Trade = {
            id: Date.now(),
            symbol: payload.symbol,
            qty: payload.qty,
            price: payload.price / PRICE_SCALE,
            timestamp: payload.timestamp * 1000,
        };
        set((state) => ({
            trades: [trade, ...state.trades].slice(0, LIMITS.TRADES_HISTORY)
        }));
    },

    _updateCandle: (payload) => {
        const candle: Candle = {
            symbol: payload.candle.symbol,
            resolution: payload.candle.resolution,
            open: payload.candle.open / PRICE_SCALE,
            high: payload.candle.high / PRICE_SCALE,
            low: payload.candle.low / PRICE_SCALE,
            close: payload.candle.close / PRICE_SCALE,
            volume: payload.candle.volume,
            timestamp: payload.candle.timestamp * 1000,
        };

        set((state) => {
            const currentCandles = state.candles[payload.symbol] || [];
            const lastCandle = currentCandles[currentCandles.length - 1];

            if (lastCandle && lastCandle.timestamp === candle.timestamp) {
                // Update existing candle
                const updated = [...currentCandles];
                updated[updated.length - 1] = candle;
                return { candles: { ...state.candles, [payload.symbol]: updated } };
            } else {
                // Add new candle
                return {
                    candles: {
                        ...state.candles,
                        [payload.symbol]: [...currentCandles, candle].slice(-LIMITS.CANDLES_PER_SYMBOL)
                    }
                };
            }
        });
    },

    _updateDepth: (payload) => {
        set((state) => ({
            orderBooks: {
                ...state.orderBooks,
                [payload.symbol]: {
                    symbol: payload.symbol,
                    bids: payload.bids.map(([price, qty]) => ({ price: price / PRICE_SCALE, quantity: qty })),
                    asks: payload.asks.map(([price, qty]) => ({ price: price / PRICE_SCALE, quantity: qty })),
                    spread: payload.spread ? payload.spread / PRICE_SCALE : null,
                }
            }
        }));
    },

    _updateIndex: (payload) => {
        const currentIndex = get().indices[payload.name];
        const newValue = payload.value / PRICE_SCALE;
        const change = currentIndex ? newValue - currentIndex.value : 0;
        const changePercent = currentIndex && currentIndex.value !== 0
            ? (change / currentIndex.value) * 100
            : 0;

        set((state) => ({
            indices: {
                ...state.indices,
                [payload.name]: {
                    name: payload.name,
                    value: newValue,
                    timestamp: Date.now(),
                    change,
                    changePercent,
                }
            }
        }));
    },

    _addNews: (payload) => {
        const news: NewsItem = {
            id: payload.news.id,
            headline: payload.news.headline,
            sentiment: payload.news.sentiment,
            symbol: payload.news.symbol || undefined,
            timestamp: payload.news.timestamp * 1000,
        };
        // Limit news items to prevent ticker speed issues
        set((state) => ({
            news: [news, ...state.news].slice(0, LIMITS.NEWS_ITEMS)
        }));
    },

    _updateLeaderboard: (payload) => {
        log.debug('Leaderboard update:', payload.entries.length, 'entries');
        set({
            leaderboard: payload.entries.map(entry => ({
                rank: entry.rank,
                name: entry.name,
                netWorth: entry.net_worth / PRICE_SCALE,
            }))
        });
    },

    _addChatMessage: (payload) => {
        log.debug('Chat message from', payload.message.username);
        const msg: ChatMessage = {
            id: payload.message.id,
            userId: payload.message.user_id,
            username: payload.message.username,
            message: payload.message.message,
            timestamp: payload.message.timestamp * 1000,
        };
        set((state) => ({
            chatMessages: [...state.chatMessages, msg].slice(-LIMITS.CHAT_MESSAGES)
        }));
    },

    _setCircuitBreaker: (payload) => {
        log.debug('Circuit breaker:', payload.symbol, 'halted until', payload.halted_until);
        set((state) => ({
            haltedSymbols: {
                ...state.haltedSymbols,
                [payload.symbol]: payload.halted_until * 1000
            }
        }));
    },

    _setMarketStatus: (payload) => {
        log.debug('Market status:', payload.is_open ? 'OPEN' : 'CLOSED');
        set({ marketOpen: payload.is_open });
    },

    _handleOrderAck: (payload) => {
        const state = get();
        const pendingOrder = state.pendingOrder;

        log.debug('Order acknowledged:', payload);

        // If status is "Filled", the order was fully filled immediately - don't add to open orders
        if (payload.status === 'Filled') {
            set({ pendingOrder: null });
            useUIStore.getState().showToast({
                type: 'success',
                message: `Order filled: ${pendingOrder?.qty || payload.filled_qty} ${pendingOrder?.symbol || ''} @ $${pendingOrder?.price?.toFixed(2) || ''}`
            });
            return;
        }

        // If we have a pending order and status is Open or Partial, add to open orders
        if (pendingOrder && (payload.status === 'Open' || payload.status === 'Partial')) {
            const newOrder: Order = {
                id: payload.order_id,
                userId: 0, // We don't have this from the server
                symbol: pendingOrder.symbol,
                orderType: pendingOrder.orderType,
                side: pendingOrder.side,
                qty: pendingOrder.qty,
                filledQty: payload.filled_qty,
                price: pendingOrder.price,
                status: payload.status as 'Open' | 'Partial',
                timestamp: Date.now(),
                timeInForce: 'GTC',
            };

            set((state) => ({
                openOrders: [...state.openOrders, newOrder],
                pendingOrder: null,
            }));

            useUIStore.getState().showToast({
                type: 'success',
                message: `Order placed: ${pendingOrder.side} ${pendingOrder.qty} ${pendingOrder.symbol} @ $${pendingOrder.price.toFixed(2)}`
            });
        } else {
            set({ pendingOrder: null });
        }
    },

    _handleOrderCancelled: (payload) => {
        set((state) => ({
            openOrders: state.openOrders.filter(o => o.id !== payload.order_id)
        }));
        useUIStore.getState().showToast({
            type: 'info',
            message: `Order #${payload.order_id} cancelled`
        });
    },

    _handleOrderRejected: (payload) => {
        // Clear pending order
        set({ pendingOrder: null });
        // Show error toast
        useUIStore.getState().showToast({
            type: 'error',
            message: `Order rejected: ${payload.reason}`
        });
    },

    _handleError: (payload) => {
        useUIStore.getState().showToast({
            type: 'error',
            message: payload.message
        });
    },

    // === New UI-Ready Handlers ===

    _handleFullStateSync: (payload) => {
        log.debug('Full state sync received:', {
            syncId: payload.sync_id,
            companies: payload.companies?.length,
            portfolio: !!payload.portfolio,
            openOrders: payload.open_orders?.length,
            indices: payload.indices?.length,
            leaderboard: payload.leaderboard?.length,
            news: payload.news?.length,
        });

        // Convert companies
        const companies = (payload.companies || []).map(c => ({
            id: c.id,
            symbol: c.symbol,
            name: c.name,
            sector: c.sector,
            volatility: 0, // Not in CompanyUI
        }));

        // Convert portfolio items (values already pre-computed, just scale prices)
        const portfolioItems: PortfolioItem[] = payload.portfolio?.items?.map(item => ({
            userId: 0, // Not provided in UI model
            symbol: item.symbol,
            qty: item.qty,
            shortQty: item.short_qty,
            lockedQty: item.locked_qty,
            averageBuyPrice: item.average_buy_price / PRICE_SCALE,
            // Store pre-computed UI values
            currentPrice: item.current_price / PRICE_SCALE,
            marketValue: item.market_value / PRICE_SCALE,
            costBasis: item.cost_basis / PRICE_SCALE,
            unrealizedPnl: item.unrealized_pnl / PRICE_SCALE,
            unrealizedPnlPercent: item.unrealized_pnl_percent,
            shortMarketValue: item.short_market_value / PRICE_SCALE,
            shortUnrealizedPnl: item.short_unrealized_pnl / PRICE_SCALE,
        })) || [];

        // Convert open orders (values already pre-computed)
        const openOrders: Order[] = (payload.open_orders || []).map(o => ({
            id: o.order_id,
            userId: 0,
            symbol: o.symbol,
            orderType: o.order_type,
            side: o.side,
            qty: o.qty,
            filledQty: o.filled_qty,
            price: o.price / PRICE_SCALE,
            status: o.status as 'Open' | 'Partial',
            timestamp: o.timestamp * 1000,
            timeInForce: o.time_in_force,
        }));

        // Convert indices (values already pre-computed with change data)
        const indices: Record<string, MarketIndex> = {};
        for (const idx of (payload.indices || [])) {
            indices[idx.name] = {
                name: idx.name,
                value: idx.value / PRICE_SCALE,
                timestamp: idx.timestamp * 1000,
                change: idx.change / PRICE_SCALE,
                changePercent: idx.change_percent,
            };
        }

        // Convert leaderboard (values already pre-computed)
        const leaderboard: LeaderboardEntry[] = (payload.leaderboard || []).map(e => ({
            rank: e.rank,
            name: e.name,
            netWorth: e.net_worth / PRICE_SCALE,
            userId: e.user_id,
            changeRank: e.change_rank,
        }));

        // Convert news items
        const news: NewsItem[] = (payload.news || []).map(n => ({
            id: n.id,
            headline: n.headline,
            sentiment: n.sentiment as 'Bullish' | 'Bearish' | 'Neutral',
            symbol: n.symbol || undefined,
            timestamp: n.timestamp * 1000,
        }));

        // Convert chat messages
        const chatMessages: ChatMessage[] = (payload.chat_history || []).map(m => ({
            id: m.id,
            userId: m.user_id,
            username: m.username,
            message: m.message,
            timestamp: m.timestamp * 1000,
        }));

        // Convert halted symbols
        const haltedSymbols: Record<string, number> = {};
        for (const [symbol, timestamp] of (payload.halted_symbols || [])) {
            haltedSymbols[symbol] = timestamp * 1000;
        }

        // Convert candles if provided
        const candles: Record<string, Candle[]> = {};
        if (payload.candles && payload.active_symbol) {
            candles[payload.active_symbol] = payload.candles.map(c => ({
                symbol: payload.active_symbol!,
                resolution: '1m',
                open: c.open / PRICE_SCALE,
                high: c.high / PRICE_SCALE,
                low: c.low / PRICE_SCALE,
                close: c.close / PRICE_SCALE,
                volume: c.volume,
                timestamp: c.timestamp * 1000,
            }));
        }

        // Convert orderbook if provided
        const orderBooks: Record<string, OrderBookDepth> = {};
        if (payload.orderbook && payload.active_symbol) {
            orderBooks[payload.active_symbol] = {
                symbol: payload.orderbook.symbol,
                bids: payload.orderbook.bids.map(b => ({
                    price: b.price / PRICE_SCALE,
                    quantity: b.qty,
                })),
                asks: payload.orderbook.asks.map(a => ({
                    price: a.price / PRICE_SCALE,
                    quantity: a.qty,
                })),
                spread: payload.orderbook.spread ? payload.orderbook.spread / PRICE_SCALE : null,
            };
        }

        // Set active symbol
        const activeSymbol = payload.active_symbol ||
            (companies.length > 0 ? companies[0].symbol : '');

        // Update all state at once
        set({
            marketOpen: payload.market_open,
            haltedSymbols,
            companies,
            money: payload.portfolio?.money ? payload.portfolio.money / PRICE_SCALE : 0,
            lockedMoney: payload.portfolio?.locked_money ? payload.portfolio.locked_money / PRICE_SCALE : 0,
            marginLocked: payload.portfolio?.margin_locked ? payload.portfolio.margin_locked / PRICE_SCALE : 0,
            netWorth: payload.portfolio?.net_worth ? payload.portfolio.net_worth / PRICE_SCALE : 0,
            portfolio: portfolioItems,
            openOrders,
            indices,
            leaderboard,
            news,
            chatMessages,
            candles: { ...get().candles, ...candles },
            orderBooks: { ...get().orderBooks, ...orderBooks },
            activeSymbol,
            syncId: payload.sync_id,
            lastSyncTimestamp: payload.timestamp * 1000,
            isFullSyncComplete: true,
            // Clear all loading states after full sync
            loading: {
                portfolio: false,
                orders: false,
                leaderboard: false,
                indices: false,
                orderbook: false,
                candles: false,
                news: false,
                chat: false,
                tradeHistory: false,
            },
        });

        // Update auth store with portfolio data
        if (payload.portfolio) {
            useAuthStore.getState().setUser({
                money: payload.portfolio.money / PRICE_SCALE,
                lockedMoney: payload.portfolio.locked_money / PRICE_SCALE,
                marginLocked: payload.portfolio.margin_locked / PRICE_SCALE,
                netWorth: payload.portfolio.net_worth / PRICE_SCALE,
            });
        }

        // Subscribe to active symbol
        if (activeSymbol) {
            get().subscribeSymbol(activeSymbol);
        }
    },

    _handlePortfolioUpdateUI: (payload) => {
        log.debug('Portfolio UI update:', {
            money: payload.money / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
            positions: payload.items.length
        });

        const portfolioItems: PortfolioItem[] = payload.items.map(item => ({
            userId: 0,
            symbol: item.symbol,
            qty: item.qty,
            shortQty: item.short_qty,
            lockedQty: item.locked_qty,
            averageBuyPrice: item.average_buy_price / PRICE_SCALE,
            currentPrice: item.current_price / PRICE_SCALE,
            marketValue: item.market_value / PRICE_SCALE,
            costBasis: item.cost_basis / PRICE_SCALE,
            unrealizedPnl: item.unrealized_pnl / PRICE_SCALE,
            unrealizedPnlPercent: item.unrealized_pnl_percent,
            shortMarketValue: item.short_market_value / PRICE_SCALE,
            shortUnrealizedPnl: item.short_unrealized_pnl / PRICE_SCALE,
        }));

        set({
            money: payload.money / PRICE_SCALE,
            lockedMoney: payload.locked_money / PRICE_SCALE,
            marginLocked: payload.margin_locked / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
            portfolio: portfolioItems,
        });

        useAuthStore.getState().setUser({
            money: payload.money / PRICE_SCALE,
            lockedMoney: payload.locked_money / PRICE_SCALE,
            marginLocked: payload.margin_locked / PRICE_SCALE,
            netWorth: payload.net_worth / PRICE_SCALE,
        });
    },

    _handleOpenOrdersUpdate: (payload) => {
        log.debug('Open orders update:', payload.orders.length, 'orders');
        const openOrders: Order[] = payload.orders.map(o => ({
            id: o.order_id,
            userId: 0,
            symbol: o.symbol,
            orderType: o.order_type,
            side: o.side,
            qty: o.qty,
            filledQty: o.filled_qty,
            price: o.price / PRICE_SCALE,
            status: o.status as 'Open' | 'Partial',
            timestamp: o.timestamp * 1000,
            timeInForce: o.time_in_force,
        }));
        set({ openOrders });
    },

    _handleIndexUpdateUI: (payload) => {
        const idx = payload.index;
        set((state) => ({
            indices: {
                ...state.indices,
                [idx.name]: {
                    name: idx.name,
                    value: idx.value / PRICE_SCALE,
                    timestamp: idx.timestamp * 1000,
                    change: idx.change / PRICE_SCALE,
                    changePercent: idx.change_percent,
                }
            }
        }));
    },

    _handleLeaderboardUpdateUI: (payload) => {
        log.debug('Leaderboard UI update:', payload.entries.length, 'entries');
        set({
            leaderboard: payload.entries.map(e => ({
                rank: e.rank,
                name: e.name,
                netWorth: e.net_worth / PRICE_SCALE,
                userId: e.user_id,
                changeRank: e.change_rank,
            }))
        });
    },

    _handleTradeHistory: (payload) => {
        log.debug('Trade history received:', {
            trades: payload.trades.length,
            total: payload.total_count,
            page: payload.page,
            hasMore: payload.has_more
        });
        set({
            tradeHistory: payload.trades,
            tradeHistoryTotal: payload.total_count,
            tradeHistoryPage: payload.page,
            tradeHistoryHasMore: payload.has_more,
        });
    },

    _handleStockTradeHistory: (payload) => {
        log.debug('Stock trade history received:', payload.symbol, payload.trades.length, 'trades');
        set((state) => ({
            stockTradeHistory: {
                ...state.stockTradeHistory,
                [payload.symbol]: payload.trades,
            }
        }));
    },
}));

// === WebSocket Event Bindings ===

websocketService.on('connected', () => {
    useGameStore.getState().setConnected(true);
    // Subscribe to default symbol
    const symbol = useGameStore.getState().activeSymbol;
    useGameStore.getState().subscribeSymbol(symbol);
});

websocketService.on('disconnected', () => {
    useGameStore.getState().setConnected(false);
});

websocketService.on('CompanyList', (payload: { companies: Company[] }) => {
    useGameStore.getState()._setCompanies(payload.companies);
});

websocketService.on('PortfolioUpdate', (payload) => {
    useGameStore.getState()._updatePortfolio(payload as ServerPortfolioUpdate['payload']);
});

websocketService.on('TradeUpdate', (payload) => {
    useGameStore.getState()._addTrade(payload as ServerTradeUpdate['payload']);
});

websocketService.on('CandleUpdate', (payload) => {
    useGameStore.getState()._updateCandle(payload as ServerCandleUpdate['payload']);
});

websocketService.on('DepthUpdate', (payload) => {
    useGameStore.getState()._updateDepth(payload as ServerDepthUpdate['payload']);
});

websocketService.on('IndexUpdate', (payload) => {
    useGameStore.getState()._updateIndex(payload as ServerIndexUpdate['payload']);
});

websocketService.on('NewsUpdate', (payload) => {
    useGameStore.getState()._addNews(payload as ServerNewsUpdate['payload']);
});

websocketService.on('LeaderboardUpdate', (payload) => {
    useGameStore.getState()._updateLeaderboard(payload as ServerLeaderboardUpdate['payload']);
});

websocketService.on('ChatUpdate', (payload) => {
    useGameStore.getState()._addChatMessage(payload as ServerChatUpdate['payload']);
});

websocketService.on('CircuitBreaker', (payload) => {
    useGameStore.getState()._setCircuitBreaker(payload as ServerCircuitBreaker['payload']);
});

websocketService.on('MarketStatus', (payload) => {
    useGameStore.getState()._setMarketStatus(payload as ServerMarketStatus['payload']);
});

websocketService.on('OrderAck', (payload) => {
    useGameStore.getState()._handleOrderAck(payload as ServerOrderAck['payload']);
});

websocketService.on('OrderCancelled', (payload) => {
    useGameStore.getState()._handleOrderCancelled(payload as ServerOrderCancelled['payload']);
});

websocketService.on('OrderRejected', (payload) => {
    useGameStore.getState()._handleOrderRejected(payload as ServerOrderRejected['payload']);
});

websocketService.on('Error', (payload) => {
    useGameStore.getState()._handleError(payload as ServerError['payload']);
});

// === New UI-Ready Message Bindings ===

websocketService.on('FullStateSync', (payload: { payload: FullStateSyncPayload }) => {
    // The message has a nested payload structure
    useGameStore.getState()._handleFullStateSync(payload.payload);
});

websocketService.on('PortfolioUpdateUI', (payload) => {
    useGameStore.getState()._handlePortfolioUpdateUI(payload as ServerPortfolioUpdateUI['payload']);
});

websocketService.on('OpenOrdersUpdate', (payload) => {
    useGameStore.getState()._handleOpenOrdersUpdate(payload as ServerOpenOrdersUpdate['payload']);
});

websocketService.on('IndexUpdateUI', (payload) => {
    useGameStore.getState()._handleIndexUpdateUI(payload as ServerIndexUpdateUI['payload']);
});

websocketService.on('LeaderboardUpdateUI', (payload) => {
    useGameStore.getState()._handleLeaderboardUpdateUI(payload as ServerLeaderboardUpdateUI['payload']);
});

websocketService.on('TradeHistory', (payload) => {
    useGameStore.getState()._handleTradeHistory(payload as ServerTradeHistory['payload']);
});

websocketService.on('StockTradeHistory', (payload) => {
    useGameStore.getState()._handleStockTradeHistory(payload as ServerStockTradeHistory['payload']);
});

// === Component Sync Message Bindings ===

websocketService.on('PortfolioSync', (payload: { sync_id: number; money: number; locked_money: number; margin_locked: number; portfolio_value: number; net_worth: number; items: PortfolioItemUI[] }) => {
    useGameStore.getState()._handlePortfolioUpdateUI({
        money: payload.money,
        locked_money: payload.locked_money,
        margin_locked: payload.margin_locked,
        portfolio_value: payload.portfolio_value,
        net_worth: payload.net_worth,
        items: payload.items,
    });
});

websocketService.on('OpenOrdersSync', (payload: { sync_id: number; orders: OpenOrderUI[] }) => {
    useGameStore.getState()._handleOpenOrdersUpdate({ orders: payload.orders });
});

websocketService.on('LeaderboardSync', (payload: { sync_id: number; entries: LeaderboardEntryUI[] }) => {
    useGameStore.getState()._handleLeaderboardUpdateUI({ entries: payload.entries });
});

websocketService.on('IndicesSync', (payload: { sync_id: number; indices: MarketIndexUI[] }) => {
    for (const index of payload.indices) {
        useGameStore.getState()._handleIndexUpdateUI({ index });
    }
});

interface OrderbookSyncPayload {
    symbol: string;
    bids: Array<{ price: number; qty: number }>;
    asks: Array<{ price: number; qty: number }>;
    spread: number | null;
}

websocketService.on('OrderbookSync', (payload: { sync_id: number; symbol: string; orderbook: OrderbookSyncPayload }) => {
    const state = useGameStore.getState();
    const ob = payload.orderbook;
    useGameStore.setState({
        orderBooks: {
            ...state.orderBooks,
            [payload.symbol]: {
                symbol: payload.symbol,
                bids: ob.bids.map((b) => ({ price: b.price / PRICE_SCALE, quantity: b.qty })),
                asks: ob.asks.map((a) => ({ price: a.price / PRICE_SCALE, quantity: a.qty })),
                spread: ob.spread ? ob.spread / PRICE_SCALE : null,
            },
        },
    });
});

websocketService.on('CandlesSync', (payload: { sync_id: number; symbol: string; candles: Array<{ timestamp: number; open: number; high: number; low: number; close: number; volume: number }> }) => {
    const state = useGameStore.getState();
    useGameStore.setState({
        candles: {
            ...state.candles,
            [payload.symbol]: payload.candles.map((c) => ({
                symbol: payload.symbol,
                resolution: '1m',
                open: c.open / PRICE_SCALE,
                high: c.high / PRICE_SCALE,
                low: c.low / PRICE_SCALE,
                close: c.close / PRICE_SCALE,
                volume: c.volume,
                timestamp: c.timestamp * 1000,
            })),
        },
    });
});

websocketService.on('NewsSync', (payload: { sync_id: number; news: Array<{ id: string; headline: string; sentiment: string; symbol: string | null; timestamp: number }> }) => {
    useGameStore.setState({
        news: payload.news.map((n) => ({
            id: n.id,
            headline: n.headline,
            sentiment: n.sentiment as 'Bullish' | 'Bearish' | 'Neutral',
            symbol: n.symbol || undefined,
            timestamp: n.timestamp * 1000,
        })),
    });
});

websocketService.on('ChatSync', (payload: { sync_id: number; messages: Array<{ id: string; user_id: number; username: string; message: string; timestamp: number }> }) => {
    useGameStore.setState({
        chatMessages: payload.messages.map((m) => ({
            id: m.id,
            userId: m.user_id,
            username: m.username,
            message: m.message,
            timestamp: m.timestamp * 1000,
        })),
    });
});

export default useGameStore;
