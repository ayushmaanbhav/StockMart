// ============================================
// Admin Open Orders Page
// ============================================

import React, { useState, useEffect } from 'react';
import {
    BookOpen,
    Search,
    ChevronUp,
    ChevronDown,
    RefreshCw
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { useConfigStore } from '../../store/configStore';
import { useAdminStore } from '../../store/adminStore';
import { Button, Badge } from '../../components/common';

// Sort icon component - defined outside to avoid recreation during render
interface SortIconProps {
    field: string;
    sortField: string;
    sortOrder: 'asc' | 'desc';
}

const SortIcon: React.FC<SortIconProps> = ({ field, sortField, sortOrder }) => {
    if (sortField !== field) return null;
    return sortOrder === 'asc' ? <ChevronUp size={14} /> : <ChevronDown size={14} />;
};

export const OrdersPage: React.FC = () => {
    const { companies } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const { orders, ordersTotal, ordersLoading, fetchOrders } = useAdminStore();

    // Filters
    const [symbolFilter, setSymbolFilter] = useState<string>('');
    const [searchQuery, setSearchQuery] = useState('');
    const [sortField, setSortField] = useState<'timestamp' | 'symbol' | 'qty' | 'price'>('timestamp');
    const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

    // Load orders on mount and when filters change
    useEffect(() => {
        fetchOrders(symbolFilter || undefined);
    }, [symbolFilter, fetchOrders]);

    // Filter and sort orders locally
    const displayedOrders = React.useMemo(() => {
        let filtered = [...orders];

        // Apply symbol filter
        if (symbolFilter) {
            filtered = filtered.filter(o => o.symbol === symbolFilter);
        }

        // Apply search filter (by trader name or symbol)
        if (searchQuery) {
            filtered = filtered.filter(o =>
                o.symbol.toLowerCase().includes(searchQuery.toLowerCase()) ||
                o.user_name?.toLowerCase().includes(searchQuery.toLowerCase())
            );
        }

        // Sort
        filtered.sort((a, b) => {
            const aVal = a[sortField];
            const bVal = b[sortField];

            if (typeof aVal === 'string' && typeof bVal === 'string') {
                return sortOrder === 'asc'
                    ? aVal.localeCompare(bVal)
                    : bVal.localeCompare(aVal);
            }

            return sortOrder === 'asc'
                ? (aVal as number) - (bVal as number)
                : (bVal as number) - (aVal as number);
        });

        return filtered;
    }, [orders, symbolFilter, searchQuery, sortField, sortOrder]);

    const handleSort = (field: 'timestamp' | 'symbol' | 'qty' | 'price') => {
        if (sortField === field) {
            setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
        } else {
            setSortField(field);
            setSortOrder('desc');
        }
    };

    const formatTime = (timestamp: number) => {
        const date = new Date(timestamp * 1000);
        return date.toLocaleString();
    };

    // Calculate stats
    const buyOrders = displayedOrders.filter(o => o.side === 'Buy');
    const sellOrders = displayedOrders.filter(o => o.side === 'Sell' || o.side === 'Short');
    const totalBuyVolume = buyOrders.reduce((sum, o) => sum + o.remaining_qty, 0);
    const totalSellVolume = sellOrders.reduce((sum, o) => sum + o.remaining_qty, 0);

    return (
        <div className="orders-page">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold mb-1">Open Orders</h1>
                    <p className="text-muted">
                        {ordersTotal} open orders across all traders
                    </p>
                </div>
                <div className="flex gap-3">
                    <div className="input-with-icon">
                        <span className="input-icon">
                            <Search size={18} />
                        </span>
                        <input
                            type="text"
                            className="input"
                            placeholder="Search orders..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            style={{ paddingLeft: '40px', width: '200px' }}
                        />
                    </div>
                    <select
                        className="input"
                        value={symbolFilter}
                        onChange={(e) => setSymbolFilter(e.target.value)}
                        style={{ width: '150px' }}
                    >
                        <option value="">All Stocks</option>
                        {companies.map(c => (
                            <option key={c.symbol} value={c.symbol}>{c.symbol}</option>
                        ))}
                    </select>
                    <Button variant="secondary" onClick={() => fetchOrders(symbolFilter || undefined)} disabled={ordersLoading}>
                        <RefreshCw size={16} className={ordersLoading ? 'animate-spin' : ''} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-4 gap-4 mb-6">
                <div className="stat-card">
                    <div className="stat-label">
                        <BookOpen size={14} />
                        Total Orders
                    </div>
                    <div className="stat-value">{ordersTotal}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Buy Orders</div>
                    <div className="stat-value text-success">{buyOrders.length}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Sell/Short Orders</div>
                    <div className="stat-value text-danger">{sellOrders.length}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Net Volume</div>
                    <div className={`stat-value ${totalBuyVolume >= totalSellVolume ? 'text-success' : 'text-danger'}`}>
                        {(totalBuyVolume - totalSellVolume).toLocaleString()}
                    </div>
                </div>
            </div>

            {/* Table */}
            <div className="panel">
                <div className="table-container">
                    <table className="table">
                        <thead>
                            <tr>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('timestamp')}>
                                    <div className="flex items-center gap-1">
                                        Time
                                        <SortIcon field="timestamp" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('symbol')}>
                                    <div className="flex items-center gap-1">
                                        Symbol
                                        <SortIcon field="symbol" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Side</th>
                                <th>Type</th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('qty')}>
                                    <div className="flex items-center gap-1">
                                        Qty
                                        <SortIcon field="qty" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Filled</th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('price')}>
                                    <div className="flex items-center gap-1">
                                        Price
                                        <SortIcon field="price" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Status</th>
                                <th>Trader</th>
                            </tr>
                        </thead>
                        <tbody>
                            {ordersLoading ? (
                                <tr>
                                    <td colSpan={9} className="text-center text-muted py-8">
                                        Loading orders...
                                    </td>
                                </tr>
                            ) : displayedOrders.length === 0 ? (
                                <tr>
                                    <td colSpan={9} className="text-center text-muted py-8">
                                        No open orders found
                                    </td>
                                </tr>
                            ) : (
                                displayedOrders.map((order) => (
                                    <tr key={order.order_id}>
                                        <td className="text-muted text-sm">
                                            {formatTime(order.timestamp)}
                                        </td>
                                        <td>
                                            <span className="font-bold">{order.symbol}</span>
                                        </td>
                                        <td>
                                            <Badge variant={
                                                order.side === 'Buy' ? 'buy' :
                                                order.side === 'Short' ? 'short' : 'sell'
                                            }>
                                                {order.side}
                                            </Badge>
                                        </td>
                                        <td>
                                            <Badge variant="primary">
                                                {order.order_type}
                                            </Badge>
                                        </td>
                                        <td className="font-mono">{order.qty}</td>
                                        <td className="font-mono text-muted">
                                            {order.filled_qty} / {order.qty}
                                        </td>
                                        <td className="font-mono">
                                            {formatCurrency(order.price)}
                                        </td>
                                        <td>
                                            <Badge variant={order.status === 'Partial' ? 'warning' : 'primary'}>
                                                {order.status}
                                            </Badge>
                                        </td>
                                        <td className="text-muted">
                                            {order.user_name || `User #${order.user_id}`}
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
};

export default OrdersPage;
