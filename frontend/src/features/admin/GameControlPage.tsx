// ============================================
// Admin Game Control Page
// ============================================

import React, { useState } from 'react';
import {
    Play,
    Pause,
    Power,
    RefreshCw,
    Users,
    Building2,
    AlertTriangle,
    Clock,
    Zap,
    DollarSign,
    Settings,
    Check,
    X
} from 'lucide-react';
import { useGameStore } from '../../store/gameStore';
import websocketService from '../../services/websocket';
import { Button, Badge, Modal } from '../../components/common';
import { GAME_DEFAULTS } from '../../constants';
import { loggers } from '../../utils';

const log = loggers.admin;

// === Game Lifecycle Section ===
const GameLifecycleSection: React.FC = () => {
    const { marketOpen, leaderboard, companies } = useGameStore();
    const [isLoading, setIsLoading] = useState<string | null>(null);
    const [showInitModal, setShowInitModal] = useState(false);

    // Init game configuration
    // Target net worth - traders will get ~half in cash and ~half in shares
    const [targetNetworth, setTargetNetworth] = useState(String(GAME_DEFAULTS.TARGET_NETWORTH));
    const [sharesPerTrader, setSharesPerTrader] = useState(String(GAME_DEFAULTS.SHARES_PER_TRADER));

    const handleAction = async (action: string, payload: Record<string, unknown> = {}) => {
        setIsLoading(action);
        setShowInitModal(false);

        log.debug('Sending action:', action, payload);
        websocketService.send({
            type: 'AdminAction',
            payload: { action, payload }
        });

        // Reset loading after a short delay
        setTimeout(() => setIsLoading(null), 1500);
    };

    const handleInitGame = () => {
        // Pass target_networth - backend will allocate ~half as cash and ~half as shares
        // The "starting_cash" param name is kept for backward compat but represents target networth
        handleAction('InitGame', {
            starting_cash: parseFloat(targetNetworth),
            shares_per_trader: parseInt(sharesPerTrader)
        });
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Zap size={18} />
                    Game Lifecycle
                </div>
            </div>
            <div className="panel-body">
                <div className="grid grid-cols-2 gap-4">
                    {/* Market Toggle */}
                    <div className="stat-card">
                        <div className="flex items-center justify-between mb-3">
                            <div className="stat-label">
                                <Power size={14} />
                                Market Status
                            </div>
                            <Badge variant={marketOpen ? 'success' : 'danger'} pulse>
                                {marketOpen ? 'OPEN' : 'CLOSED'}
                            </Badge>
                        </div>
                        <Button
                            variant={marketOpen ? 'danger' : 'success'}
                            className="w-full"
                            onClick={() => handleAction('ToggleMarket', { open: !marketOpen })}
                            loading={isLoading === 'ToggleMarket'}
                        >
                            {marketOpen ? <Pause size={16} /> : <Play size={16} />}
                            {marketOpen ? 'Close Market' : 'Open Market'}
                        </Button>
                    </div>

                    {/* Initialize Game */}
                    <div className="stat-card">
                        <div className="stat-label mb-3">
                            <RefreshCw size={14} />
                            Game Reset
                        </div>
                        <Button
                            variant="secondary"
                            className="w-full"
                            onClick={() => setShowInitModal(true)}
                            loading={isLoading === 'InitGame'}
                        >
                            <RefreshCw size={16} />
                            Initialize Game
                        </Button>
                    </div>
                </div>

                {/* Current Status */}
                <div className="mt-4 p-3 rounded-lg" style={{ background: 'var(--bg-tertiary)' }}>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span className="text-muted">Registered Traders:</span>
                            <span className="ml-2 font-bold">{leaderboard.length}</span>
                        </div>
                        <div>
                            <span className="text-muted">Companies:</span>
                            <span className="ml-2 font-bold">{companies.length}</span>
                        </div>
                    </div>
                </div>

                {/* Warning about game init */}
                <div className="mt-4 p-3 rounded-lg" style={{
                    background: 'var(--color-warning-bg)',
                    border: '1px solid var(--color-warning)'
                }}>
                    <div className="flex items-start gap-2">
                        <AlertTriangle size={16} style={{ color: 'var(--color-warning)', flexShrink: 0, marginTop: 2 }} />
                        <div className="text-sm">
                            <strong>Initialize Game</strong> will reset all trader portfolios with configurable starting cash and share allocation. This action cannot be undone.
                        </div>
                    </div>
                </div>
            </div>

            {/* Init Config Modal */}
            {showInitModal && (
                <Modal
                    title="Initialize Game"
                    onClose={() => setShowInitModal(false)}
                    isOpen={true}
                >
                    <div className="space-y-4">
                        <p className="text-muted text-sm">
                            Configure initial settings for all traders. Each trader will start with the <strong>same net worth</strong>.
                            The net worth is split between cash (~50%) and randomly allocated shares (~50%).
                        </p>

                        <div className="input-group">
                            <label className="input-label">
                                <DollarSign size={14} />
                                Target Net Worth per Trader
                            </label>
                            <input
                                type="number"
                                className="input"
                                value={targetNetworth}
                                onChange={(e) => setTargetNetworth(e.target.value)}
                                min="10000"
                                step="10000"
                            />
                            <span className="text-xs text-muted mt-1">
                                Each trader will have this total net worth (cash + shares value)
                            </span>
                        </div>

                        <div className="input-group">
                            <label className="input-label">
                                <Building2 size={14} />
                                Base Shares per Company
                            </label>
                            <input
                                type="number"
                                className="input"
                                value={sharesPerTrader}
                                onChange={(e) => setSharesPerTrader(e.target.value)}
                                min="10"
                                step="10"
                            />
                            <span className="text-xs text-muted mt-1">
                                ~{sharesPerTrader} shares per company (±20% random variance).
                                Cash is auto-adjusted to maintain equal net worth.
                            </span>
                        </div>

                        <div className="p-3 rounded-lg" style={{
                            background: 'var(--bg-tertiary)'
                        }}>
                            <div className="text-sm">
                                <strong>Preview:</strong> Each trader gets ~${parseInt(targetNetworth)/2} cash +
                                shares worth ~${parseInt(targetNetworth)/2} ({companies.length} companies × ~{sharesPerTrader} shares @ $100/share)
                            </div>
                        </div>

                        <div className="p-3 rounded-lg" style={{
                            background: 'var(--color-warning-bg)',
                            border: '1px solid var(--color-warning)'
                        }}>
                            <div className="flex items-start gap-2">
                                <AlertTriangle size={16} style={{ color: 'var(--color-warning)', flexShrink: 0, marginTop: 2 }} />
                                <div className="text-sm">
                                    This will reset all existing trader data! Market will be closed after initialization.
                                </div>
                            </div>
                        </div>

                        <div className="flex gap-3 justify-end mt-6">
                            <Button variant="secondary" onClick={() => setShowInitModal(false)}>
                                Cancel
                            </Button>
                            <Button variant="danger" onClick={handleInitGame}>
                                <RefreshCw size={16} />
                                Initialize Game
                            </Button>
                        </div>
                    </div>
                </Modal>
            )}
        </div>
    );
};

// === Trading Hours Section ===
const TradingHoursSection: React.FC = () => {
    const [startTime, setStartTime] = useState<string>(GAME_DEFAULTS.TRADING_START_TIME);
    const [endTime, setEndTime] = useState<string>(GAME_DEFAULTS.TRADING_END_TIME);
    const [isSaving, setIsSaving] = useState(false);

    const handleSave = () => {
        setIsSaving(true);
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'SetTradingHours',
                payload: { start: startTime, end: endTime }
            }
        });
        setTimeout(() => setIsSaving(false), 1000);
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Clock size={18} />
                    Trading Hours
                </div>
            </div>
            <div className="panel-body">
                <div className="grid grid-cols-2 gap-4 mb-4">
                    <div className="input-group">
                        <label className="input-label">Market Open</label>
                        <input
                            type="time"
                            className="input"
                            value={startTime}
                            onChange={(e) => setStartTime(e.target.value)}
                        />
                    </div>
                    <div className="input-group">
                        <label className="input-label">Market Close</label>
                        <input
                            type="time"
                            className="input"
                            value={endTime}
                            onChange={(e) => setEndTime(e.target.value)}
                        />
                    </div>
                </div>
                <Button
                    variant="primary"
                    className="w-full"
                    onClick={handleSave}
                    loading={isSaving}
                >
                    <Check size={16} />
                    Save Trading Hours
                </Button>
            </div>
        </div>
    );
};

// === Circuit Breaker Section ===
const CircuitBreakerSection: React.FC = () => {
    const { haltedSymbols } = useGameStore();
    const [threshold, setThreshold] = useState(String(GAME_DEFAULTS.CIRCUIT_BREAKER_THRESHOLD));
    const [duration, setDuration] = useState(String(GAME_DEFAULTS.CIRCUIT_BREAKER_DURATION));
    const [isSaving, setIsSaving] = useState(false);
    const [currentTime, setCurrentTime] = useState(() => Date.now());

    // Update current time periodically for active halts calculation
    React.useEffect(() => {
        const interval = setInterval(() => setCurrentTime(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    const activeHalts = Object.entries(haltedSymbols).filter(
        ([, until]) => until > currentTime
    );

    const handleSave = () => {
        setIsSaving(true);
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'SetCircuitBreaker',
                payload: {
                    threshold_percent: parseFloat(threshold),
                    halt_duration_secs: parseInt(duration)
                }
            }
        });
        setTimeout(() => setIsSaving(false), 1000);
    };

    const handleLiftHalt = (symbol: string) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'LiftCircuitBreaker',
                payload: { symbol }
            }
        });
    };

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <AlertTriangle size={18} />
                    Circuit Breakers
                </div>
                {activeHalts.length > 0 && (
                    <Badge variant="warning" pulse>
                        {activeHalts.length} Active
                    </Badge>
                )}
            </div>
            <div className="panel-body">
                {/* Active Halts */}
                {activeHalts.length > 0 && (
                    <div className="mb-4">
                        <h4 className="text-sm font-semibold text-muted mb-2">Active Halts</h4>
                        <div className="space-y-2">
                            {activeHalts.map(([symbol, until]) => (
                                <div
                                    key={symbol}
                                    className="flex items-center justify-between p-2 rounded"
                                    style={{ background: 'var(--color-danger-bg)' }}
                                >
                                    <div className="flex items-center gap-2">
                                        <Badge variant="danger">{symbol}</Badge>
                                        <span className="text-sm text-muted">
                                            Until {new Date(until).toLocaleTimeString()}
                                        </span>
                                    </div>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => handleLiftHalt(symbol)}
                                    >
                                        <X size={14} />
                                        Lift
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {/* Settings */}
                <div className="grid grid-cols-2 gap-4 mb-4">
                    <div className="input-group">
                        <label className="input-label">Threshold (%)</label>
                        <input
                            type="number"
                            className="input"
                            value={threshold}
                            onChange={(e) => setThreshold(e.target.value)}
                            min="1"
                            max="50"
                        />
                    </div>
                    <div className="input-group">
                        <label className="input-label">Duration (seconds)</label>
                        <input
                            type="number"
                            className="input"
                            value={duration}
                            onChange={(e) => setDuration(e.target.value)}
                            min="60"
                            max="3600"
                        />
                    </div>
                </div>
                <Button
                    variant="secondary"
                    className="w-full"
                    onClick={handleSave}
                    loading={isSaving}
                >
                    <Settings size={16} />
                    Update Settings
                </Button>
            </div>
        </div>
    );
};

// === Quick Stats ===
const QuickStatsSection: React.FC = () => {
    const { leaderboard, trades, indices } = useGameStore();

    const totalTraders = leaderboard.length;
    const totalTrades = trades.length;
    const totalVolume = trades.reduce((sum, t) => sum + (t.price * t.qty), 0);
    const indexCount = Object.keys(indices).length;

    return (
        <div className="panel">
            <div className="panel-header">
                <div className="panel-title">
                    <Building2 size={18} />
                    Quick Stats
                </div>
            </div>
            <div className="panel-body">
                <div className="grid grid-cols-2 gap-4">
                    <div className="stat-card">
                        <div className="stat-label">
                            <Users size={14} />
                            Traders
                        </div>
                        <div className="stat-value">{totalTraders}</div>
                    </div>
                    <div className="stat-card">
                        <div className="stat-label">
                            <Zap size={14} />
                            Trades
                        </div>
                        <div className="stat-value">{totalTrades}</div>
                    </div>
                    <div className="stat-card">
                        <div className="stat-label">
                            <DollarSign size={14} />
                            Volume
                        </div>
                        <div className="stat-value text-sm">
                            ${totalVolume.toLocaleString()}
                        </div>
                    </div>
                    <div className="stat-card">
                        <div className="stat-label">
                            <Building2 size={14} />
                            Indices
                        </div>
                        <div className="stat-value">{indexCount}</div>
                    </div>
                </div>
            </div>
        </div>
    );
};

// === Main Page ===
export const GameControlPage: React.FC = () => {
    return (
        <div className="game-control-grid">
            <GameLifecycleSection />
            <TradingHoursSection />
            <CircuitBreakerSection />
            <QuickStatsSection />
        </div>
    );
};

export default GameControlPage;
