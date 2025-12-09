// ============================================
// Admin Companies Management Page
// ============================================

import React, { useState } from 'react';
import {
    Building2,
    Plus,
    Search,
    TrendingUp,
    TrendingDown,
    AlertTriangle,
    Settings,
    DollarSign,
    Layers,
    Activity
} from 'lucide-react';
import websocketService from '../../services/websocket';
import { Button, Badge, Modal } from '../../components/common';
import { useGameStore } from '../../store/gameStore';
import { useConfigStore } from '../../store/configStore';

// Format helpers
const formatNumber = (value: number) => {
    return new Intl.NumberFormat('en-US').format(value);
};

// UI Company interface (enriched with market data)
interface UICompany {
    symbol: string;
    name: string;
    sector: string;
    price: number;
    change: number;
    volume: number;
    totalShares: number;
    volatility: number;
    isBankrupt: boolean;
}

// === Stats Cards ===
const StatsCards: React.FC<{ companies: UICompany[] }> = ({ companies }) => {
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const totalMarketCap = companies.reduce((sum, c) => sum + (c.price * c.totalShares), 0);
    const totalVolume = companies.reduce((sum, c) => sum + c.volume, 0);
    const avgVolatility = companies.length > 0
        ? companies.reduce((sum, c) => sum + c.volatility, 0) / companies.length
        : 0;
    const bankruptCount = companies.filter(c => c.isBankrupt).length;

    return (
        <div className="stats-row grid grid-cols-4 gap-4 mb-6">
            <div className="stat-card">
                <div className="stat-label">
                    <Building2 size={14} />
                    Total Companies
                </div>
                <div className="stat-value">{companies.length}</div>
            </div>
            <div className="stat-card">
                <div className="stat-label">
                    <DollarSign size={14} />
                    Total Market Cap
                </div>
                <div className="stat-value text-sm">{formatCurrency(totalMarketCap)}</div>
            </div>
            <div className="stat-card">
                <div className="stat-label">
                    <Activity size={14} />
                    Total Volume
                </div>
                <div className="stat-value">{formatNumber(totalVolume)}</div>
            </div>
            <div className="stat-card">
                <div className="stat-label">
                    <AlertTriangle size={14} />
                    Avg Volatility
                </div>
                <div className="stat-value">{(avgVolatility * 100).toFixed(1)}%</div>
                {bankruptCount > 0 && (
                    <Badge variant="danger" className="mt-2">{bankruptCount} Bankrupt</Badge>
                )}
            </div>
        </div>
    );
};

// === Create Company Modal ===
interface CreateCompanyModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSubmit: (company: Partial<UICompany>) => void;
}

const CreateCompanyModal: React.FC<CreateCompanyModalProps> = ({ isOpen, onClose, onSubmit }) => {
    const [symbol, setSymbol] = useState('');
    const [name, setName] = useState('');
    const [sector, setSector] = useState('Tech');
    const [volatility, setVolatility] = useState('0.25');
    const [isLoading, setIsLoading] = useState(false);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);

        onSubmit({
            symbol: symbol.toUpperCase(),
            name,
            sector,
            volatility: parseFloat(volatility),
        });

        setTimeout(() => {
            setIsLoading(false);
            setSymbol('');
            setName('');
            setSector('Tech');
            setVolatility('0.25');
            onClose();
        }, 1000);
    };

    if (!isOpen) return null;

    return (
        <Modal title="Create New Company" onClose={onClose} isOpen={isOpen}>
            <form onSubmit={handleSubmit}>
                <div className="space-y-4">
                    <div className="input-group">
                        <label className="input-label">Symbol</label>
                        <input
                            type="text"
                            className="input"
                            placeholder="e.g., TSLA"
                            value={symbol}
                            onChange={(e) => setSymbol(e.target.value.toUpperCase())}
                            maxLength={5}
                            required
                        />
                        <span className="text-xs text-muted mt-1">Max 5 characters, uppercase</span>
                    </div>

                    <div className="input-group">
                        <label className="input-label">Company Name</label>
                        <input
                            type="text"
                            className="input"
                            placeholder="e.g., Tesla Inc."
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            required
                        />
                    </div>

                    <div className="input-group">
                        <label className="input-label">Sector</label>
                        <select
                            className="input"
                            value={sector}
                            onChange={(e) => setSector(e.target.value)}
                        >
                            <option value="Tech">Technology</option>
                            <option value="Finance">Finance</option>
                            <option value="Healthcare">Healthcare</option>
                            <option value="Energy">Energy</option>
                            <option value="Consumer">Consumer</option>
                            <option value="Industrial">Industrial</option>
                        </select>
                    </div>

                    <div className="input-group">
                        <label className="input-label">Volatility Factor</label>
                        <input
                            type="range"
                            min="0.1"
                            max="0.5"
                            step="0.05"
                            value={volatility}
                            onChange={(e) => setVolatility(e.target.value)}
                            className="w-full"
                        />
                        <div className="flex justify-between text-xs text-muted mt-1">
                            <span>Low (10%)</span>
                            <span className="font-bold">{(parseFloat(volatility) * 100).toFixed(0)}%</span>
                            <span>High (50%)</span>
                        </div>
                    </div>

                    <div className="p-3 rounded-lg" style={{ background: 'var(--bg-tertiary)' }}>
                        <div className="text-sm text-muted">
                            <strong>Default Values:</strong>
                            <ul className="mt-2 space-y-1">
                                <li>• Total Shares: 1,000,000</li>
                                <li>• Initial Price: $100.00</li>
                                <li>• Price Precision: 2 decimals</li>
                            </ul>
                        </div>
                    </div>
                </div>

                <div className="flex gap-3 mt-6">
                    <Button variant="secondary" type="button" onClick={onClose} className="flex-1">
                        Cancel
                    </Button>
                    <Button variant="primary" type="submit" loading={isLoading} className="flex-1">
                        <Plus size={16} />
                        Create Company
                    </Button>
                </div>
            </form>
        </Modal>
    );
};

// === Company Row ===
interface CompanyRowProps {
    company: UICompany;
    onEdit: (company: UICompany) => void;
    onSetBankrupt: (symbol: string) => void;
}

const CompanyRow: React.FC<CompanyRowProps> = ({ company, onEdit, onSetBankrupt }) => {
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [showActions, setShowActions] = useState(false);
    const changePercent = (company.change / company.price) * 100;
    const isPositive = company.change >= 0;

    return (
        <tr
            className="company-row"
            onMouseEnter={() => setShowActions(true)}
            onMouseLeave={() => setShowActions(false)}
        >
            <td>
                <div className="flex items-center gap-3">
                    <div className="symbol-badge" style={{
                        background: company.isBankrupt ? 'var(--color-danger)' : 'var(--color-primary)',
                        color: 'white',
                        padding: '4px 8px',
                        borderRadius: 'var(--radius-sm)',
                        fontWeight: 'bold',
                        fontSize: '12px'
                    }}>
                        {company.symbol}
                    </div>
                    <div>
                        <div className="font-medium">{company.name}</div>
                        <div className="text-xs text-muted">{company.sector}</div>
                    </div>
                </div>
            </td>
            <td className="text-right font-mono">
                {formatCurrency(company.price)}
            </td>
            <td className={`text-right ${isPositive ? 'text-success' : 'text-danger'}`}>
                <div className="flex items-center justify-end gap-1">
                    {isPositive ? <TrendingUp size={14} /> : <TrendingDown size={14} />}
                    {isPositive ? '+' : ''}{changePercent.toFixed(2)}%
                </div>
            </td>
            <td className="text-right font-mono">
                {formatNumber(company.volume)}
            </td>
            <td className="text-right">
                <Badge variant={company.volatility > 0.3 ? 'warning' : 'primary'}>
                    {(company.volatility * 100).toFixed(0)}%
                </Badge>
            </td>
            <td className="text-right">
                {company.isBankrupt ? (
                    <Badge variant="danger">Bankrupt</Badge>
                ) : (
                    <Badge variant="success">Active</Badge>
                )}
            </td>
            <td className="text-right">
                <div className={`flex items-center justify-end gap-1 transition-opacity ${showActions ? 'opacity-100' : 'opacity-0'}`}>
                    <Button variant="ghost" size="sm" onClick={() => onEdit(company)}>
                        <Settings size={14} />
                    </Button>
                    {!company.isBankrupt && (
                        <Button variant="ghost" size="sm" onClick={() => onSetBankrupt(company.symbol)}>
                            <AlertTriangle size={14} />
                        </Button>
                    )}
                </div>
            </td>
        </tr>
    );
};

// === Edit Company Modal ===
interface EditCompanyModalProps {
    company: UICompany | null;
    isOpen: boolean;
    onClose: () => void;
    onSave: (symbol: string, volatility: number) => void;
}

const EditCompanyModal: React.FC<EditCompanyModalProps> = ({ company, isOpen, onClose, onSave }) => {
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [volatility, setVolatility] = useState(company?.volatility.toString() || '0.25');
    const [isLoading, setIsLoading] = useState(false);

    React.useEffect(() => {
        if (company) {
            setVolatility(company.volatility.toString());
        }
    }, [company]);

    const handleSave = () => {
        if (!company) return;
        setIsLoading(true);
        onSave(company.symbol, parseFloat(volatility));
        setTimeout(() => {
            setIsLoading(false);
            onClose();
        }, 1000);
    };

    if (!isOpen || !company) return null;

    return (
        <Modal title={`Edit ${company.symbol}`} onClose={onClose} isOpen={isOpen}>
            <div className="space-y-4">
                <div className="p-4 rounded-lg" style={{ background: 'var(--bg-tertiary)' }}>
                    <div className="flex items-center gap-3 mb-3">
                        <div className="symbol-badge" style={{
                            background: 'var(--color-primary)',
                            color: 'white',
                            padding: '8px 12px',
                            borderRadius: 'var(--radius-sm)',
                            fontWeight: 'bold'
                        }}>
                            {company.symbol}
                        </div>
                        <div>
                            <div className="font-medium">{company.name}</div>
                            <div className="text-sm text-muted">{company.sector}</div>
                        </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                            <span className="text-muted">Current Price:</span>
                            <span className="ml-2 font-mono">{formatCurrency(company.price)}</span>
                        </div>
                        <div>
                            <span className="text-muted">Total Shares:</span>
                            <span className="ml-2 font-mono">{formatNumber(company.totalShares)}</span>
                        </div>
                    </div>
                </div>

                <div className="input-group">
                    <label className="input-label">Volatility Factor</label>
                    <input
                        type="range"
                        min="0.1"
                        max="0.5"
                        step="0.05"
                        value={volatility}
                        onChange={(e) => setVolatility(e.target.value)}
                        className="w-full"
                    />
                    <div className="flex justify-between text-xs text-muted mt-1">
                        <span>Low Risk</span>
                        <span className="font-bold text-primary">{(parseFloat(volatility) * 100).toFixed(0)}%</span>
                        <span>High Risk</span>
                    </div>
                </div>

                <div className="p-3 rounded-lg" style={{ background: 'var(--color-warning-bg)', border: '1px solid var(--color-warning)' }}>
                    <div className="flex items-start gap-2">
                        <AlertTriangle size={16} style={{ color: 'var(--color-warning)', flexShrink: 0 }} />
                        <div className="text-sm">
                            Higher volatility means bigger price swings. Use with caution during active trading.
                        </div>
                    </div>
                </div>
            </div>

            <div className="flex gap-3 mt-6">
                <Button variant="secondary" onClick={onClose} className="flex-1">
                    Cancel
                </Button>
                <Button variant="primary" onClick={handleSave} loading={isLoading} className="flex-1">
                    Save Changes
                </Button>
            </div>
        </Modal>
    );
};

// === Main Page ===
export const CompaniesPage: React.FC = () => {
    const { companies: storeCompanies, candles, trades } = useGameStore();
    const [searchTerm, setSearchTerm] = useState('');
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [editingCompany, setEditingCompany] = useState<UICompany | null>(null);
    const [confirmBankrupt, setConfirmBankrupt] = useState<string | null>(null);
    const [localBankrupt, setLocalBankrupt] = useState<Set<string>>(new Set());

    // Transform store companies to UI companies with market data
    const companies: UICompany[] = storeCompanies.map(c => {
        const symbolCandles = candles[c.symbol] || [];
        const latestCandle = symbolCandles[symbolCandles.length - 1];
        const prevCandle = symbolCandles[symbolCandles.length - 2];

        // Calculate price from latest candle, or default to 100
        const price = latestCandle?.close ?? 100;
        const prevPrice = prevCandle?.close ?? price;
        const change = price - prevPrice;

        // Calculate volume from recent trades for this symbol
        const symbolTrades = trades.filter(t => t.symbol === c.symbol);
        const volume = symbolTrades.reduce((sum, t) => sum + t.qty, 0);

        return {
            symbol: c.symbol,
            name: c.name,
            sector: c.sector,
            price,
            change,
            volume,
            totalShares: 1000000, // Default total shares
            volatility: c.volatility / 100, // Backend stores as integer (25 = 25%), UI expects decimal (0.25)
            isBankrupt: localBankrupt.has(c.symbol),
        };
    });

    // Filter companies based on search
    const filteredCompanies = companies.filter(c =>
        c.symbol.toLowerCase().includes(searchTerm.toLowerCase()) ||
        c.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        c.sector.toLowerCase().includes(searchTerm.toLowerCase())
    );

    // Handlers
    const handleCreateCompany = (company: Partial<UICompany>) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'CreateCompany',
                payload: company
            }
        });
        // Backend will send updated CompanyList
    };

    const handleSetVolatility = (symbol: string, volatility: number) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'SetVolatility',
                payload: { symbol, volatility: Math.round(volatility * 100) } // Convert to integer for backend
            }
        });
    };

    const handleSetBankrupt = (symbol: string) => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'SetBankrupt',
                payload: { symbol }
            }
        });
        // Update local state for immediate feedback
        setLocalBankrupt(prev => new Set(prev).add(symbol));
        setConfirmBankrupt(null);
    };

    return (
        <div className="companies-page">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold flex items-center gap-2">
                        <Building2 size={24} />
                        Company Management
                    </h1>
                    <p className="text-muted mt-1">Manage stocks, IPOs, and company settings</p>
                </div>
                <Button variant="primary" onClick={() => setShowCreateModal(true)}>
                    <Plus size={16} />
                    New Company (IPO)
                </Button>
            </div>

            {/* Stats Cards */}
            <StatsCards companies={companies} />

            {/* Companies Table */}
            <div className="panel">
                <div className="panel-header">
                    <div className="panel-title">
                        <Layers size={18} />
                        Listed Companies
                    </div>
                    <div className="flex items-center gap-3">
                        <div className="relative">
                            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
                            <input
                                type="text"
                                className="input pl-9"
                                placeholder="Search companies..."
                                value={searchTerm}
                                onChange={(e) => setSearchTerm(e.target.value)}
                                style={{ width: '250px' }}
                            />
                        </div>
                    </div>
                </div>
                <div className="panel-body p-0">
                    <table className="table w-full">
                        <thead>
                            <tr>
                                <th className="text-left">Company</th>
                                <th className="text-right">Price</th>
                                <th className="text-right">Change</th>
                                <th className="text-right">Volume</th>
                                <th className="text-right">Volatility</th>
                                <th className="text-right">Status</th>
                                <th className="text-right w-24">Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filteredCompanies.map(company => (
                                <CompanyRow
                                    key={company.symbol}
                                    company={company}
                                    onEdit={setEditingCompany}
                                    onSetBankrupt={(symbol) => setConfirmBankrupt(symbol)}
                                />
                            ))}
                            {filteredCompanies.length === 0 && (
                                <tr>
                                    <td colSpan={7} className="text-center py-8 text-muted">
                                        No companies found matching "{searchTerm}"
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Create Modal */}
            <CreateCompanyModal
                isOpen={showCreateModal}
                onClose={() => setShowCreateModal(false)}
                onSubmit={handleCreateCompany}
            />

            {/* Edit Modal */}
            <EditCompanyModal
                company={editingCompany}
                isOpen={!!editingCompany}
                onClose={() => setEditingCompany(null)}
                onSave={handleSetVolatility}
            />

            {/* Bankrupt Confirmation */}
            {confirmBankrupt && (
                <Modal
                    title="Confirm Bankruptcy"
                    onClose={() => setConfirmBankrupt(null)}
                    isOpen={true}
                >
                    <div className="text-center">
                        <AlertTriangle size={48} style={{ color: 'var(--color-danger)', margin: '0 auto 16px' }} />
                        <p className="mb-2">
                            Are you sure you want to mark <strong>{confirmBankrupt}</strong> as bankrupt?
                        </p>
                        <p className="text-sm text-muted mb-6">
                            This will halt all trading for this stock and traders will lose their positions.
                        </p>
                        <div className="flex gap-3 justify-center">
                            <Button variant="secondary" onClick={() => setConfirmBankrupt(null)}>
                                Cancel
                            </Button>
                            <Button variant="danger" onClick={() => handleSetBankrupt(confirmBankrupt)}>
                                Yes, Mark Bankrupt
                            </Button>
                        </div>
                    </div>
                </Modal>
            )}
        </div>
    );
};

export default CompaniesPage;
