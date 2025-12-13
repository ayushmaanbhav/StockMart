// ============================================
// Admin Dashboard Page
// ============================================

import React from 'react';
import {
    Users,
    TrendingUp,
    TrendingDown,
    Activity,
    DollarSign,
    BarChart2,
    Power,
    AlertTriangle,
    Clock,
    Zap,
    Building
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { useConfigStore } from '../../store/configStore';
import { useAdminStore } from '../../store/adminStore';
import websocketService from '../../services/websocket';
import { Badge, Button } from '../../components/common';
import { POLLING } from '../../constants';

// Format helpers
const formatNumber = (value: number) => {
    if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
    return value.toString();
};

// Stats Card Component
interface StatsCardProps {
    label: string;
    value: string | number;
    subValue?: string;
    icon: React.ReactNode;
    trend?: 'up' | 'down' | 'neutral';
}

const StatsCard: React.FC<StatsCardProps> = ({ label, value, subValue, icon, trend }) => {
    return (
        <div className="stat-card">
            <div className="stat-icon">{icon}</div>
            <div className="stat-content">
                <div className="stat-label">{label}</div>
                <div className="stat-value">{value}</div>
                {subValue && (
                    <div className={`stat-subvalue ${trend || ''}`}>
                        {trend === 'up' && <TrendingUp size={12} />}
                        {trend === 'down' && <TrendingDown size={12} />}
                        {subValue}
                    </div>
                )}
            </div>
        </div>
    );
};

export const AdminDashboardPage: React.FC = () => {
    const {
        isConnected,
        marketOpen,
        leaderboard,
        companies,
        haltedSymbols,
        indices,
        trades,
        news
    } = useGameStore();
    const { metrics, fetchMetrics } = useAdminStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    const [isTogglingMarket, setIsTogglingMarket] = React.useState(false);
    const [currentTime, setCurrentTime] = React.useState(() => Date.now());

    // Fetch metrics on mount and periodically
    React.useEffect(() => {
        fetchMetrics();
        const interval = setInterval(fetchMetrics, POLLING.DASHBOARD_METRICS_INTERVAL);
        return () => clearInterval(interval);
    }, [fetchMetrics]);

    // Update current time periodically for halted symbols calculation
    React.useEffect(() => {
        const interval = setInterval(() => setCurrentTime(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    const handleToggleMarket = () => {
        setIsTogglingMarket(true);
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'ToggleMarket',
                payload: { open: !marketOpen }
            }
        });
        // Reset after a short delay
        setTimeout(() => setIsTogglingMarket(false), 1000);
    };

    // Use backend metrics if available, fallback to local state
    const totalTraders = metrics?.total_traders ?? leaderboard.length;
    const activeTraders = metrics?.active_traders ?? 0;
    const totalTrades = metrics?.total_trades ?? 0;
    const totalVolume = metrics?.total_volume ?? 0;
    const recentVolume = metrics?.recent_volume ?? 0;
    const haltedCount = metrics?.halted_symbols_count ?? Object.keys(haltedSymbols).filter(
        s => haltedSymbols[s] > currentTime
    ).length;
    const totalCompanies = companies.length;
    const totalOpenOrders = metrics?.open_orders_count ?? 0;
    const totalMarketCap = metrics?.total_market_cap ?? leaderboard.reduce((sum, t) => sum + t.netWorth, 0);

    return (
        <div className="admin-dashboard">
            {/* Header with Market Control */}
            <div className="dashboard-header">
                <div className="dashboard-title">
                    <h1>Admin Dashboard</h1>
                    <span className="dashboard-subtitle">Real-time market oversight</span>
                </div>
                <div className="market-control">
                    <div className="market-status">
                        <Badge variant={marketOpen ? 'success' : 'danger'} pulse>
                            {marketOpen ? 'MARKET OPEN' : 'MARKET CLOSED'}
                        </Badge>
                        {haltedCount > 0 && (
                            <Badge variant="warning">
                                <AlertTriangle size={12} /> {haltedCount} Circuit Breaker{haltedCount > 1 ? 's' : ''}
                            </Badge>
                        )}
                    </div>
                    <Button
                        variant={marketOpen ? 'danger' : 'success'}
                        size="md"
                        onClick={handleToggleMarket}
                        loading={isTogglingMarket}
                    >
                        <Power size={16} />
                        {marketOpen ? 'Close Market' : 'Open Market'}
                    </Button>
                </div>
            </div>

            {/* Stats Grid */}
            <div className="stats-grid">
                <StatsCard
                    icon={<Users size={20} />}
                    label="Total Traders"
                    value={totalTraders}
                    subValue={`${activeTraders} online`}
                    trend={activeTraders > 0 ? 'up' : 'neutral'}
                />
                <StatsCard
                    icon={<Activity size={20} />}
                    label="Total Trades"
                    value={formatNumber(totalTrades)}
                    subValue={`${formatCurrency(recentVolume)} in last 5min`}
                    trend={recentVolume > 0 ? 'up' : 'neutral'}
                />
                <StatsCard
                    icon={<DollarSign size={20} />}
                    label="Total Volume"
                    value={formatCurrency(totalVolume)}
                    subValue={`${formatCurrency(recentVolume)} recent`}
                    trend={recentVolume > 0 ? 'up' : 'neutral'}
                />
                <StatsCard
                    icon={<Building size={20} />}
                    label="Companies"
                    value={totalCompanies}
                    subValue="listed"
                />
                <StatsCard
                    icon={<Zap size={20} />}
                    label="Open Orders"
                    value={totalOpenOrders}
                    subValue="pending"
                    trend={totalOpenOrders > 0 ? 'up' : 'neutral'}
                />
                <StatsCard
                    icon={<BarChart2 size={20} />}
                    label="Total Market Cap"
                    value={formatCurrency(totalMarketCap)}
                    subValue="combined"
                />
            </div>

            {/* Main Content Grid */}
            <div className="dashboard-content">
                {/* Leaderboard Panel */}
                <div className="panel leaderboard-panel">
                    <div className="panel-header">
                        <div className="panel-title">
                            <BarChart2 size={18} />
                            Leaderboard
                        </div>
                        <Badge variant="primary">{leaderboard.length} traders</Badge>
                    </div>
                    <div className="panel-body">
                        {leaderboard.length === 0 ? (
                            <div className="empty-state">
                                <Users size={32} />
                                <p>No traders registered yet</p>
                            </div>
                        ) : (
                            <div className="leaderboard-list">
                                {leaderboard.map((entry) => {
                                    const maxNetWorth = Math.max(...leaderboard.map(l => l.netWorth));
                                    const percentage = maxNetWorth > 0 ? (entry.netWorth / maxNetWorth) * 100 : 0;
                                    return (
                                        <div key={entry.rank} className="leaderboard-item">
                                            <div className={`rank ${entry.rank <= 3 ? 'top' : ''}`}>
                                                {entry.rank}
                                            </div>
                                            <div className="trader-info">
                                                <span className="trader-name">{entry.name}</span>
                                                <div className="progress-bar">
                                                    <div
                                                        className="progress-fill"
                                                        style={{ width: `${percentage}%` }}
                                                    />
                                                </div>
                                            </div>
                                            <div className="net-worth">
                                                {formatCurrency(entry.netWorth)}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </div>

                {/* Recent Trades Panel */}
                <div className="panel trades-panel">
                    <div className="panel-header">
                        <div className="panel-title">
                            <Activity size={18} />
                            Recent Trades
                        </div>
                        <Badge variant="primary">{trades.length} total</Badge>
                    </div>
                    <div className="panel-body">
                        {trades.length === 0 ? (
                            <div className="empty-state">
                                <Activity size={32} />
                                <p>No trades executed yet</p>
                            </div>
                        ) : (
                            <div className="trades-list">
                                {trades.slice(-20).reverse().map((trade, i) => (
                                    <div key={i} className="trade-item">
                                        <div className="trade-symbol">
                                            <Badge variant="primary">{trade.symbol}</Badge>
                                        </div>
                                        <div className="trade-details">
                                            <span className="qty">{trade.qty} shares</span>
                                            <span className="time">
                                                <Clock size={10} />
                                                {new Date(trade.timestamp).toLocaleTimeString()}
                                            </span>
                                        </div>
                                        <div className="trade-price">
                                            ${trade.price.toFixed(2)}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* System Status Panel */}
                <div className="panel status-panel">
                    <div className="panel-header">
                        <div className="panel-title">
                            <AlertTriangle size={18} />
                            System Status
                        </div>
                    </div>
                    <div className="panel-body">
                        <div className="status-list">
                            <div className="status-item">
                                <span className="status-label">WebSocket</span>
                                <Badge variant={isConnected ? 'success' : 'danger'}>
                                    {isConnected ? 'Connected' : 'Disconnected'}
                                </Badge>
                            </div>
                            <div className="status-item">
                                <span className="status-label">Circuit Breakers</span>
                                <Badge variant={haltedCount > 0 ? 'warning' : 'success'}>
                                    {haltedCount} Active
                                </Badge>
                            </div>
                            <div className="status-item">
                                <span className="status-label">Market</span>
                                <Badge variant={marketOpen ? 'success' : 'danger'}>
                                    {marketOpen ? 'Trading' : 'Closed'}
                                </Badge>
                            </div>
                            <div className="status-item">
                                <span className="status-label">News Feed</span>
                                <Badge variant={news.length > 0 ? 'success' : 'warning'}>
                                    {news.length} items
                                </Badge>
                            </div>
                            <div className="status-item">
                                <span className="status-label">Indices</span>
                                <Badge variant={Object.keys(indices).length > 0 ? 'success' : 'warning'}>
                                    {Object.keys(indices).length} tracked
                                </Badge>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default AdminDashboardPage;
