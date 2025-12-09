// ============================================
// Admin Store
// Manages admin-specific state (metrics, trades, orders)
// ============================================

import { create } from 'zustand';
import type {
    AdminTradeHistoryItem,
    AdminOpenOrderUI,
    AdminDashboardMetrics,
} from '../types/api';
import { PRICE_SCALE } from '../types/models';
import websocketService from '../services/websocket';

interface AdminState {
    // Dashboard metrics
    metrics: AdminDashboardMetrics | null;
    metricsLoading: boolean;

    // Trade history
    trades: AdminTradeHistoryItem[];
    tradesTotal: number;
    tradesPage: number;
    tradesPageSize: number;
    tradesHasMore: boolean;
    tradesLoading: boolean;
    tradesFilter: {
        userId?: number;
        symbol?: string;
    };

    // Open orders
    orders: AdminOpenOrderUI[];
    ordersTotal: number;
    ordersLoading: boolean;
    ordersFilter: {
        symbol?: string;
    };

    // Orderbook
    orderbookSymbol: string | null;
    orderbookBids: AdminOpenOrderUI[];
    orderbookAsks: AdminOpenOrderUI[];
    orderbookLoading: boolean;

    // Actions
    fetchMetrics: () => void;
    fetchTrades: (page?: number, userId?: number, symbol?: string) => void;
    fetchOrders: (symbol?: string) => void;
    fetchOrderbook: (symbol: string) => void;
    setTradesFilter: (filter: { userId?: number; symbol?: string }) => void;
    setOrdersFilter: (filter: { symbol?: string }) => void;
}

export const useAdminStore = create<AdminState>((set, get) => ({
    // Initial state
    metrics: null,
    metricsLoading: false,

    trades: [],
    tradesTotal: 0,
    tradesPage: 0,
    tradesPageSize: 20,
    tradesHasMore: false,
    tradesLoading: false,
    tradesFilter: {},

    orders: [],
    ordersTotal: 0,
    ordersLoading: false,
    ordersFilter: {},

    orderbookSymbol: null,
    orderbookBids: [],
    orderbookAsks: [],
    orderbookLoading: false,

    // Actions
    fetchMetrics: () => {
        set({ metricsLoading: true });
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'GetDashboardMetrics',
                payload: {},
            },
        });
    },

    fetchTrades: (page = 0, userId, symbol) => {
        set({
            tradesLoading: true,
            tradesFilter: { userId, symbol },
        });
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'GetAllTrades',
                payload: {
                    page,
                    page_size: get().tradesPageSize,
                    ...(userId !== undefined && { user_id: userId }),
                    ...(symbol && { symbol }),
                },
            },
        });
    },

    fetchOrders: (symbol) => {
        set({
            ordersLoading: true,
            ordersFilter: { symbol },
        });
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'GetAllOpenOrders',
                payload: symbol ? { symbol } : {},
            },
        });
    },

    fetchOrderbook: (symbol) => {
        set({ orderbookLoading: true, orderbookSymbol: symbol });
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'GetOrderbook',
                payload: { symbol },
            },
        });
    },

    setTradesFilter: (filter) => {
        set({ tradesFilter: filter });
    },

    setOrdersFilter: (filter) => {
        set({ ordersFilter: filter });
    },
}));

// === WebSocket Event Bindings ===

websocketService.on('AdminDashboardMetrics', (payload: { metrics: AdminDashboardMetrics }) => {
    // Scale prices
    const metrics: AdminDashboardMetrics = {
        ...payload.metrics,
        total_volume: payload.metrics.total_volume / PRICE_SCALE,
        recent_volume: payload.metrics.recent_volume / PRICE_SCALE,
        total_market_cap: payload.metrics.total_market_cap / PRICE_SCALE,
    };
    useAdminStore.setState({ metrics, metricsLoading: false });
});

websocketService.on('AdminTradeHistory', (payload: {
    trades: AdminTradeHistoryItem[];
    total_count: number;
    page: number;
    page_size: number;
    has_more: boolean;
}) => {
    // Scale prices in trades
    const trades = payload.trades.map((t) => ({
        ...t,
        price: t.price / PRICE_SCALE,
        total_value: t.total_value / PRICE_SCALE,
    }));
    useAdminStore.setState({
        trades,
        tradesTotal: payload.total_count,
        tradesPage: payload.page,
        tradesPageSize: payload.page_size,
        tradesHasMore: payload.has_more,
        tradesLoading: false,
    });
});

websocketService.on('AdminOpenOrders', (payload: {
    orders: AdminOpenOrderUI[];
    total_count: number;
}) => {
    // Scale prices in orders
    const orders = payload.orders.map((o) => ({
        ...o,
        price: o.price / PRICE_SCALE,
    }));
    useAdminStore.setState({
        orders,
        ordersTotal: payload.total_count,
        ordersLoading: false,
    });
});

websocketService.on('AdminOrderbook', (payload: {
    symbol: string;
    bids: AdminOpenOrderUI[];
    asks: AdminOpenOrderUI[];
}) => {
    // Scale prices
    const bids = payload.bids.map((o) => ({
        ...o,
        price: o.price / PRICE_SCALE,
    }));
    const asks = payload.asks.map((o) => ({
        ...o,
        price: o.price / PRICE_SCALE,
    }));
    useAdminStore.setState({
        orderbookSymbol: payload.symbol,
        orderbookBids: bids,
        orderbookAsks: asks,
        orderbookLoading: false,
    });
});

export default useAdminStore;
