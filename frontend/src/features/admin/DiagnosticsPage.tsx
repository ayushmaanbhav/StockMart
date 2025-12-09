// ============================================
// Admin Diagnostics Page
// System health, performance, and debugging info
// ============================================

import React, { useState, useEffect } from 'react';
import {
    Activity,
    Cpu,
    HardDrive,
    Wifi,
    Clock,
    Users,
    AlertTriangle,
    CheckCircle,
    XCircle,
    RefreshCw,
    Server,
    Database,
    Zap,
    TrendingUp,
    MessageSquare,
    ShoppingCart
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import { Badge, Button } from '../../components/common';

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
const ConnectionMonitor: React.FC = () => {
    const { isConnected } = useGameStore();
    const [connectionHistory, setConnectionHistory] = useState<Array<{time: Date, connected: boolean}>>([]);
    const [latency, setLatency] = useState<number>(0);

    useEffect(() => {
        // Simulate latency measurement
        const interval = setInterval(() => {
            setLatency(Math.floor(Math.random() * 50) + 10); // 10-60ms simulated
        }, 2000);

        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        setConnectionHistory(prev => [
            ...prev.slice(-19),
            { time: new Date(), connected: isConnected }
        ]);
    }, [isConnected]);

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
                        <div className="text-2xl font-bold" style={{ color: latency < 50 ? 'var(--color-success)' : 'var(--color-warning)' }}>
                            {latency}ms
                        </div>
                        <div className="text-xs text-muted">Latency</div>
                    </div>
                    <div className="text-center p-3 rounded" style={{ background: 'var(--bg-tertiary)' }}>
                        <div className="text-2xl font-bold">
                            {connectionHistory.filter(h => h.connected).length}
                        </div>
                        <div className="text-xs text-muted">Uptime Events</div>
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
const MessageThroughput: React.FC = () => {
    const { trades, chatMessages } = useGameStore();
    const [messageRates, setMessageRates] = useState({
        trades: 0,
        chat: 0,
        orders: 0,
        total: 0
    });

    useEffect(() => {
        // Calculate message rates based on recent activity
        const now = Date.now();
        const recentTrades = trades.filter(t => now - t.timestamp < 60000).length;
        const recentChat = chatMessages.filter(m => now - m.timestamp < 60000).length;

        setMessageRates({
            trades: recentTrades,
            chat: recentChat,
            orders: Math.floor(recentTrades * 1.5), // Estimate
            total: recentTrades + recentChat + Math.floor(recentTrades * 1.5)
        });
    }, [trades, chatMessages]);

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Zap size={18} />
                    Message Throughput
                </div>
                <span className="text-sm text-muted">Last 60 seconds</span>
            </div>
            <div className="panel-body">
                <div className="space-y-4">
                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <TrendingUp size={14} />
                                Trade Updates
                            </span>
                            <span className="font-mono">{messageRates.trades}/min</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((messageRates.trades / 100) * 100, 100)}%`,
                                    background: 'var(--color-success)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <ShoppingCart size={14} />
                                Order Messages
                            </span>
                            <span className="font-mono">{messageRates.orders}/min</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((messageRates.orders / 100) * 100, 100)}%`,
                                    background: 'var(--color-primary)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <MessageSquare size={14} />
                                Chat Messages
                            </span>
                            <span className="font-mono">{messageRates.chat}/min</span>
                        </div>
                        <div className="h-2 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${Math.min((messageRates.chat / 50) * 100, 100)}%`,
                                    background: 'var(--color-warning)'
                                }}
                            />
                        </div>
                    </div>

                    <div className="pt-3 border-t" style={{ borderColor: 'var(--border-secondary)' }}>
                        <div className="flex justify-between">
                            <span className="font-medium">Total Throughput</span>
                            <span className="font-mono font-bold">{messageRates.total} msg/min</span>
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
    const activeHalts = Object.entries(haltedSymbols).filter(
        ([, until]) => until > Date.now()
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
                            const remaining = Math.max(0, Math.ceil((until - Date.now()) / 1000));
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

// === Active Sessions ===
const ActiveSessions: React.FC = () => {
    const { leaderboard } = useGameStore();

    // Simulate session data based on leaderboard
    const sessions = leaderboard.slice(0, 10).map((entry) => ({
        userId: entry.rank,
        name: entry.name,
        connectedAt: new Date(Date.now() - Math.random() * 3600000),
        lastActivity: new Date(Date.now() - Math.random() * 60000),
        messagesCount: Math.floor(Math.random() * 100)
    }));

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
                            <th className="text-right">Messages</th>
                        </tr>
                    </thead>
                    <tbody>
                        {sessions.map(session => (
                            <tr key={session.userId}>
                                <td>
                                    <div className="flex items-center gap-2">
                                        <div className="w-2 h-2 rounded-full" style={{ background: 'var(--color-success)' }} />
                                        {session.name}
                                    </div>
                                </td>
                                <td className="text-right text-sm text-muted">
                                    {session.connectedAt.toLocaleTimeString()}
                                </td>
                                <td className="text-right text-sm text-muted">
                                    {Math.floor((Date.now() - session.lastActivity.getTime()) / 1000)}s ago
                                </td>
                                <td className="text-right font-mono">
                                    {session.messagesCount}
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

// === Server Metrics (Simulated) ===
const ServerMetrics: React.FC = () => {
    const [metrics, setMetrics] = useState({
        cpuUsage: 0,
        memoryUsage: 0,
        diskUsage: 0,
        uptime: 0
    });

    useEffect(() => {
        // Simulate server metrics
        const interval = setInterval(() => {
            setMetrics({
                cpuUsage: Math.floor(Math.random() * 30) + 10,
                memoryUsage: Math.floor(Math.random() * 20) + 40,
                diskUsage: Math.floor(Math.random() * 5) + 25,
                uptime: Math.floor((Date.now() % 86400000) / 1000) // Simulate uptime
            });
        }, 3000);

        return () => clearInterval(interval);
    }, []);

    const formatUptime = (seconds: number) => {
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        return `${hours}h ${minutes}m`;
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Server size={18} />
                    Server Metrics
                </div>
                <Button variant="ghost" size="sm">
                    <RefreshCw size={14} />
                </Button>
            </div>
            <div className="panel-body">
                <div className="space-y-4">
                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <Cpu size={14} />
                                CPU Usage
                            </span>
                            <span className="font-mono">{metrics.cpuUsage}%</span>
                        </div>
                        <div className="h-3 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${metrics.cpuUsage}%`,
                                    background: metrics.cpuUsage > 80 ? 'var(--color-danger)' :
                                        metrics.cpuUsage > 60 ? 'var(--color-warning)' : 'var(--color-success)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <Database size={14} />
                                Memory Usage
                            </span>
                            <span className="font-mono">{metrics.memoryUsage}%</span>
                        </div>
                        <div className="h-3 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${metrics.memoryUsage}%`,
                                    background: metrics.memoryUsage > 80 ? 'var(--color-danger)' :
                                        metrics.memoryUsage > 60 ? 'var(--color-warning)' : 'var(--color-primary)'
                                }}
                            />
                        </div>
                    </div>

                    <div>
                        <div className="flex justify-between text-sm mb-1">
                            <span className="flex items-center gap-2">
                                <HardDrive size={14} />
                                Disk Usage
                            </span>
                            <span className="font-mono">{metrics.diskUsage}%</span>
                        </div>
                        <div className="h-3 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                            <div
                                className="h-full rounded-full transition-all"
                                style={{
                                    width: `${metrics.diskUsage}%`,
                                    background: 'var(--color-primary)'
                                }}
                            />
                        </div>
                    </div>

                    <div className="pt-3 border-t flex justify-between items-center" style={{ borderColor: 'var(--border-secondary)' }}>
                        <span className="flex items-center gap-2 text-sm">
                            <Clock size={14} />
                            Uptime
                        </span>
                        <span className="font-mono font-bold">{formatUptime(metrics.uptime)}</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

// === Main Page ===
export const DiagnosticsPage: React.FC = () => {
    const { isConnected, marketOpen, leaderboard, trades } = useGameStore();

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
            status: leaderboard.length > 0 ? 'healthy' : 'warning',
            value: leaderboard.length.toString(),
            detail: 'Currently connected',
            icon: <Users size={14} />
        },
        {
            name: 'Trade Volume',
            status: 'healthy',
            value: trades.length.toString(),
            detail: 'Total executions',
            icon: <TrendingUp size={14} />
        }
    ];

    return (
        <div className="diagnostics-page">
            {/* Header */}
            <div className="mb-6">
                <h1 className="text-2xl font-bold flex items-center gap-2">
                    <Activity size={24} />
                    System Diagnostics
                </h1>
                <p className="text-muted mt-1">Monitor system health, performance, and connectivity</p>
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
                    <ConnectionMonitor />
                    <CircuitBreakerStatus />
                </div>

                {/* Right Column */}
                <div className="space-y-6">
                    <MessageThroughput />
                    <ServerMetrics />
                </div>
            </div>

            {/* Active Sessions - Full Width */}
            <div className="mt-6">
                <ActiveSessions />
            </div>
        </div>
    );
};

export default DiagnosticsPage;
