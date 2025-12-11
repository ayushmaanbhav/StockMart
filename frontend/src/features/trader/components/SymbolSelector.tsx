// ============================================
// Symbol Selector Component
// Dropdown for selecting trading symbol
// ============================================

import React, { useState, useMemo } from 'react';
import { ChevronDown } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';

export const SymbolSelector: React.FC = () => {
    const { activeSymbol, setActiveSymbol, companies, orderBooks } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);
    const [isOpen, setIsOpen] = useState(false);
    const [search, setSearch] = useState('');

    // Filter companies by search
    const filteredCompanies = useMemo(() =>
        companies.filter(c =>
            c.symbol.toLowerCase().includes(search.toLowerCase()) ||
            c.name.toLowerCase().includes(search.toLowerCase())
        ),
        [companies, search]
    );

    if (!activeSymbol) {
        return (
            <div className="symbol-selector-btn loading">
                <span className="text-muted">Loading...</span>
            </div>
        );
    }

    const orderBook = orderBooks[activeSymbol];
    const currentPrice = orderBook?.asks[0]?.price || orderBook?.bids[0]?.price;

    return (
        <div className="symbol-selector">
            <button
                className="symbol-selector-btn"
                onClick={() => setIsOpen(!isOpen)}
            >
                <div className="symbol-main">
                    <span className="symbol-name">{activeSymbol}</span>
                    {currentPrice && (
                        <span className="symbol-price">{formatCurrency(currentPrice)}</span>
                    )}
                </div>
                <ChevronDown size={14} className={isOpen ? 'rotated' : ''} />
            </button>

            {isOpen && (
                <>
                    <div
                        className="symbol-selector-overlay"
                        onClick={() => {
                            setIsOpen(false);
                            setSearch('');
                        }}
                    />
                    <div className="symbol-selector-dropdown">
                        <div className="symbol-search">
                            <input
                                type="text"
                                placeholder="Search stocks..."
                                value={search}
                                onChange={(e) => setSearch(e.target.value)}
                                autoFocus
                            />
                        </div>
                        <div className="symbol-list">
                            {filteredCompanies.length === 0 && (
                                <div className="symbol-empty">No matches found</div>
                            )}
                            {filteredCompanies.map(company => {
                                const companyOrderBook = orderBooks[company.symbol];
                                const price = companyOrderBook?.asks[0]?.price || companyOrderBook?.bids[0]?.price;
                                const isActive = company.symbol === activeSymbol;

                                return (
                                    <button
                                        key={company.symbol}
                                        className={`symbol-option ${isActive ? 'active' : ''}`}
                                        onClick={() => {
                                            setActiveSymbol(company.symbol);
                                            setIsOpen(false);
                                            setSearch('');
                                        }}
                                    >
                                        <div className="symbol-option-info">
                                            <span className="symbol-option-symbol">{company.symbol}</span>
                                            <span className="symbol-option-name">{company.name}</span>
                                        </div>
                                        {price && (
                                            <span className="symbol-option-price">{formatCurrency(price)}</span>
                                        )}
                                    </button>
                                );
                            })}
                        </div>
                    </div>
                </>
            )}
        </div>
    );
};

export default SymbolSelector;
