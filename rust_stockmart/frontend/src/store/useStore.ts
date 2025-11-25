import { create } from 'zustand';
import { socketService } from '../services/socket';

interface User {
    id: number;
    name: string;
    money: number;
    portfolio: any[];
}

interface Trade {
    symbol: string;
    price: number;
    qty: number;
    timestamp: number;
}

interface Candle {
    time: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
}

interface AppState {
    user: User | null;
    isConnected: boolean;
    trades: Trade[];
    candles: Record<string, Candle[]>;
    indices: Record<string, number>;
    activeSymbol: string;
    setUser: (user: User) => void;
    addTrade: (trade: Trade) => void;
    setConnected: (connected: boolean) => void;
    setActiveSymbol: (symbol: string) => void;
    updateCandle: (symbol: string, candle: Candle) => void;
    updateIndex: (name: string, value: number) => void;
    updatePortfolio: (money: number, locked: number, items: any[]) => void;
    halted: Record<string, number>; // Symbol -> Halted Until Timestamp
    updateHalted: (symbol: string, haltedUntil: number) => void;
    news: any[];
    addNews: (newsItem: any) => void;
    leaderboard: any[];
    updateLeaderboard: (entries: any[]) => void;
    chatMessages: any[];
    addChatMessage: (message: any) => void;
}

export const useStore = create<AppState>((set) => ({
    user: null,
    isConnected: false,
    trades: [],
    candles: {},
    indices: {},
    activeSymbol: 'AAPL',
    halted: {},
    news: [],
    leaderboard: [],
    chatMessages: [],
    setUser: (user) => set({ user }),
    addTrade: (trade) => set((state) => ({ trades: [trade, ...state.trades].slice(0, 50) })),
    setConnected: (isConnected) => set({ isConnected }),
    setActiveSymbol: (activeSymbol) => set({ activeSymbol }),
    addNews: (newsItem) => set((state) => ({ news: [newsItem, ...state.news].slice(0, 20) })),
    updateLeaderboard: (entries) => set({ leaderboard: entries }),
    addChatMessage: (message) => set((state) => ({ chatMessages: [...state.chatMessages, message].slice(-50) })),
    updateCandle: (symbol, candle) => set((state) => {
        const currentCandles = state.candles[symbol] || [];
        // Check if we should update the last candle or add a new one
        const lastCandle = currentCandles[currentCandles.length - 1];
        if (lastCandle && lastCandle.time === candle.time) {
            // Update last
            const newCandles = [...currentCandles];
            newCandles[newCandles.length - 1] = candle;
            return { candles: { ...state.candles, [symbol]: newCandles } };
        } else {
            // Append
            return { candles: { ...state.candles, [symbol]: [...currentCandles, candle] } };
        }
    }),
    updateIndex: (name, value) => set((state) => ({
        indices: { ...state.indices, [name]: value }
    })),
    updatePortfolio: (money, locked, items) => set((state) => {
        if (!state.user) return {};
        return {
            user: {
                ...state.user,
                money: money / 10000,
                portfolio: items
            }
        };
    }),
    updateHalted: (symbol, haltedUntil) => set((state) => ({
        halted: { ...state.halted, [symbol]: haltedUntil }
    })),
}));

// Bind socket events to store
socketService.on('connected', () => {
    useStore.getState().setConnected(true);
    // Subscribe to initial symbol
    socketService.send('Subscribe', { symbol: useStore.getState().activeSymbol });
});
socketService.on('disconnected', () => useStore.getState().setConnected(false));
socketService.on('AuthSuccess', (payload: any) => {
    useStore.getState().setUser({
        id: payload.user_id,
        name: payload.name,
        money: 0, // Mock initial money until we get full profile
        portfolio: []
    });
});
socketService.on('PortfolioUpdate', (payload: any) => {
    useStore.getState().updatePortfolio(payload.money, payload.locked, payload.items);
});
socketService.on('CircuitBreaker', (payload: any) => {
    useStore.getState().updateHalted(payload.symbol, payload.halted_until);
});
socketService.on('NewsUpdate', (payload: any) => {
    useStore.getState().addNews(payload.news);
});
socketService.on('LeaderboardUpdate', (payload: any) => {
    useStore.getState().updateLeaderboard(payload.entries);
});
socketService.on('ChatUpdate', (payload: any) => {
    useStore.getState().addChatMessage(payload.message);
});
socketService.on('TradeUpdate', (payload: any) => {
    useStore.getState().addTrade({
        symbol: payload.symbol,
        price: payload.price / 10000, // Scale down
        qty: payload.qty,
        timestamp: Date.now()
    });
});
socketService.on('CandleUpdate', (payload: any) => {
    useStore.getState().updateCandle(payload.symbol, {
        time: payload.candle.timestamp,
        open: payload.candle.open,
        high: payload.candle.high,
        low: payload.candle.low,
        close: payload.candle.close,
        volume: payload.candle.volume
    });
});
socketService.on('IndexUpdate', (payload: any) => {
    useStore.getState().updateIndex(payload.name, payload.value / 10000);
});
