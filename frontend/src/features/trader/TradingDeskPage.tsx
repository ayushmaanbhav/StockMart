// ============================================
// Trading Desk Page
// Main trading interface using extracted components
// ============================================

import React, { useState, useCallback } from 'react';
import { TrendingUp } from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { CandlestickChart } from '../../components/charts/CandlestickChart';
import {
    SymbolSelector,
    MarketIndicesBar,
    OrderBookWidget,
    PortfolioWidget,
    QuickTradeWidget,
    OpenOrdersWidget,
    NewsTicker,
    LeaderboardWidget,
    ChatWidget,
} from './components';

export const TradingDeskPage: React.FC = () => {
    const { activeSymbol } = useGameStore();

    // Collapsible widget states
    const [leaderboardCollapsed, setLeaderboardCollapsed] = useState(false);
    const [chatCollapsed, setChatCollapsed] = useState(false);
    const [quickTradeCollapsed, setQuickTradeCollapsed] = useState(false);
    const [openOrdersCollapsed, setOpenOrdersCollapsed] = useState(false);

    // Shared trade form state - for order book click and portfolio quick sell
    const [tradeFormState, setTradeFormState] = useState<{
        price?: number;
        side?: 'Buy' | 'Sell' | 'Short';
        qty?: number;
        updateKey: number;
    }>({ updateKey: 0 });

    // Handle order book price click
    const handleOrderBookPriceClick = useCallback((price: number, side: 'Buy' | 'Sell') => {
        setTradeFormState(prev => ({
            price,
            side,
            qty: undefined,
            updateKey: prev.updateKey + 1
        }));
    }, []);

    // Handle portfolio quick sell
    const handleQuickSell = useCallback((_symbol: string, qty: number, price: number) => {
        setTradeFormState(prev => ({
            price,
            side: 'Sell',
            qty,
            updateKey: prev.updateKey + 1
        }));
    }, []);

    // Clear external state after it's been consumed
    const clearExternalState = useCallback(() => {
        // Small delay to ensure state is consumed
        setTimeout(() => {
            setTradeFormState(prev => ({
                price: undefined,
                side: undefined,
                qty: undefined,
                updateKey: prev.updateKey
            }));
        }, 100);
    }, []);

    return (
        <div className="trading-desk-new">
            {/* Top Bar - Market Indices */}
            <MarketIndicesBar />

            {/* Price Chart - spans full width of left column */}
            <div className="chart-panel">
                <div className="chart-header">
                    <div className="chart-title">
                        <TrendingUp size={18} />
                        <span style={{ marginLeft: '8px', fontWeight: 600 }}>{activeSymbol}</span>
                    </div>
                </div>
                <div className="chart-body">
                    <CandlestickChart symbol={activeSymbol} height={180} />
                </div>
            </div>

            {/* Right Sidebar - Company selector, Leaderboard, Chat, Order Placement */}
            <div className="trading-right">
                {/* Company Selector at top of right sidebar - STICKY */}
                <div className="symbol-selector-sticky">
                    <SymbolSelector />
                </div>

                {/* Scrollable widgets container */}
                <div className="trading-right-scroll">
                    {/* Leaderboard Widget */}
                    <LeaderboardWidget
                        isCollapsed={leaderboardCollapsed}
                        onToggle={() => setLeaderboardCollapsed(!leaderboardCollapsed)}
                    />

                    {/* Chat Widget */}
                    <ChatWidget
                        isCollapsed={chatCollapsed}
                        onToggle={() => setChatCollapsed(!chatCollapsed)}
                    />

                    {/* Order Placement */}
                    <QuickTradeWidget
                        key={tradeFormState.updateKey}
                        externalPrice={tradeFormState.price}
                        externalSide={tradeFormState.side}
                        externalQty={tradeFormState.qty}
                        onExternalUpdate={clearExternalState}
                        isCollapsed={quickTradeCollapsed}
                        onToggle={() => setQuickTradeCollapsed(!quickTradeCollapsed)}
                    />

                    {/* Open Orders */}
                    <OpenOrdersWidget
                        isCollapsed={openOrdersCollapsed}
                        onToggle={() => setOpenOrdersCollapsed(!openOrdersCollapsed)}
                    />
                </div>
            </div>

            {/* Bottom Widgets Row - Order Book and Portfolio */}
            <div className="bottom-widgets">
                <OrderBookWidget onPriceClick={handleOrderBookPriceClick} />
                <PortfolioWidget onQuickSell={handleQuickSell} />
            </div>

            {/* News Ticker */}
            <NewsTicker />
        </div>
    );
};

export default TradingDeskPage;
