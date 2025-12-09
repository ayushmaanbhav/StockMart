// ============================================
// Trading Desk Page - Redesigned
// ============================================

import React from 'react';
import {
    TrendingUp,
    BarChart2,
    BookOpen,
    Briefcase,
    MessageCircle,
    Trophy,
    Newspaper,
    DollarSign,
    Lock,
    ArrowUpRight,
    ArrowDownRight,
    ChevronDown,
    ChevronRight,
    X,
    AlertTriangle,
    CheckCircle,
    XCircle
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { useAuthStore } from '../../store/authStore';
import { useConfigStore } from '../../store/configStore';
import { Badge, Tabs, Modal, Button } from '../../components/common';
import { CandlestickChart } from '../../components/charts/CandlestickChart';

// Format helpers - uses store's formatCurrency
const formatPercent = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

// === Symbol Selector ===
const SymbolSelector: React.FC = () => {
    const { activeSymbol, setActiveSymbol, companies, orderBooks } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [isOpen, setIsOpen] = React.useState(false);
    const [search, setSearch] = React.useState('');

    // Filter companies by search
    const filteredCompanies = companies.filter(c =>
        c.symbol.toLowerCase().includes(search.toLowerCase()) ||
        c.name.toLowerCase().includes(search.toLowerCase())
    );

    if (!activeSymbol) {
        return (
            <div className="symbol-selector-btn loading">
                <span className="text-muted">Loading...</span>
            </div>
        );
    }

    const orderBook = orderBooks[activeSymbol];
    const currentPrice = orderBook?.asks[0]?.price || orderBook?.bids[0]?.price;

    return (
        <div className="symbol-selector">
            <button
                className="symbol-selector-btn"
                onClick={() => setIsOpen(!isOpen)}
            >
                <div className="symbol-main">
                    <span className="symbol-name">{activeSymbol}</span>
                    {currentPrice && (
                        <span className="symbol-price">{formatCurrency(currentPrice)}</span>
                    )}
                </div>
                <ChevronDown size={14} className={isOpen ? 'rotated' : ''} />
            </button>

            {isOpen && (
                <>
                    <div
                        className="symbol-selector-overlay"
                        onClick={() => {
                            setIsOpen(false);
                            setSearch('');
                        }}
                    />
                    <div className="symbol-selector-dropdown">
                        <div className="symbol-search">
                            <input
                                type="text"
                                placeholder="Search stocks..."
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                                autoFocus
                            />
                        </div>
                        <div className="symbol-list">
                            {filteredCompanies.length === 0 && (
                                <div className="symbol-empty">No matches found</div>
                            )}
                            {filteredCompanies.map(company => {
                                const companyOrderBook = orderBooks[company.symbol];
                                const price = companyOrderBook?.asks[0]?.price || companyOrderBook?.bids[0]?.price;
                                const isActive = company.symbol === activeSymbol;

                                return (
                                    <button
                                        key={company.symbol}
                                        className={`symbol-option ${isActive ? 'active' : ''}`}
                                        onClick={() => {
                                            setActiveSymbol(company.symbol);
                                            setIsOpen(false);
                                            setSearch('');
                                        }}
                                    >
                                        <div className="symbol-option-info">
                                            <span className="symbol-option-symbol">{company.symbol}</span>
                                            <span className="symbol-option-name">{company.name}</span>
                                        </div>
                                        {price && (
                                            <span className="symbol-option-price">{formatCurrency(price)}</span>
                                        )}
                                    </button>
                                );
                            })}
                        </div>
                    </div>
                </>
            )}
        </div>
    );
};

// === Market Indices Bar (Animated Ticker) ===
const MarketIndicesBar: React.FC = () => {
    const { indices } = useGameStore();
    const indexList = Object.values(indices);

    // Duplicate items for seamless infinite scroll
    const duplicatedItems = [...indexList, ...indexList];

    return (
        <div className="market-indices-bar">
            <div className="indices-label">
                <TrendingUp size={14} />
                <span>LIVE</span>
            </div>
            <div className="indices-scroll-wrapper">
                {indexList.length === 0 ? (
                    <span className="text-muted text-sm p-2">Waiting for market data...</span>
                ) : (
                    <div className="indices-scroll">
                        {duplicatedItems.map((index, i) => (
                            <div key={`${index.name}-${i}`} className="index-item">
                                <span className="index-name">{index.name.replace('SECTOR:', '')}</span>
                                <span className="index-value">{index.value.toFixed(2)}</span>
                                {index.changePercent !== undefined && index.changePercent !== 0 && (
                                    <span className={`index-change ${index.changePercent >= 0 ? 'positive' : 'negative'}`}>
                                        {index.changePercent >= 0 ? <ArrowUpRight size={12} /> : <ArrowDownRight size={12} />}
                                        {formatPercent(index.changePercent)}
                                    </span>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

// === Order Book Widget ===
interface OrderBookWidgetProps {
    onPriceClick?: (price: number, side: 'Buy' | 'Sell') => void;
}

const OrderBookWidget: React.FC<OrderBookWidgetProps> = ({ onPriceClick }) => {
    const { activeSymbol, orderBooks, stockTradeHistory, requestStockTrades } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const orderBook = orderBooks[activeSymbol];
    const [activeTab, setActiveTab] = React.useState<'depth' | 'trades'>('depth');

    // Calculate max quantity for depth visualization
    const maxQty = Math.max(
        ...(orderBook?.bids.map(b => b.quantity) || [1]),
        ...(orderBook?.asks.map(a => a.quantity) || [1])
    );

    const handlePriceClick = (price: number, side: 'Buy' | 'Sell') => {
        if (onPriceClick) {
            onPriceClick(price, side);
        }
    };

    // Load stock trades when switching to trades tab or symbol changes
    React.useEffect(() => {
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
                        {orderBook?.bids.slice(0, 8).map((level, i) => (
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
                        {orderBook?.asks.slice(0, 8).map((level, i) => (
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
                            {stockTrades.slice(0, 15).map((trade) => (
                                <div key={trade.trade_id} className="stock-trade-row">
                                    <span className="trade-time">
                                        {new Date(trade.timestamp * 1000).toLocaleTimeString()}
                                    </span>
                                    <span className={`trade-price ${trade.side === 'Buy' ? 'positive' : 'negative'}`}>
                                        {formatCurrency(trade.price / 10000)}
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

// === Portfolio Widget ===
interface PortfolioWidgetProps {
    onQuickSell?: (symbol: string, qty: number, price: number) => void;
}

const PortfolioWidget: React.FC<PortfolioWidgetProps> = ({ onQuickSell }) => {
    const { money, lockedMoney, marginLocked, portfolio, orderBooks, setActiveSymbol, tradeHistory, requestTradeHistory } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [activeTab, setActiveTab] = React.useState<'positions' | 'history'>('positions');

    // Get P&L for each position - use server-provided values when available
    const getPositionPnL = (item: typeof portfolio[0]) => {
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
    };

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
    React.useEffect(() => {
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
                            {tradeHistory.slice(0, 20).map((trade) => (
                                <div key={trade.trade_id} className="trade-history-row">
                                    <div className="trade-info">
                                        <Badge variant={trade.side === 'Buy' ? 'buy' : 'sell'}>
                                            {trade.side}
                                        </Badge>
                                        <span className="symbol">{trade.symbol}</span>
                                    </div>
                                    <div className="trade-details">
                                        <span className="qty-price">{trade.qty} @ {formatCurrency(trade.price / 10000)}</span>
                                        <span className="total">{formatCurrency(trade.total_value / 10000)}</span>
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

// === Order Confirmation Modal ===
interface OrderConfirmationProps {
    isOpen: boolean;
    onClose: () => void;
    onConfirm: () => void;
    orderDetails: {
        symbol: string;
        side: 'Buy' | 'Sell' | 'Short';
        orderType: 'Market' | 'Limit';
        qty: number;
        price: number;
        timeInForce: 'GTC' | 'IOC';
        estimatedTotal: number;
    } | null;
}

const OrderConfirmationModal: React.FC<OrderConfirmationProps> = ({
    isOpen,
    onClose,
    onConfirm,
    orderDetails
}) => {
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    if (!orderDetails) return null;

    const getSideColor = (side: string) => {
        switch (side) {
            case 'Buy': return 'var(--color-buy)';
            case 'Sell': return 'var(--color-sell)';
            case 'Short': return 'var(--color-warning)';
            default: return 'var(--text-primary)';
        }
    };

    const getSideIcon = (side: string) => {
        switch (side) {
            case 'Buy': return <ArrowUpRight size={24} style={{ color: getSideColor(side) }} />;
            case 'Sell': return <ArrowDownRight size={24} style={{ color: getSideColor(side) }} />;
            case 'Short': return <AlertTriangle size={24} style={{ color: getSideColor(side) }} />;
            default: return null;
        }
    };

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Confirm Order"
            size="sm"
        >
            <div className="order-confirmation-content">
                <div className="order-confirmation-header">
                    {getSideIcon(orderDetails.side)}
                    <div>
                        <span className="confirmation-action" style={{ color: getSideColor(orderDetails.side) }}>
                            {orderDetails.side}
                        </span>
                        <span className="confirmation-symbol">{orderDetails.symbol}</span>
                    </div>
                </div>

                <div className="order-confirmation-details">
                    <div className="confirmation-row">
                        <span className="confirmation-label">Order Type</span>
                        <span className="confirmation-value">{orderDetails.orderType}</span>
                    </div>
                    <div className="confirmation-row">
                        <span className="confirmation-label">Quantity</span>
                        <span className="confirmation-value">{orderDetails.qty} shares</span>
                    </div>
                    <div className="confirmation-row">
                        <span className="confirmation-label">Price</span>
                        <span className="confirmation-value">
                            {orderDetails.orderType === 'Market' ? 'Market Price' : formatCurrency(orderDetails.price)}
                        </span>
                    </div>
                    {orderDetails.orderType === 'Limit' && (
                        <div className="confirmation-row">
                            <span className="confirmation-label">Time in Force</span>
                            <span className="confirmation-value">
                                {orderDetails.timeInForce === 'GTC' ? "Good 'Til Cancelled" : 'Immediate or Cancel'}
                            </span>
                        </div>
                    )}
                    <div className="confirmation-row total">
                        <span className="confirmation-label">Estimated Total</span>
                        <span className="confirmation-value">{formatCurrency(orderDetails.estimatedTotal)}</span>
                    </div>
                </div>

                {orderDetails.side === 'Short' && (
                    <div className="order-confirmation-warning">
                        <AlertTriangle size={16} />
                        <span>Short selling requires 150% margin. Make sure you have sufficient funds.</span>
                    </div>
                )}

                {orderDetails.orderType === 'Market' && (
                    <div className="order-confirmation-info">
                        <AlertTriangle size={16} />
                        <span>Market orders execute immediately at the best available price.</span>
                    </div>
                )}

                <div className="order-confirmation-actions">
                    <Button variant="secondary" onClick={onClose}>
                        Cancel
                    </Button>
                    <Button
                        variant={orderDetails.side === 'Buy' ? 'success' : orderDetails.side === 'Sell' ? 'danger' : 'warning'}
                        onClick={onConfirm}
                    >
                        <CheckCircle size={16} />
                        Confirm {orderDetails.side}
                    </Button>
                </div>
            </div>
        </Modal>
    );
};

// === Quick Trade Widget ===
interface QuickTradeWidgetProps {
    externalPrice?: number;
    externalSide?: 'Buy' | 'Sell' | 'Short';
    externalQty?: number;
    onExternalUpdate?: () => void;
    isCollapsed: boolean;
    onToggle: () => void;
}

const QuickTradeWidget: React.FC<QuickTradeWidgetProps> = ({
    externalPrice,
    externalSide,
    externalQty,
    onExternalUpdate,
    isCollapsed,
    onToggle
}) => {
    const { activeSymbol, placeOrder, orderBooks, candles } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [side, setSide] = React.useState<'Buy' | 'Sell' | 'Short'>('Buy');
    const [orderType, setOrderType] = React.useState<'Market' | 'Limit'>('Limit');
    const [qty, setQty] = React.useState('10');
    const [price, setPrice] = React.useState('');
    const [timeInForce, setTimeInForce] = React.useState<'GTC' | 'IOC'>('GTC');

    // Confirmation modal state
    const [showConfirmation, setShowConfirmation] = React.useState(false);
    const [pendingOrder, setPendingOrder] = React.useState<OrderConfirmationProps['orderDetails']>(null);

    const orderBook = orderBooks[activeSymbol];
    const bestAsk = orderBook?.asks[0]?.price;
    const bestBid = orderBook?.bids[0]?.price;

    // Get last price from candles if no order book data
    const symbolCandles = candles[activeSymbol] || [];
    const lastCandlePrice = symbolCandles.length > 0 ? symbolCandles[symbolCandles.length - 1].close : null;

    // Use order book price, or last candle price, or default to 100
    const lastPrice = bestAsk || bestBid || lastCandlePrice || 100;

    // Get appropriate market price based on side
    const marketPrice = side === 'Buy' ? bestAsk : bestBid;

    // Check if order book has liquidity
    const hasLiquidity = orderBook && (orderBook.bids.length > 0 || orderBook.asks.length > 0);

    React.useEffect(() => {
        if (!price && lastPrice) {
            setPrice(lastPrice.toFixed(2));
        }
    }, [lastPrice, price]);

    // Update price when switching between market and limit
    React.useEffect(() => {
        if (orderType === 'Limit' && marketPrice) {
            setPrice(marketPrice.toFixed(2));
        }
    }, [orderType, marketPrice]);

    // Handle external updates (from order book click or portfolio quick sell)
    React.useEffect(() => {
        if (externalPrice !== undefined) {
            setPrice(externalPrice.toFixed(2));
            setOrderType('Limit');
        }
        if (externalSide !== undefined) {
            setSide(externalSide);
        }
        if (externalQty !== undefined) {
            setQty(externalQty.toString());
        }
        if (onExternalUpdate) {
            onExternalUpdate();
        }
    }, [externalPrice, externalSide, externalQty, onExternalUpdate]);

    const estimatedTotal = orderType === 'Market'
        ? (parseInt(qty) || 0) * (marketPrice || 0)
        : (parseInt(qty) || 0) * (parseFloat(price) || 0);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        const parsedQty = parseInt(qty);
        if (!parsedQty || parsedQty <= 0) return;

        const parsedPrice = orderType === 'Limit' ? parseFloat(price) : (marketPrice || 0);
        if (orderType === 'Limit' && (!parsedPrice || parsedPrice <= 0)) return;

        // Prepare order details for confirmation
        setPendingOrder({
            symbol: activeSymbol,
            side,
            orderType,
            qty: parsedQty,
            price: parsedPrice,
            timeInForce: orderType === 'Market' ? 'IOC' : timeInForce,
            estimatedTotal,
        });
        setShowConfirmation(true);
    };

    const handleConfirmOrder = () => {
        if (!pendingOrder) return;

        if (pendingOrder.orderType === 'Limit') {
            placeOrder({
                symbol: pendingOrder.symbol,
                side: pendingOrder.side,
                orderType: 'Limit',
                qty: pendingOrder.qty,
                price: pendingOrder.price,
                timeInForce: pendingOrder.timeInForce,
            });
        } else {
            placeOrder({
                symbol: pendingOrder.symbol,
                side: pendingOrder.side,
                orderType: 'Market',
                qty: pendingOrder.qty,
                price: 0,
                timeInForce: 'IOC',
            });
        }

        setShowConfirmation(false);
        setPendingOrder(null);
    };

    const handleCancelConfirmation = () => {
        setShowConfirmation(false);
        setPendingOrder(null);
    };

    const sideTabs = [
        { id: 'Buy', label: 'Buy' },
        { id: 'Sell', label: 'Sell' },
        { id: 'Short', label: 'Short' },
    ];

    return (
        <div className={`quick-trade-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <BarChart2 size={14} /> Quick Trade
                </span>
                <Badge variant="primary">{activeSymbol}</Badge>
            </div>
            {!isCollapsed && (
            <form onSubmit={handleSubmit} className="trade-form">
                <Tabs
                    tabs={sideTabs}
                    activeTab={side}
                    onChange={(id) => setSide(id as 'Buy' | 'Sell' | 'Short')}
                    variant="trading"
                />

                {/* Order Type Toggle */}
                <div className="order-type-toggle">
                    <button
                        type="button"
                        className={`order-type-btn ${orderType === 'Market' ? 'active' : ''}`}
                        onClick={() => setOrderType('Market')}
                    >
                        Market
                    </button>
                    <button
                        type="button"
                        className={`order-type-btn ${orderType === 'Limit' ? 'active' : ''}`}
                        onClick={() => setOrderType('Limit')}
                    >
                        Limit
                    </button>
                </div>

                <div className="trade-inputs">
                    <div className="input-group">
                        <label>Qty</label>
                        <input
                            type="number"
                            value={qty}
                            onChange={(e) => setQty(e.target.value)}
                            min="1"
                        />
                    </div>
                    {orderType === 'Limit' ? (
                        <div className="input-group">
                            <label>Price</label>
                            <input
                                type="number"
                                value={price}
                                onChange={(e) => setPrice(e.target.value)}
                                step="0.01"
                            />
                        </div>
                    ) : (
                        <div className="input-group">
                            <label>Market Price</label>
                            <div className={`market-price-display ${!marketPrice ? 'no-data' : ''}`}>
                                {marketPrice
                                    ? formatCurrency(marketPrice)
                                    : hasLiquidity
                                        ? `~${formatCurrency(lastPrice)}`
                                        : 'No liquidity'
                                }
                            </div>
                        </div>
                    )}
                </div>

                {/* Time in Force (only for limit orders) */}
                {orderType === 'Limit' && (
                    <div className="time-in-force">
                        <label>Time in Force</label>
                        <div className="tif-options">
                            <button
                                type="button"
                                className={`tif-btn ${timeInForce === 'GTC' ? 'active' : ''}`}
                                onClick={() => setTimeInForce('GTC')}
                                title="Good 'Til Cancelled"
                            >
                                GTC
                            </button>
                            <button
                                type="button"
                                className={`tif-btn ${timeInForce === 'IOC' ? 'active' : ''}`}
                                onClick={() => setTimeInForce('IOC')}
                                title="Immediate or Cancel"
                            >
                                IOC
                            </button>
                        </div>
                    </div>
                )}

                <div className="trade-total">
                    <span>Est. Total</span>
                    <span className="total-value">{formatCurrency(estimatedTotal)}</span>
                </div>
                <button
                    type="submit"
                    className={`trade-btn ${side.toLowerCase()}`}
                    disabled={orderType === 'Market' && !marketPrice}
                    title={orderType === 'Market' && !marketPrice ? 'No market liquidity available. Use a limit order instead.' : undefined}
                >
                    {orderType === 'Market'
                        ? marketPrice
                            ? `${side} at Market`
                            : 'No Market Liquidity'
                        : `${side} ${activeSymbol}`
                    }
                </button>
            </form>
            )}

            {/* Order Confirmation Modal */}
            <OrderConfirmationModal
                isOpen={showConfirmation}
                onClose={handleCancelConfirmation}
                onConfirm={handleConfirmOrder}
                orderDetails={pendingOrder}
            />
        </div>
    );
};

// === Open Orders Widget ===
interface OpenOrdersWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

const OpenOrdersWidget: React.FC<OpenOrdersWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { openOrders, cancelOrder } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    const handleCancelAll = (e: React.MouseEvent) => {
        e.stopPropagation(); // Prevent toggle when clicking cancel all
        openOrders.forEach(order => {
            cancelOrder(order.symbol, order.id);
        });
    };

    return (
        <div className={`open-orders-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <BarChart2 size={14} /> Open Orders
                </span>
                <div className="widget-header-actions">
                    {openOrders.length > 0 && (
                        <>
                            <Badge variant="primary">{openOrders.length}</Badge>
                            <button
                                className="cancel-all-btn"
                                onClick={handleCancelAll}
                                title="Cancel All Orders"
                            >
                                <XCircle size={14} />
                                Cancel All
                            </button>
                        </>
                    )}
                </div>
            </div>
            {!isCollapsed && (
                <div className="orders-list">
                    {openOrders.length === 0 && <div className="empty-state">No open orders</div>}
                    {openOrders.map((order) => (
                        <div key={order.id} className="order-row">
                            <div className="order-info">
                                <Badge variant={order.side === 'Buy' ? 'buy' : order.side === 'Sell' ? 'sell' : 'short'}>
                                    {order.side}
                                </Badge>
                                <span className="symbol">{order.symbol}</span>
                            </div>
                            <div className="order-details">
                                <span className="qty-price">{order.qty} @ {formatCurrency(order.price)}</span>
                                <span className="filled">{order.filledQty || 0} filled</span>
                            </div>
                            <button className="cancel-btn" onClick={() => cancelOrder(order.symbol, order.id)}>
                                <X size={14} />
                            </button>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

// === News Ticker (Animated Marquee) ===
const NewsTicker: React.FC = () => {
    const { news, companies } = useGameStore();

    const getSentimentColor = (sentiment: string) => {
        switch (sentiment) {
            case 'Bullish': return 'var(--color-success)';
            case 'Bearish': return 'var(--color-danger)';
            default: return 'var(--text-muted)';
        }
    };

    // Get stock symbols for highlighting
    const stockSymbols = companies.map(c => c.symbol);
    const keywordPatterns = [
        ...stockSymbols,
        'bullish', 'bearish', 'rally', 'crash', 'surge', 'plunge',
        'profit', 'loss', 'growth', 'decline', 'buy', 'sell'
    ];

    // Highlight keywords in headline
    const highlightHeadline = (headline: string) => {
        const regex = new RegExp(`\\b(${keywordPatterns.join('|')})\\b`, 'gi');
        const parts = headline.split(regex);

        return parts.map((part, i) => {
            const lowerPart = part.toLowerCase();
            const isSymbol = stockSymbols.some(s => s.toLowerCase() === lowerPart);
            const isBullish = ['bullish', 'rally', 'surge', 'profit', 'growth', 'buy'].includes(lowerPart);
            const isBearish = ['bearish', 'crash', 'plunge', 'loss', 'decline', 'sell'].includes(lowerPart);

            if (isSymbol) {
                return <span key={i} className="news-keyword">{part}</span>;
            } else if (isBullish) {
                return <span key={i} className="news-keyword bullish">{part}</span>;
            } else if (isBearish) {
                return <span key={i} className="news-keyword bearish">{part}</span>;
            }
            return part;
        });
    };

    // Duplicate items for seamless infinite scroll
    const duplicatedNews = [...news, ...news];

    // Calculate animation duration based on number of items (more items = slower scroll)
    // Base: 15s for 1 item, add 5s per additional item
    const animationDuration = Math.max(20, news.length * 5);

    return (
        <div className="news-ticker">
            <div className="news-label">
                <Newspaper size={14} />
                <span>NEWS</span>
            </div>
            <div className="news-content">
                {news.length === 0 ? (
                    <span className="text-muted" style={{ padding: '0 var(--space-2)' }}>Waiting for news...</span>
                ) : (
                    <div
                        className="news-scroll"
                        style={{ animationDuration: `${animationDuration}s` }}
                    >
                        {duplicatedNews.map((item, i) => (
                            <span key={`${item.id}-${i}`} className="news-item">
                                <span className="sentiment-dot" style={{ background: getSentimentColor(item.sentiment) }} />
                                {highlightHeadline(item.headline)}
                            </span>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

// === Leaderboard Widget (Compact, for sidebar) ===
interface LeaderboardWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

const LeaderboardWidget: React.FC<LeaderboardWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { leaderboard } = useGameStore();
    const { user } = useAuthStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    return (
        <div className={`leaderboard-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <Trophy size={14} /> Leaderboard
                </span>
                <Badge variant="primary">{leaderboard.length}</Badge>
            </div>
            {!isCollapsed && (
                <div className="leaderboard-content">
                    {leaderboard.length === 0 && (
                        <div className="empty-state">Loading...</div>
                    )}
                    {leaderboard.slice(0, 10).map((entry) => {
                        const isCurrentUser = entry.name === user?.name;
                        return (
                            <div key={entry.rank} className={`leaderboard-row ${isCurrentUser ? 'current-user' : ''}`}>
                                <div className="rank-info">
                                    <span className={`rank ${entry.rank <= 3 ? 'top' : ''}`}>{entry.rank}</span>
                                    <span className="name">{entry.name}</span>
                                </div>
                                <span className="net-worth">{formatCurrency(entry.netWorth)}</span>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

// === Chat Widget (Compact, for sidebar) ===
interface ChatWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

const ChatWidget: React.FC<ChatWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { chatMessages, sendChatMessage } = useGameStore();
    const [message, setMessage] = React.useState('');
    const messagesContainerRef = React.useRef<HTMLDivElement>(null);

    React.useEffect(() => {
        // Scroll within the container instead of using scrollIntoView
        // to avoid affecting the entire page scroll position
        if (messagesContainerRef.current) {
            messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
        }
    }, [chatMessages]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (message.trim()) {
            console.log('[Chat] Sending message:', message.trim());
            sendChatMessage(message.trim());
            setMessage('');
        }
    };

    return (
        <div className={`chat-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <MessageCircle size={14} /> Chat
                </span>
                <Badge variant="primary">{chatMessages.length}</Badge>
            </div>
            {!isCollapsed && (
                <>
                    <div className="chat-messages" ref={messagesContainerRef}>
                        {chatMessages.length === 0 && (
                            <div className="empty-state" style={{ fontSize: '12px' }}>No messages yet</div>
                        )}
                        {chatMessages.slice(-20).map((msg) => (
                            <div key={msg.id} className="chat-message" style={{ fontSize: '12px', marginBottom: '4px' }}>
                                <span className="chat-username" style={{ fontWeight: 600, color: 'var(--color-primary)' }}>{msg.username}: </span>
                                <span className="chat-text">{msg.message}</span>
                            </div>
                        ))}
                    </div>
                    <form onSubmit={handleSubmit} className="chat-input-form" style={{ padding: '8px', borderTop: '1px solid var(--border-secondary)' }}>
                        <input
                            type="text"
                            placeholder="Type a message..."
                            value={message}
                            onChange={(e) => setMessage(e.target.value)}
                            maxLength={500}
                            style={{ flex: 1, padding: '6px 8px', fontSize: '12px' }}
                        />
                        <button type="submit" style={{ padding: '6px 12px', fontSize: '12px' }}>Send</button>
                    </form>
                </>
            )}
        </div>
    );
};

// === Main Trading Desk Page ===
export const TradingDeskPage: React.FC = () => {
    const { activeSymbol } = useGameStore();

    console.log('[TradingDesk] Rendering with activeSymbol:', activeSymbol);

    // Collapsible widget states
    const [leaderboardCollapsed, setLeaderboardCollapsed] = React.useState(false);
    const [chatCollapsed, setChatCollapsed] = React.useState(false);
    const [quickTradeCollapsed, setQuickTradeCollapsed] = React.useState(false);
    const [openOrdersCollapsed, setOpenOrdersCollapsed] = React.useState(false);

    // Shared trade form state - for order book click and portfolio quick sell
    const [tradeFormState, setTradeFormState] = React.useState<{
        price?: number;
        side?: 'Buy' | 'Sell' | 'Short';
        qty?: number;
        updateKey: number;
    }>({ updateKey: 0 });

    // Handle order book price click
    const handleOrderBookPriceClick = (price: number, side: 'Buy' | 'Sell') => {
        console.log('[TradingDesk] Order book price clicked:', price, side);
        setTradeFormState(prev => ({
            price,
            side,
            qty: undefined,
            updateKey: prev.updateKey + 1
        }));
    };

    // Handle portfolio quick sell
    const handleQuickSell = (_symbol: string, qty: number, price: number) => {
        console.log('[TradingDesk] Quick sell:', _symbol, qty, price);
        setTradeFormState(prev => ({
            price,
            side: 'Sell',
            qty,
            updateKey: prev.updateKey + 1
        }));
    };

    // Clear external state after it's been consumed
    const clearExternalState = React.useCallback(() => {
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
