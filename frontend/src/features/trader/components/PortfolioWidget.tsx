// ============================================
// Portfolio Widget Component
// Shows holdings, P&L, and trade history
// ============================================

import React, { useState, useEffect, useCallback } from 'react';
import { Briefcase, DollarSign, TrendingUp, Lock } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';
import { Badge } from '../../../components/common';
import { PRICE_SCALE } from '../../../types/models';
import { UI } from '../../../constants';

interface PortfolioWidgetProps {
    onQuickSell?: (symbol: string, qty: number, price: number) => void;
}

const formatPercent = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

export const PortfolioWidget: React.FC<PortfolioWidgetProps> = ({ onQuickSell }) => {
    const { money, lockedMoney, marginLocked, portfolio, orderBooks, setActiveSymbol, tradeHistory, requestTradeHistory } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [activeTab, setActiveTab] = useState<'positions' | 'history'>('positions');

    // Get P&L for each position - use server-provided values when available
    const getPositionPnL = useCallback((item: typeof portfolio[0]) => {
        // Use pre-computed values from server if available
        if (item.unrealizedPnl !== undefined && item.currentPrice !== undefined) {
            return {
                pnl: item.unrealizedPnl,
                pnlPercent: item.unrealizedPnlPercent || 0,
                currentPrice: item.currentPrice,
            };
        }
        // Fallback to local calculation
        const orderBook = orderBooks[item.symbol];
        const currentPrice = orderBook?.bids[0]?.price || orderBook?.asks[0]?.price || item.averageBuyPrice;
        const pnl = (currentPrice - item.averageBuyPrice) * item.qty;
        const pnlPercent = item.averageBuyPrice > 0 ? ((currentPrice - item.averageBuyPrice) / item.averageBuyPrice) * 100 : 0;
        return { pnl, pnlPercent, currentPrice };
    }, [orderBooks]);

    const handleQuickSell = (item: typeof portfolio[0]) => {
        const { currentPrice } = getPositionPnL(item);
        const sellableQty = item.qty - item.lockedQty;
        if (sellableQty > 0 && onQuickSell) {
            setActiveSymbol(item.symbol);
            onQuickSell(item.symbol, sellableQty, currentPrice);
        }
    };

    // Calculate total portfolio value and P&L
    const totalValue = portfolio.reduce((sum, item) => {
        // Use pre-computed marketValue if available
        if (item.marketValue !== undefined) {
            return sum + item.marketValue;
        }
        const { currentPrice } = getPositionPnL(item);
        return sum + (currentPrice * item.qty);
    }, 0);

    const totalPnL = portfolio.reduce((sum, item) => {
        const { pnl } = getPositionPnL(item);
        return sum + pnl;
    }, 0);

    // Load trade history when switching to history tab
    useEffect(() => {
        if (activeTab === 'history') {
            requestTradeHistory();
        }
    }, [activeTab, requestTradeHistory]);

    return (
        <div className="portfolio-widget">
            <div className="widget-header">
                <span className="widget-title"><Briefcase size={14} /> Portfolio</span>
                {totalPnL !== 0 && (
                    <Badge variant={totalPnL >= 0 ? 'success' : 'danger'}>
                        {totalPnL >= 0 ? '+' : ''}{formatCurrency(totalPnL)}
                    </Badge>
                )}
            </div>
            <div className="portfolio-stats">
                <div className="stat">
                    <span className="stat-label"><DollarSign size={12} /> Cash</span>
                    <span className="stat-value positive">{formatCurrency(money)}</span>
                </div>
                <div className="stat">
                    <span className="stat-label"><Briefcase size={12} /> Holdings</span>
                    <span className="stat-value">{formatCurrency(totalValue)}</span>
                </div>
                <div className="stat">
                    <span className="stat-label"><TrendingUp size={12} /> Net Worth</span>
                    <span className="stat-value">{formatCurrency(money + lockedMoney + marginLocked + totalValue)}</span>
                </div>
            </div>

            {/* Tab Switcher */}
            <div className="portfolio-tabs">
                <button
                    className={`portfolio-tab ${activeTab === 'positions' ? 'active' : ''}`}
                    onClick={() => setActiveTab('positions')}
                >
                    Positions
                </button>
                <button
                    className={`portfolio-tab ${activeTab === 'history' ? 'active' : ''}`}
                    onClick={() => setActiveTab('history')}
                >
                    Trade History
                </button>
            </div>

            {activeTab === 'positions' && (
                <div className="holdings">
                    <div className="holdings-header">
                        <span>Positions</span>
                        {lockedMoney + marginLocked > 0 && (
                            <span className="locked-indicator">
                                <Lock size={10} /> {formatCurrency(lockedMoney + marginLocked)} locked
                            </span>
                        )}
                    </div>
                    {portfolio.length === 0 ? (
                        <div className="empty-state">No positions yet</div>
                    ) : (
                        <div className="holdings-list">
                            {portfolio.map((item) => {
                                const { pnl, pnlPercent, currentPrice } = getPositionPnL(item);
                                const sellableQty = item.qty - item.lockedQty;
                                return (
                                    <div key={item.symbol} className="holding-row">
                                        <div className="holding-info">
                                            <span className="symbol">{item.symbol}</span>
                                            <span className="position-details">
                                                {item.qty} @ {formatCurrency(item.averageBuyPrice)}
                                            </span>
                                        </div>
                                        <div className="holding-value">
                                            <span className="current-value">{formatCurrency(currentPrice * item.qty)}</span>
                                            <span className={`pnl ${pnl >= 0 ? 'positive' : 'negative'}`}>
                                                {pnl >= 0 ? '+' : ''}{formatCurrency(pnl)} ({formatPercent(pnlPercent)})
                                            </span>
                                        </div>
                                        {sellableQty > 0 && (
                                            <button
                                                className="quick-sell-btn"
                                                onClick={() => handleQuickSell(item)}
                                                title={`Sell ${sellableQty} shares`}
                                            >
                                                Sell
                                            </button>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            )}

            {activeTab === 'history' && (
                <div className="trade-history">
                    {tradeHistory.length === 0 ? (
                        <div className="empty-state">No trade history yet</div>
                    ) : (
                        <div className="trade-history-list">
                            {tradeHistory.slice(0, UI.TRADE_HISTORY_WIDGET).map((trade) => (
                                <div key={trade.trade_id} className="trade-history-row">
                                    <div className="trade-info">
                                        <Badge variant={trade.side === 'Buy' ? 'buy' : 'sell'}>
                                            {trade.side}
                                        </Badge>
                                        <span className="symbol">{trade.symbol}</span>
                                    </div>
                                    <div className="trade-details">
                                        <span className="qty-price">{trade.qty} @ {formatCurrency(trade.price / PRICE_SCALE)}</span>
                                        <span className="total">{formatCurrency(trade.total_value / PRICE_SCALE)}</span>
                                    </div>
                                    <div className="trade-time">
                                        {new Date(trade.timestamp * 1000).toLocaleTimeString()}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export default PortfolioWidget;
