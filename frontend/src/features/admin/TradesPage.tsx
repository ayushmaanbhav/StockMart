// ============================================
// Admin Trade History Page
// ============================================

import React, { useState, useEffect } from 'react';
import {
    ArrowRightLeft,
    Search,
    Filter,
    ChevronUp,
    ChevronDown,
    RefreshCw
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { useConfigStore } from '../../store/configStore';
import { useAdminStore } from '../../store/adminStore';
import { Button, Badge } from '../../components/common';

// Sort icon component - defined outside to avoid recreation during render
interface TradeSortIconProps {
    field: string;
    sortField: string;
    sortOrder: 'asc' | 'desc';
}

const TradeSortIcon: React.FC<TradeSortIconProps> = ({ field, sortField, sortOrder }) => {
    if (sortField !== field) return null;
    return sortOrder === 'asc' ? <ChevronUp size={14} /> : <ChevronDown size={14} />;
};

export const TradesPage: React.FC = () => {
    const { companies } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const { trades, tradesTotal, tradesHasMore, tradesLoading, fetchTrades } = useAdminStore();
    const [page, setPage] = useState(0);

    // Filters
    const [symbolFilter, setSymbolFilter] = useState<string>('');
    const [searchQuery, setSearchQuery] = useState('');
    const [sortField, setSortField] = useState<'timestamp' | 'symbol' | 'qty' | 'price'>('timestamp');
    const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

    // Load trades on mount and when filters change
    useEffect(() => {
        fetchTrades(page, undefined, symbolFilter || undefined);
    }, [page, symbolFilter, fetchTrades]);

    // Filter and sort trades locally
    const displayedTrades = React.useMemo(() => {
        let filtered = [...trades];

        // Apply search filter (by trader name)
        if (searchQuery) {
            filtered = filtered.filter(t =>
                t.buyer_name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
                t.seller_name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
                t.symbol.toLowerCase().includes(searchQuery.toLowerCase())
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
    }, [trades, searchQuery, sortField, sortOrder]);

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

    return (
        <div className="trades-page">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold mb-1">Trade History</h1>
                    <p className="text-muted">
                        {tradesTotal} total trades
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
                            placeholder="Search trades..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            style={{ paddingLeft: '40px', width: '200px' }}
                        />
                    </div>
                    <select
                        className="input"
                        value={symbolFilter}
                        onChange={(e) => {
                            setSymbolFilter(e.target.value);
                            setPage(0);
                        }}
                        style={{ width: '150px' }}
                    >
                        <option value="">All Stocks</option>
                        {companies.map(c => (
                            <option key={c.symbol} value={c.symbol}>{c.symbol}</option>
                        ))}
                    </select>
                    <Button variant="secondary" onClick={() => fetchTrades(page, undefined, symbolFilter || undefined)} disabled={tradesLoading}>
                        <RefreshCw size={16} className={tradesLoading ? 'animate-spin' : ''} />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-4 gap-4 mb-6">
                <div className="stat-card">
                    <div className="stat-label">
                        <ArrowRightLeft size={14} />
                        Total Trades
                    </div>
                    <div className="stat-value">{tradesTotal}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <Filter size={14} />
                        Displayed
                    </div>
                    <div className="stat-value">{displayedTrades.length}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Total Volume</div>
                    <div className="stat-value">
                        {displayedTrades
                            .reduce((sum, t) => sum + t.qty, 0)
                            .toLocaleString()}
                    </div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">Total Value</div>
                    <div className="stat-value">
                        {formatCurrency(displayedTrades
                            .reduce((sum, t) => sum + t.total_value, 0))}
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
                                        <TradeSortIcon field="timestamp" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('symbol')}>
                                    <div className="flex items-center gap-1">
                                        Symbol
                                        <TradeSortIcon field="symbol" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Type</th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('qty')}>
                                    <div className="flex items-center gap-1">
                                        Qty
                                        <TradeSortIcon field="qty" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th style={{ cursor: 'pointer' }} onClick={() => handleSort('price')}>
                                    <div className="flex items-center gap-1">
                                        Price
                                        <TradeSortIcon field="price" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Total</th>
                                <th>Buyer → Seller</th>
                            </tr>
                        </thead>
                        <tbody>
                            {tradesLoading ? (
                                <tr>
                                    <td colSpan={7} className="text-center text-muted py-8">
                                        Loading trades...
                                    </td>
                                </tr>
                            ) : displayedTrades.length === 0 ? (
                                <tr>
                                    <td colSpan={7} className="text-center text-muted py-8">
                                        No trades found
                                    </td>
                                </tr>
                            ) : (
                                displayedTrades.map((trade) => (
                                    <tr key={trade.trade_id}>
                                        <td className="text-muted text-sm">
                                            {formatTime(trade.timestamp)}
                                        </td>
                                        <td>
                                            <span className="font-bold">{trade.symbol}</span>
                                        </td>
                                        <td>
                                            <Badge variant="buy">Buy</Badge>
                                        </td>
                                        <td className="font-mono">{trade.qty}</td>
                                        <td className="font-mono">
                                            {formatCurrency(trade.price)}
                                        </td>
                                        <td className="font-mono font-bold">
                                            {formatCurrency(trade.total_value)}
                                        </td>
                                        <td className="text-muted text-sm">
                                            <span className="text-success">{trade.buyer_name}</span>
                                            {' → '}
                                            <span className="text-danger">{trade.seller_name}</span>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>

                {/* Pagination */}
                {tradesHasMore || page > 0 ? (
                    <div className="flex justify-center gap-2 p-4 border-t border-secondary">
                        <Button
                            variant="secondary"
                            size="sm"
                            onClick={() => setPage(p => Math.max(0, p - 1))}
                            disabled={page === 0}
                        >
                            Previous
                        </Button>
                        <span className="px-4 py-2 text-muted">
                            Page {page + 1} of {Math.ceil(tradesTotal / 20)}
                        </span>
                        <Button
                            variant="secondary"
                            size="sm"
                            onClick={() => setPage(p => p + 1)}
                            disabled={!tradesHasMore}
                        >
                            Next
                        </Button>
                    </div>
                ) : null}
            </div>
        </div>
    );
};

export default TradesPage;
