// ============================================
// Quick Trade Widget Component
// Order entry form with confirmation
// ============================================

import React, { useState, useEffect } from 'react';
import { BarChart2, ChevronDown, ChevronRight, AlertTriangle, CheckCircle } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';
import { Badge, Tabs, Modal, Button } from '../../../components/common';
import { TRADING } from '../../../constants';

interface QuickTradeWidgetProps {
    externalPrice?: number;
    externalSide?: 'Buy' | 'Sell' | 'Short';
    externalQty?: number;
    onExternalUpdate?: () => void;
    isCollapsed: boolean;
    onToggle: () => void;
}

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

    return (
        <Modal
            isOpen={isOpen}
            onClose={onClose}
            title="Confirm Order"
            size="sm"
        >
            <div className="order-confirmation-content">
                <div className="order-confirmation-header">
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
                        <span>Short selling requires {TRADING.SHORT_MARGIN_PERCENT}% margin. Make sure you have sufficient funds.</span>
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

export const QuickTradeWidget: React.FC<QuickTradeWidgetProps> = ({
    externalPrice,
    externalSide,
    externalQty,
    onExternalUpdate,
    isCollapsed,
    onToggle
}) => {
    const { activeSymbol, placeOrder, orderBooks, candles } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [side, setSide] = useState<'Buy' | 'Sell' | 'Short'>('Buy');
    const [orderType, setOrderType] = useState<'Market' | 'Limit'>('Limit');
    const [qty, setQty] = useState(TRADING.DEFAULT_ORDER_QTY.toString());
    const [price, setPrice] = useState('');
    const [timeInForce, setTimeInForce] = useState<'GTC' | 'IOC'>('GTC');

    // Confirmation modal state
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [pendingOrder, setPendingOrder] = useState<OrderConfirmationProps['orderDetails']>(null);

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

    useEffect(() => {
        if (!price && lastPrice) {
            // Use queueMicrotask to avoid synchronous setState in effect body
            queueMicrotask(() => setPrice(lastPrice.toFixed(2)));
        }
    }, [lastPrice, price]);

    // Update price when switching between market and limit
    useEffect(() => {
        if (orderType === 'Limit' && marketPrice) {
            // Use queueMicrotask to avoid synchronous setState in effect body
            queueMicrotask(() => setPrice(marketPrice.toFixed(2)));
        }
    }, [orderType, marketPrice]);

    // Handle external updates (from order book click or portfolio quick sell)
    useEffect(() => {
        // Use queueMicrotask to avoid synchronous setState in effect body
        queueMicrotask(() => {
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
        });
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

export default QuickTradeWidget;
