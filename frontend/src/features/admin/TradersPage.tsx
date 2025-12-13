// ============================================
// Admin Traders Management Page
// ============================================

import React, { useState, useMemo } from 'react';
import {
    Users,
    Search,
    Ban,
    CheckCircle,
    MessageCircleOff,
    DollarSign,
    TrendingUp,
    TrendingDown,
    ChevronUp,
    ChevronDown
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import websocketService from '../../services/websocket';
import { Button, Badge, Modal } from '../../components/common';

// Sort icon component - defined outside to avoid recreation during render
interface TraderSortIconProps {
    field: 'rank' | 'name' | 'netWorth';
    sortField: 'rank' | 'name' | 'netWorth';
    sortOrder: 'asc' | 'desc';
}

const TraderSortIcon: React.FC<TraderSortIconProps> = ({ field, sortField, sortOrder }) => {
    if (sortField !== field) return null;
    return sortOrder === 'asc' ? <ChevronUp size={14} /> : <ChevronDown size={14} />;
};

interface TraderRowProps {
    entry: {
        rank: number;
        name: string;
        netWorth: number;
    };
    avgNetWorth: number;
    isMuted: boolean;
    isBanned: boolean;
    onBan: (name: string) => void;
    onUnban: (name: string) => void;
    onToggleChat: (name: string, enabled: boolean) => void;
}

const TraderRow: React.FC<TraderRowProps> = ({ entry, avgNetWorth, isMuted, isBanned, onBan, onUnban, onToggleChat }) => {
    const [showActions, setShowActions] = useState(false);

    // Calculate performance relative to average
    const performancePercent = avgNetWorth > 0
        ? ((entry.netWorth - avgNetWorth) / avgNetWorth) * 100
        : 0;

    return (
        <tr
            onMouseEnter={() => setShowActions(true)}
            onMouseLeave={() => setShowActions(false)}
            style={{ opacity: isBanned ? 0.5 : 1 }}
        >
            <td>
                <div className="flex items-center gap-2">
                    <span
                        className="font-bold"
                        style={{
                            width: '28px',
                            height: '28px',
                            borderRadius: '50%',
                            background: entry.rank <= 3 ? 'var(--color-warning)' : 'var(--bg-tertiary)',
                            color: entry.rank <= 3 ? 'black' : 'var(--text-secondary)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: 'var(--text-xs)'
                        }}
                    >
                        {entry.rank}
                    </span>
                </div>
            </td>
            <td>
                <div className="flex items-center gap-2">
                    <div
                        className="font-bold"
                        style={{
                            width: '32px',
                            height: '32px',
                            borderRadius: '50%',
                            background: isBanned
                                ? 'var(--color-danger)'
                                : 'linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-dark) 100%)',
                            color: 'white',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            fontSize: 'var(--text-sm)'
                        }}
                    >
                        {isBanned ? <Ban size={14} /> : entry.name.charAt(0).toUpperCase()}
                    </div>
                    <div>
                        <span className="font-medium">{entry.name}</span>
                        {isMuted && (
                            <div className="flex items-center gap-1 text-xs text-warning">
                                <MessageCircleOff size={10} />
                                Muted
                            </div>
                        )}
                    </div>
                </div>
            </td>
            <td>
                <span className="font-mono font-bold text-success">
                    ${entry.netWorth.toLocaleString()}
                </span>
            </td>
            <td>
                <div className={`flex items-center gap-1 ${performancePercent >= 0 ? 'text-buy' : 'text-sell'}`}>
                    {performancePercent >= 0 ? <TrendingUp size={14} /> : <TrendingDown size={14} />}
                    <span className="font-mono">{performancePercent >= 0 ? '+' : ''}{performancePercent.toFixed(1)}%</span>
                </div>
            </td>
            <td>
                {isBanned ? (
                    <Badge variant="danger">Banned</Badge>
                ) : (
                    <Badge variant="success">Active</Badge>
                )}
            </td>
            <td>
                <div className={`flex gap-2 transition-opacity ${showActions ? 'opacity-100' : 'opacity-0'}`}>
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onToggleChat(entry.name, isMuted)}
                        title={isMuted ? 'Unmute Chat' : 'Mute Chat'}
                        style={{ color: isMuted ? 'var(--color-warning)' : undefined }}
                    >
                        <MessageCircleOff size={14} />
                    </Button>
                    {isBanned ? (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onUnban(entry.name)}
                            title="Unban Trader"
                            style={{ color: 'var(--color-success)' }}
                        >
                            <CheckCircle size={14} />
                        </Button>
                    ) : (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onBan(entry.name)}
                            title="Ban Trader"
                            style={{ color: 'var(--color-danger)' }}
                        >
                            <Ban size={14} />
                        </Button>
                    )}
                </div>
            </td>
        </tr>
    );
};

export const TradersPage: React.FC = () => {
    const { leaderboard } = useGameStore();
    const [searchQuery, setSearchQuery] = useState('');
    const [sortField, setSortField] = useState<'rank' | 'name' | 'netWorth'>('rank');
    const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
    const [showBanModal, setShowBanModal] = useState<string | null>(null);

    // Track banned and muted users locally (would come from backend in production)
    const [bannedUsers, setBannedUsers] = useState<Set<string>>(new Set());
    const [mutedUsers, setMutedUsers] = useState<Set<string>>(new Set());

    // Calculate average net worth for performance comparison
    const avgNetWorth = useMemo(() => {
        if (leaderboard.length === 0) return 0;
        return leaderboard.reduce((sum, t) => sum + t.netWorth, 0) / leaderboard.length;
    }, [leaderboard]);

    // Filter and sort traders
    const filteredTraders = useMemo(() => {
        let traders = [...leaderboard];

        // Filter
        if (searchQuery) {
            traders = traders.filter(t =>
                t.name.toLowerCase().includes(searchQuery.toLowerCase())
            );
        }

        // Sort
        traders.sort((a, b) => {
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

        return traders;
    }, [leaderboard, searchQuery, sortField, sortOrder]);

    const handleSort = (field: 'rank' | 'name' | 'netWorth') => {
        if (sortField === field) {
            setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
        } else {
            setSortField(field);
            setSortOrder('asc');
        }
    };

    const handleBan = (name: string) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'BanTrader',
                payload: { name }
            }
        });
        setBannedUsers(prev => new Set(prev).add(name));
        setShowBanModal(null);
    };

    const handleUnban = (name: string) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'UnbanTrader',
                payload: { name }
            }
        });
        setBannedUsers(prev => {
            const newSet = new Set(prev);
            newSet.delete(name);
            return newSet;
        });
    };

    const handleToggleChat = (name: string, currentlyMuted: boolean) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: currentlyMuted ? 'UnmuteTrader' : 'MuteTrader',
                payload: { name }
            }
        });
        setMutedUsers(prev => {
            const newSet = new Set(prev);
            if (currentlyMuted) {
                newSet.delete(name);
            } else {
                newSet.add(name);
            }
            return newSet;
        });
    };

    return (
        <div className="traders-page">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold mb-1">Trader Management</h1>
                    <p className="text-muted">
                        {leaderboard.length} registered traders
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
                            placeholder="Search traders..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            style={{ paddingLeft: '40px', width: '250px' }}
                        />
                    </div>
                </div>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-4 gap-4 mb-6">
                <div className="stat-card">
                    <div className="stat-label">
                        <Users size={14} />
                        Total Traders
                    </div>
                    <div className="stat-value">{leaderboard.length}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <CheckCircle size={14} />
                        Active
                    </div>
                    <div className="stat-value text-success">{leaderboard.length}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <Ban size={14} />
                        Banned
                    </div>
                    <div className="stat-value text-danger">{bannedUsers.size}</div>
                </div>
                <div className="stat-card">
                    <div className="stat-label">
                        <DollarSign size={14} />
                        Total Net Worth
                    </div>
                    <div className="stat-value text-sm">
                        ${leaderboard.reduce((sum, t) => sum + t.netWorth, 0).toLocaleString()}
                    </div>
                </div>
            </div>

            {/* Table */}
            <div className="panel">
                <div className="table-container">
                    <table className="table">
                        <thead>
                            <tr>
                                <th
                                    style={{ width: '80px', cursor: 'pointer' }}
                                    onClick={() => handleSort('rank')}
                                >
                                    <div className="flex items-center gap-1">
                                        Rank
                                        <TraderSortIcon field="rank" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th
                                    style={{ cursor: 'pointer' }}
                                    onClick={() => handleSort('name')}
                                >
                                    <div className="flex items-center gap-1">
                                        Trader
                                        <TraderSortIcon field="name" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th
                                    style={{ cursor: 'pointer' }}
                                    onClick={() => handleSort('netWorth')}
                                >
                                    <div className="flex items-center gap-1">
                                        Net Worth
                                        <TraderSortIcon field="netWorth" sortField={sortField} sortOrder={sortOrder} />
                                    </div>
                                </th>
                                <th>Change</th>
                                <th>Status</th>
                                <th style={{ width: '120px' }}>Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filteredTraders.length === 0 ? (
                                <tr>
                                    <td colSpan={6} className="text-center text-muted py-8">
                                        {searchQuery ? 'No traders found matching your search' : 'No traders registered yet'}
                                    </td>
                                </tr>
                            ) : (
                                filteredTraders.map((entry) => (
                                    <TraderRow
                                        key={entry.rank}
                                        entry={entry}
                                        avgNetWorth={avgNetWorth}
                                        isMuted={mutedUsers.has(entry.name)}
                                        isBanned={bannedUsers.has(entry.name)}
                                        onBan={(name) => setShowBanModal(name)}
                                        onUnban={handleUnban}
                                        onToggleChat={handleToggleChat}
                                    />
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Ban Confirmation Modal */}
            {showBanModal && (
                <Modal
                    title="Ban Trader"
                    onClose={() => setShowBanModal(null)}
                    isOpen={true}
                >
                    <div className="text-center">
                        <Ban size={48} style={{ color: 'var(--color-danger)', margin: '0 auto 16px' }} />
                        <p className="mb-6">
                            Are you sure you want to ban <strong>{showBanModal}</strong>?
                            <br />
                            <span className="text-muted text-sm">They will be logged out immediately.</span>
                        </p>
                        <div className="flex gap-3 justify-center">
                            <Button variant="secondary" onClick={() => setShowBanModal(null)}>
                                Cancel
                            </Button>
                            <Button variant="danger" onClick={() => handleBan(showBanModal)}>
                                <Ban size={16} />
                                Ban Trader
                            </Button>
                        </div>
                    </div>
                </Modal>
            )}
        </div>
    );
};

export default TradersPage;
