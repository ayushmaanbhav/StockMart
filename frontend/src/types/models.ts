// ============================================
// Domain Models - TypeScript Types
// ============================================

// === Scalars ===
export type UserId = number;
export type CompanyId = number;
export type OrderId = number;
export type TradeId = number;
export type Price = number; // Already scaled (displayed value)
export type RawPrice = number; // Scaled by 10,000 (from backend)
export type Quantity = number;

export const PRICE_SCALE = 10000;

// === Utility Functions ===
export const scalePrice = (raw: RawPrice): Price => raw / PRICE_SCALE;
export const unscalePrice = (price: Price): RawPrice => Math.round(price * PRICE_SCALE);

// === Enums ===
export type OrderType = 'Market' | 'Limit';
export type OrderSide = 'Buy' | 'Sell' | 'Short';
export type OrderStatus = 'Open' | 'Partial' | 'Filled' | 'Cancelled' | 'Rejected';
export type TimeInForce = 'GTC' | 'IOC';
export type UserRole = 'admin' | 'trader';

// === User ===
export interface User {
    id: UserId;
    regno: string;
    name: string;
    role: UserRole;
    money: Price;
    lockedMoney: Price;
    marginLocked: Price;
    netWorth: Price;
    portfolio: PortfolioItem[];
    chatEnabled: boolean;
    banned: boolean;
    createdAt: number;
}

// === Company ===
export interface Company {
    id: CompanyId;
    symbol: string;
    name: string;
    sector: string;
    totalShares: Quantity;
    bankrupt: boolean;
    volatility: number;
}

// === Order ===
export interface Order {
    id: OrderId;
    userId: UserId;
    symbol: string;
    orderType: OrderType;
    side: OrderSide;
    qty: Quantity;
    filledQty: Quantity;
    price: Price;
    status: OrderStatus;
    timestamp: number;
    timeInForce: TimeInForce;
}

// === Trade ===
export interface Trade {
    id: TradeId;
    symbol: string;
    qty: Quantity;
    price: Price;
    timestamp: number;
}

// === Portfolio Item ===
export interface PortfolioItem {
    userId: UserId;
    symbol: string;
    qty: Quantity;
    shortQty: Quantity;
    lockedQty: Quantity;
    averageBuyPrice: Price;
    // UI-ready pre-computed fields (from backend)
    currentPrice?: Price;
    marketValue?: Price;
    costBasis?: Price;
    unrealizedPnl?: Price;
    unrealizedPnlPercent?: number;
    shortMarketValue?: Price;
    shortUnrealizedPnl?: Price;
}

// === Candle (OHLCV) ===
export interface Candle {
    symbol: string;
    resolution: string;
    open: Price;
    high: Price;
    low: Price;
    close: Price;
    volume: Quantity;
    timestamp: number;
}

// === Order Book ===
export interface OrderBookLevel {
    price: Price;
    quantity: Quantity;
}

export interface OrderBookDepth {
    symbol: string;
    bids: OrderBookLevel[];
    asks: OrderBookLevel[];
    spread: Price | null;
}

// === Market Index ===
export interface MarketIndex {
    name: string;
    value: Price;
    timestamp: number;
    change?: number;
    changePercent?: number;
}

// === News ===
export interface NewsItem {
    id: string | number;
    headline: string;
    sentiment: 'Bullish' | 'Bearish' | 'Neutral';
    symbol?: string;
    timestamp: number;
}

// === Leaderboard ===
export interface LeaderboardEntry {
    rank: number;
    userId?: UserId;
    name: string;
    netWorth: Price;
    change?: number;
    changePercent?: number;
    changeRank?: number; // Positive = moved up, negative = moved down
}

// === Chat ===
export interface ChatMessage {
    id: string;
    userId: UserId;
    username: string;
    message: string;
    timestamp: number;
}

// === Game State ===
export interface GameState {
    isInitialized: boolean;
    marketOpen: boolean;
    startTime?: number;
    endTime?: number;
    totalTraders: number;
    totalTrades: number;
    totalVolume: Price;
}

// === Circuit Breaker ===
export interface CircuitBreaker {
    symbol: string;
    haltedUntil: number;
    reason: string;
}
