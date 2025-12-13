// ============================================
// Admin Diagnostics Page
// System health, performance, and debugging info
// ============================================

import React, { useState, useEffect, useCallback } from 'react';
import {
    Activity,
    Wifi,
    Clock,
    Users,
    AlertTriangle,
    CheckCircle,
    XCircle,
    RefreshCw,
    Server,
    TrendingUp,
    MessageSquare,
    ShoppingCart
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { Badge, Button } from '../../components/common';
import websocketService from '../../services/websocket';
import type { AdminDashboardMetrics, ActiveSessionInfo } from '../../types/api';
import { useConfigStore } from '../../store/configStore';

// === System Health Card ===
interface HealthMetric {
    name: string;
    status: 'healthy' | 'warning' | 'critical';
    value: string;
    detail?: string;
    icon: React.ReactNode;
}

const HealthCard: React.FC<{ metric: HealthMetric }> = ({ metric }) => {
    const statusColors = {
        healthy: 'var(--color-success)',
        warning: 'var(--color-warning)',
        critical: 'var(--color-danger)'
    };

    const statusIcons = {
        healthy: <CheckCircle size={16} />,
        warning: <AlertTriangle size={16} />,
        critical: <XCircle size={16} />
    };

    return (
        <div className="stat-card">
            <div className="flex items-center justify-between mb-2">
                <div className="stat-label">
                    {metric.icon}
                    {metric.name}
                </div>
                <div style={{ color: statusColors[metric.status] }}>
                    {statusIcons[metric.status]}
                </div>
            </div>
            <div className="stat-value">{metric.value}</div>
            {metric.detail && (
                <div className="text-xs text-muted mt-1">{metric.detail}</div>
            )}
        </div>
    );
};

// === Connection Monitor ===
const ConnectionMonitor: React.FC<{ serverUptime: number }> = ({ serverUptime }) => {
    const { isConnected } = useGameStore();
    const [connectionHistory, setConnectionHistory] = useState<Array<{time: Date, connected: boolean}>>([]);
    const [latency, setLatency] = useState<number>(0);
    const pingTimeRef = React.useRef<number>(0);

    useEffect(() => {
        // Measure actual WebSocket round-trip latency using ping/pong
        const handlePong = () => {
            if (pingTimeRef.current > 0) {
                const roundTrip = Date.now() - pingTimeRef.current;
                setLatency(roundTrip);
                pingTimeRef.current = 0;
            }
        };

        // Subscribe to Pong messages
        const unsubscribe = websocketService.on('Pong', handlePong);

        // Send periodic pings to measure latency
        const interval = setInterval(() => {
            if (isConnected) {
                pingTimeRef.current = Date.now();
                websocketService.send({ type: 'Ping', payload: {} });
            }
        }, 3000);

        return () => {
            clearInterval(interval);
            unsubscribe();
        };
    }, [isConnected]);

    // Track connection state changes
    const prevConnectedRef = React.useRef<boolean | null>(null);
    useEffect(() => {
        if (prevConnectedRef.current !== null && prevConnectedRef.current !== isConnected) {
            // Connection state changed, update history in a microtask to avoid sync setState
            queueMicrotask(() => {
                setConnectionHistory(prev => [
                    ...prev.slice(-19),
                    { time: new Date(), connected: isConnected }
                ]);
            });
        }
        prevConnectedRef.current = isConnected;
    }, [isConnected]);

    const formatUptime = (seconds: number) => {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }
        return `${minutes}m ${secs}s`;
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Wifi size={18} />
                    WebSocket Connection
                </div>
                <Badge variant={isConnected ? 'success' : 'danger'} pulse>
                    {isConnected ? 'Connected' : 'Disconnected'}
                </Badge>
            </div>
            <div className="panel-body">
                <div className="grid grid-cols-3 gap-4 mb-4">
                    <div className="text-center p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
                        <div className="text-2xl font-bold" style={{ color: latency < 50 ? 'var(--color-success)' : latency < 100 ? 'var(--color-warning)' : 'var(--color-danger)' }}>
                            {latency > 0 ? `${latency}ms` : '--'}
                        </div>
                        <div className="text-xs text-muted">Latency</div>
                    </div>
                    <div className="text-center p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
                        <div className="text-2xl font-bold">
                            {formatUptime(serverUptime)}
                        </div>
                        <div className="text-xs text-muted">Server Uptime</div>
                    </div>
                    <div className="text-center p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
                        <div className="text-2xl font-bold">
                            {connectionHistory.filter(h => !h.connected).length}
                        </div>
                        <div className="text-xs text-muted">Disconnects</div>
                    </div>
                </div>

                {/* Connection Timeline */}
                <div className="mt-4">
                    <div className="text-sm text-muted mb-2">Connection History (last 20)</div>
                    <div className="flex gap-1">
                        {connectionHistory.map((entry, i) => (
                            <div
                                key={i}
                                className="flex-1 h-6 rounded-sm"
                                style={{
                                    background: entry.connected ? 'var(--color-success)' : 'var(--color-danger)',
                                    opacity: 0.3 + (i / connectionHistory.length) * 0.7
                                }}
                                title={`${entry.time.toLocaleTimeString()}: ${entry.connected ? 'Connected' : 'Disconnected'}`}
                            />
                        ))}
                        {Array(20 - connectionHistory.length).fill(0).map((_, i) => (
                            <div
                                key={`empty-${i}`}
                                className="flex-1 h-6 rounded-sm"
                                style={{ background: 'var(--bg-tertiary)' }}
                            />
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
};

// === Message Throughput ===
interface MessageThroughputProps {
    totalTrades: number;
    recentVolume: number;
    openOrdersCount: number;
}

const MessageThroughput: React.FC<MessageThroughputProps> = ({ totalTrades, recentVolume, openOrdersCount }) => {
    const { chatMessages } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [currentTime, setCurrentTime] = useState(() => Date.now());

    // Update current time periodically for chat message filtering
    useEffect(() => {
        const interval = setInterval(() => setCurrentTime(Date.now()), 5000);
        return () => clearInterval(interval);
    }, []);

    // Count chat messages from last 60 seconds
    const recentChat = chatMessages.filter(m => currentTime - m.timestamp < 60000).length;

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Activity size={18} />
                    System Activity
                </div>
                <span className="text-sm text-muted">Real-time metrics</span>
            </div>
            <div className="panel-body">
                <div className="space-y-4">
                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <TrendingUp size={14} />
                                Total Trades
                            </span>
                            <span className="font-mono">{totalTrades.toLocaleString()}</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((totalTrades / 1000) * 100, 100)}%`,
                                    background: 'var(--color-success)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <ShoppingCart size={14} />
                                Open Orders
                            </span>
                            <span className="font-mono">{openOrdersCount}</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((openOrdersCount / 100) * 100, 100)}%`,
                                    background: 'var(--color-primary)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <MessageSquare size={14} />
                                Chat Messages (1min)
                            </span>
                            <span className="font-mono">{recentChat}</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((recentChat / 50) * 100, 100)}%`,
                                    background: 'var(--color-warning)'
                                }}
                            />
                        </div>
                    </div>

                    <div className="pt-3 border-t" style={{ borderColor: 'var(--border-secondary)' }}>
                        <div className="flex justify-between">
                            <span className="font-medium">Recent Volume (5min)</span>
                            <span className="font-mono font-bold">{formatCurrency(recentVolume)}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

// === Circuit Breaker Status ===
const CircuitBreakerStatus: React.FC = () => {
    const { haltedSymbols } = useGameStore();
    const [currentTime, setCurrentTime] = useState(() => Date.now());

    // Update current time every second for countdown display
    useEffect(() => {
        const interval = setInterval(() => setCurrentTime(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    const activeHalts = Object.entries(haltedSymbols).filter(
        ([, until]) => until > currentTime
    );

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <AlertTriangle size={18} />
                    Circuit Breakers
                </div>
                <Badge variant={activeHalts.length > 0 ? 'warning' : 'success'}>
                    {activeHalts.length} Active
                </Badge>
            </div>
            <div className="panel-body">
                {activeHalts.length === 0 ? (
                    <div className="text-center py-6 text-muted">
                        <CheckCircle size={32} className="mx-auto mb-2" style={{ color: 'var(--color-success)' }} />
                        <div>All circuits operating normally</div>
                    </div>
                ) : (
                    <div className="space-y-3">
                        {activeHalts.map(([symbol, until]) => {
                            const remaining = Math.max(0, Math.ceil((until - currentTime) / 1000));
                            return (
                                <div
                                    key={symbol}
                                    className="flex items-center justify-between p-3 rounded"
                                    style={{ background: 'var(--color-danger-bg)' }}
                                >
                                    <div className="flex items-center gap-3">
                                        <Badge variant="danger">{symbol}</Badge>
                                        <span className="text-sm">Trading Halted</span>
                                    </div>
                                    <div className="text-sm font-mono">
                                        {Math.floor(remaining / 60)}:{(remaining % 60).toString().padStart(2, '0')} remaining
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
};

// === Active Sessions (Real Data) ===
interface ActiveSessionsProps {
    sessions: ActiveSessionInfo[];
}

const ActiveSessions: React.FC<ActiveSessionsProps> = ({ sessions }) => {
    const [currentTime, setCurrentTime] = useState(() => Date.now());

    // Update current time periodically for "last activity" display
    useEffect(() => {
        const interval = setInterval(() => setCurrentTime(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Users size={18} />
                    Active Sessions
                </div>
                <Badge variant="primary">{sessions.length} online</Badge>
            </div>
            <div className="panel-body p-0">
                <table className="table w-full">
                    <thead>
                        <tr>
                            <th className="text-left">User</th>
                            <th className="text-right">Connected</th>
                            <th className="text-right">Last Activity</th>
                            <th className="text-right">Session ID</th>
                        </tr>
                    </thead>
                    <tbody>
                        {sessions.map(session => (
                            <tr key={session.session_id}>
                                <td>
                                    <div className="flex items-center gap-2">
                                        <div className="w-2 h-2 rounded-full" style={{ background: 'var(--color-success)' }} />
                                        {session.user_name}
                                    </div>
                                </td>
                                <td className="text-right text-sm text-muted">
                                    {new Date(session.connected_at * 1000).toLocaleTimeString()}
                                </td>
                                <td className="text-right text-sm text-muted">
                                    {Math.floor((currentTime / 1000 - session.last_activity))}s ago
                                </td>
                                <td className="text-right font-mono text-xs text-muted">
                                    #{session.session_id}
                                </td>
                            </tr>
                        ))}
                        {sessions.length === 0 && (
                            <tr>
                                <td colSpan={4} className="text-center py-6 text-muted">
                                    No active sessions
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

// === Server Info ===
interface ServerInfoProps {
    serverUptime: number;
    totalTraders: number;
    activeTraders: number;
    marketOpen: boolean;
}

const ServerInfo: React.FC<ServerInfoProps> = ({ serverUptime, totalTraders, activeTraders, marketOpen }) => {
    const formatUptime = (seconds: number) => {
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        if (days > 0) {
            return `${days}d ${hours}h ${minutes}m`;
        }
        if (hours > 0) {
            return `${hours}h ${minutes}m`;
        }
        return `${minutes}m`;
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Server size={18} />
                    Server Status
                </div>
                <Badge variant={marketOpen ? 'success' : 'warning'}>
                    {marketOpen ? 'Market Open' : 'Market Closed'}
                </Badge>
            </div>
            <div className="panel-body">
                <div className="space-y-4">
                    <div className="flex justify-between items-center">
                        <span className="flex items-center gap-2 text-sm">
                            <Clock size={14} />
                            Server Uptime
                        </span>
                        <span className="font-mono font-bold">{formatUptime(serverUptime)}</span>
                    </div>

                    <div className="flex justify-between items-center">
                        <span className="flex items-center gap-2 text-sm">
                            <Users size={14} />
                            Registered Traders
                        </span>
                        <span className="font-mono font-bold">{totalTraders}</span>
                    </div>

                    <div className="flex justify-between items-center">
                        <span className="flex items-center gap-2 text-sm">
                            <Activity size={14} />
                            Active Connections
                        </span>
                        <span className="font-mono font-bold">{activeTraders}</span>
                    </div>

                    <div className="pt-3 border-t" style={{ borderColor: 'var(--border-secondary)' }}>
                        <div className="flex justify-between items-center">
                            <span className="font-medium">Connection Rate</span>
                            <span className="font-mono">
                                {totalTraders > 0 ? Math.round((activeTraders / totalTraders) * 100) : 0}%
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

// === Main Page ===
export const DiagnosticsPage: React.FC = () => {
    const { isConnected, marketOpen, leaderboard, trades } = useGameStore();
    const [metrics, setMetrics] = useState<AdminDashboardMetrics | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const fetchMetrics = useCallback(() => {
        setIsLoading(true);
        websocketService.send({
            type: 'AdminAction',
            payload: { action: 'GetDashboardMetrics', payload: {} }
        });
    }, []);

    // Fetch metrics on mount and periodically
    useEffect(() => {
        // Use queueMicrotask to avoid synchronous setState in effect body
        queueMicrotask(() => fetchMetrics());

        const interval = setInterval(fetchMetrics, 10000); // Refresh every 10 seconds

        return () => clearInterval(interval);
    }, [fetchMetrics]);

    // Listen for metrics response
    useEffect(() => {
        const unsubscribe = websocketService.on('AdminDashboardMetrics', (payload: { metrics: AdminDashboardMetrics }) => {
            setMetrics(payload.metrics);
            setIsLoading(false);
        });

        return unsubscribe;
    }, []);

    // Health metrics
    const healthMetrics: HealthMetric[] = [
        {
            name: 'WebSocket',
            status: isConnected ? 'healthy' : 'critical',
            value: isConnected ? 'Connected' : 'Disconnected',
            icon: <Wifi size={14} />
        },
        {
            name: 'Market Status',
            status: marketOpen ? 'healthy' : 'warning',
            value: marketOpen ? 'Open' : 'Closed',
            icon: <Activity size={14} />
        },
        {
            name: 'Active Traders',
            status: (metrics?.active_traders ?? leaderboard.length) > 0 ? 'healthy' : 'warning',
            value: (metrics?.active_traders ?? leaderboard.length).toString(),
            detail: 'Currently connected',
            icon: <Users size={14} />
        },
        {
            name: 'Trade Volume',
            status: 'healthy',
            value: (metrics?.total_trades ?? trades.length).toString(),
            detail: 'Total executions',
            icon: <TrendingUp size={14} />
        }
    ];

    return (
        <div className="diagnostics-page">
            {/* Header */}
            <div className="mb-6 flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold flex items-center gap-2">
                        <Activity size={24} />
                        System Diagnostics
                    </h1>
                    <p className="text-muted mt-1">Monitor system health, performance, and connectivity</p>
                </div>
                <Button variant="secondary" onClick={fetchMetrics} disabled={isLoading}>
                    <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
                    Refresh
                </Button>
            </div>

            {/* Health Overview */}
            <div className="stats-row grid grid-cols-4 gap-4 mb-6">
                {healthMetrics.map(metric => (
                    <HealthCard key={metric.name} metric={metric} />
                ))}
            </div>

            {/* Main Grid */}
            <div className="grid grid-cols-2 gap-6">
                {/* Left Column */}
                <div className="space-y-6">
                    <ConnectionMonitor serverUptime={metrics?.server_uptime_secs ?? 0} />
                    <CircuitBreakerStatus />
                </div>

                {/* Right Column */}
                <div className="space-y-6">
                    <MessageThroughput
                        totalTrades={metrics?.total_trades ?? 0}
                        recentVolume={metrics?.recent_volume ?? 0}
                        openOrdersCount={metrics?.open_orders_count ?? 0}
                    />
                    <ServerInfo
                        serverUptime={metrics?.server_uptime_secs ?? 0}
                        totalTraders={metrics?.total_traders ?? 0}
                        activeTraders={metrics?.active_traders ?? 0}
                        marketOpen={metrics?.market_open ?? marketOpen}
                    />
                </div>
            </div>

            {/* Active Sessions - Full Width */}
            <div className="mt-6">
                <ActiveSessions sessions={metrics?.active_sessions ?? []} />
            </div>
        </div>
    );
};

export default DiagnosticsPage;
