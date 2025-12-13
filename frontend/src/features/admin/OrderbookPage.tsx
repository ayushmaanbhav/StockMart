// ============================================
// Admin Orderbook View Page
// ============================================

import React, { useState, useEffect } from 'react';
import {
    BookOpen,
    RefreshCw,
    TrendingUp,
    TrendingDown
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { useConfigStore } from '../../store/configStore';
import { useAdminStore } from '../../store/adminStore';
import { Button } from '../../components/common';

export const OrderbookPage: React.FC = () => {
    const { companies } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const {
        orderbookSymbol,
        orderbookBids,
        orderbookAsks,
        orderbookLoading,
        fetchOrderbook
    } = useAdminStore();

    // Initialize selectedSymbol lazily based on companies
    const [selectedSymbol, setSelectedSymbol] = useState<string>(() =>
        companies.length > 0 ? companies[0].symbol : ''
    );

    // Track if we need to initialize when companies load
    const initializedRef = React.useRef(false);
    useEffect(() => {
        if (!initializedRef.current && companies.length > 0 && !selectedSymbol) {
            // Use queueMicrotask to avoid synchronous setState in effect body
            queueMicrotask(() => setSelectedSymbol(companies[0].symbol));
            initializedRef.current = true;
        }
    }, [companies, selectedSymbol]);

    // Fetch orderbook when symbol changes
    useEffect(() => {
        if (selectedSymbol) {
            fetchOrderbook(selectedSymbol);
        }
    }, [selectedSymbol, fetchOrderbook]);

    // Calculate stats
    const totalBidVolume = orderbookBids.reduce((sum, o) => sum + o.remaining_qty, 0);
    const totalAskVolume = orderbookAsks.reduce((sum, o) => sum + o.remaining_qty, 0);
    const bestBid = orderbookBids.length > 0 ? Math.max(...orderbookBids.map(o => o.price)) : 0;
    const bestAsk = orderbookAsks.length > 0 ? Math.min(...orderbookAsks.map(o => o.price)) : 0;
    const spread = bestAsk > 0 && bestBid > 0 ? bestAsk - bestBid : 0;
    const spreadPercent = bestBid > 0 ? (spread / bestBid) * 100 : 0;

    const formatTime = (timestamp: number) => {
        const date = new Date(timestamp * 1000);
        return date.toLocaleTimeString();
    };

    return (
        <div className="orderbook-page">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold mb-1">Orderbook View</h1>
                    <p className="text-muted">
                        View individual orders in the orderbook
                    </p>
                </div>
                <div className="flex gap-3">
                    <select
                        className="input"
                        value={selectedSymbol}
                        onChange={(e) => setSelectedSymbol(e.target.value)}
                        style={{ width: '150px' }}
                    >
                        {companies.map(c => (
                            <option key={c.symbol} value={c.symbol}>{c.symbol} - {c.name}</option>
                        ))}
                    </select>
                    <Button
                        variant="secondary"
                        onClick={() => selectedSymbol && fetchOrderbook(selectedSymbol)}
                        disabled={orderbookLoading || !selectedSymbol}
                    >
                        <RefreshCw size={16} className={orderbookLoading ? 'animate-spin' : ''} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-5 gap-4 mb-6">
                <div className="stat-card">
                    <div className="stat-label">
                        <BookOpen size={14} />
                        Symbol
                    </div>
                    <div className="stat-value">{orderbookSymbol || '-'}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <TrendingUp size={14} />
                        Best Bid
                    </div>
                    <div className="stat-value text-success">
                        {bestBid > 0 ? formatCurrency(bestBid) : '-'}
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <TrendingDown size={14} />
                        Best Ask
                    </div>
                    <div className="stat-value text-danger">
                        {bestAsk > 0 ? formatCurrency(bestAsk) : '-'}
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Spread</div>
                    <div className="stat-value">
                        {spread > 0 ? `${formatCurrency(spread)} (${spreadPercent.toFixed(2)}%)` : '-'}
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Volume Imbalance</div>
                    <div className={`stat-value ${totalBidVolume >= totalAskVolume ? 'text-success' : 'text-danger'}`}>
                        {totalBidVolume > 0 || totalAskVolume > 0
                            ? `${((totalBidVolume - totalAskVolume) / Math.max(totalBidVolume + totalAskVolume, 1) * 100).toFixed(1)}%`
                            : '-'}
                    </div>
                </div>
            </div>

            {/* Orderbook Tables */}
            <div className="grid grid-cols-2 gap-4">
                {/* Bids (Buy Orders) */}
                <div className="panel">
                    <div className="panel-header" style={{ background: 'var(--color-success-bg)' }}>
                        <div className="panel-title text-success">
                            <TrendingUp size={18} />
                            Bids ({orderbookBids.length} orders, {totalBidVolume.toLocaleString()} shares)
                        </div>
                    </div>
                    <div className="table-container" style={{ maxHeight: '500px', overflowY: 'auto' }}>
                        <table className="table">
                            <thead>
                                <tr>
                                    <th>Price</th>
                                    <th>Qty</th>
                                    <th>Trader</th>
                                    <th>Time</th>
                                </tr>
                            </thead>
                            <tbody>
                                {orderbookLoading ? (
                                    <tr>
                                        <td colSpan={4} className="text-center text-muted py-8">
                                            Loading bids...
                                        </td>
                                    </tr>
                                ) : orderbookBids.length === 0 ? (
                                    <tr>
                                        <td colSpan={4} className="text-center text-muted py-8">
                                            No bids
                                        </td>
                                    </tr>
                                ) : (
                                    orderbookBids
                                        .sort((a, b) => b.price - a.price) // Highest first
                                        .map((order) => (
                                            <tr key={order.order_id}>
                                                <td className="font-mono text-success font-bold">
                                                    {formatCurrency(order.price)}
                                                </td>
                                                <td className="font-mono">{order.remaining_qty}</td>
                                                <td className="text-muted text-sm">
                                                    {order.user_name || `User #${order.user_id}`}
                                                </td>
                                                <td className="text-muted text-sm">
                                                    {formatTime(order.timestamp)}
                                                </td>
                                            </tr>
                                        ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>

                {/* Asks (Sell Orders) */}
                <div className="panel">
                    <div className="panel-header" style={{ background: 'var(--color-danger-bg)' }}>
                        <div className="panel-title text-danger">
                            <TrendingDown size={18} />
                            Asks ({orderbookAsks.length} orders, {totalAskVolume.toLocaleString()} shares)
                        </div>
                    </div>
                    <div className="table-container" style={{ maxHeight: '500px', overflowY: 'auto' }}>
                        <table className="table">
                            <thead>
                                <tr>
                                    <th>Price</th>
                                    <th>Qty</th>
                                    <th>Trader</th>
                                    <th>Time</th>
                                </tr>
                            </thead>
                            <tbody>
                                {orderbookLoading ? (
                                    <tr>
                                        <td colSpan={4} className="text-center text-muted py-8">
                                            Loading asks...
                                        </td>
                                    </tr>
                                ) : orderbookAsks.length === 0 ? (
                                    <tr>
                                        <td colSpan={4} className="text-center text-muted py-8">
                                            No asks
                                        </td>
                                    </tr>
                                ) : (
                                    orderbookAsks
                                        .sort((a, b) => a.price - b.price) // Lowest first
                                        .map((order) => (
                                            <tr key={order.order_id}>
                                                <td className="font-mono text-danger font-bold">
                                                    {formatCurrency(order.price)}
                                                </td>
                                                <td className="font-mono">{order.remaining_qty}</td>
                                                <td className="text-muted text-sm">
                                                    {order.user_name || `User #${order.user_id}`}
                                                </td>
                                                <td className="text-muted text-sm">
                                                    {formatTime(order.timestamp)}
                                                </td>
                                            </tr>
                                        ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default OrderbookPage;
