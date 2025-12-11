// ============================================
// Order Book Widget Component
// Shows bid/ask depth and recent trades
// ============================================

import React, { useState, useEffect, useMemo } from 'react';
import { BookOpen } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';
import { Badge } from '../../../components/common';
import { PRICE_SCALE } from '../../../types/models';
import { UI } from '../../../constants';

interface OrderBookWidgetProps {
    onPriceClick?: (price: number, side: 'Buy' | 'Sell') => void;
}

export const OrderBookWidget: React.FC<OrderBookWidgetProps> = ({ onPriceClick }) => {
    const { activeSymbol, orderBooks, stockTradeHistory, requestStockTrades } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const orderBook = orderBooks[activeSymbol];
    const [activeTab, setActiveTab] = useState<'depth' | 'trades'>('depth');

    // Calculate max quantity for depth visualization
    const maxQty = useMemo(() => Math.max(
        ...(orderBook?.bids.map(b => b.quantity) || [1]),
        ...(orderBook?.asks.map(a => a.quantity) || [1])
    ), [orderBook]);

    const handlePriceClick = (price: number, side: 'Buy' | 'Sell') => {
        if (onPriceClick) {
            onPriceClick(price, side);
        }
    };

    // Load stock trades when switching to trades tab or symbol changes
    useEffect(() => {
        if (activeTab === 'trades' && activeSymbol) {
            requestStockTrades(activeSymbol);
        }
    }, [activeTab, activeSymbol, requestStockTrades]);

    const stockTrades = stockTradeHistory[activeSymbol] || [];

    return (
        <div className="orderbook-widget">
            <div className="widget-header">
                <span className="widget-title"><BookOpen size={14} /> Order Book</span>
                {orderBook?.spread !== undefined && orderBook?.spread !== null && orderBook.spread > 0 && (
                    <Badge variant="primary">Spread: {formatCurrency(orderBook.spread)}</Badge>
                )}
            </div>

            {/* Tab Switcher */}
            <div className="orderbook-tabs">
                <button
                    className={`orderbook-tab ${activeTab === 'depth' ? 'active' : ''}`}
                    onClick={() => setActiveTab('depth')}
                >
                    Depth
                </button>
                <button
                    className={`orderbook-tab ${activeTab === 'trades' ? 'active' : ''}`}
                    onClick={() => setActiveTab('trades')}
                >
                    Trades
                </button>
            </div>

            {activeTab === 'depth' && (
                <div className="orderbook-content">
                    <div className="orderbook-side bids">
                        <div className="orderbook-side-header">Bids</div>
                        {!orderBook?.bids.length && <div className="empty-state">No bids</div>}
                        {orderBook?.bids.slice(0, UI.ORDERBOOK_DEPTH).map((level, i) => (
                            <div
                                key={i}
                                className="orderbook-row clickable"
                                onClick={() => handlePriceClick(level.price, 'Sell')}
                                title="Click to sell at this price"
                            >
                                <div
                                    className="depth-bar bid"
                                    style={{ width: `${(level.quantity / maxQty) * 100}%` }}
                                />
                                <span className="price bid">{formatCurrency(level.price)}</span>
                                <span className="qty">{level.quantity}</span>
                            </div>
                        ))}
                    </div>
                    <div className="orderbook-side asks">
                        <div className="orderbook-side-header">Asks</div>
                        {!orderBook?.asks.length && <div className="empty-state">No asks</div>}
                        {orderBook?.asks.slice(0, UI.ORDERBOOK_DEPTH).map((level, i) => (
                            <div
                                key={i}
                                className="orderbook-row clickable"
                                onClick={() => handlePriceClick(level.price, 'Buy')}
                                title="Click to buy at this price"
                            >
                                <div
                                    className="depth-bar ask"
                                    style={{ width: `${(level.quantity / maxQty) * 100}%` }}
                                />
                                <span className="price ask">{formatCurrency(level.price)}</span>
                                <span className="qty">{level.quantity}</span>
                            </div>
                        ))}
                    </div>
                </div>
            )}

            {activeTab === 'trades' && (
                <div className="stock-trades-content">
                    {stockTrades.length === 0 ? (
                        <div className="empty-state">No trades yet for {activeSymbol}</div>
                    ) : (
                        <div className="stock-trades-list">
                            <div className="stock-trades-header">
                                <span>Time</span>
                                <span>Price</span>
                                <span>Qty</span>
                            </div>
                            {stockTrades.slice(0, UI.STOCK_TRADES_WIDGET).map((trade) => (
                                <div key={trade.trade_id} className="stock-trade-row">
                                    <span className="trade-time">
                                        {new Date(trade.timestamp * 1000).toLocaleTimeString()}
                                    </span>
                                    <span className={`trade-price ${trade.side === 'Buy' ? 'positive' : 'negative'}`}>
                                        {formatCurrency(trade.price / PRICE_SCALE)}
                                    </span>
                                    <span className="trade-qty">{trade.qty}</span>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export default OrderBookWidget;
