// ============================================
// Market Indices Bar Component
// Animated ticker for market indices
// ============================================

import React, { useMemo } from 'react';
import { TrendingUp, ArrowUpRight, ArrowDownRight } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';

const formatPercent = (value: number) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

export const MarketIndicesBar: React.FC = () => {
    const { indices } = useGameStore();
    const indexList = useMemo(() => Object.values(indices), [indices]);

    // Memoize duplicated items for seamless infinite scroll
    const duplicatedItems = useMemo(() => [...indexList, ...indexList], [indexList]);

    return (
        <div className="market-indices-bar">
            <div className="indices-label">
                <TrendingUp size={14} />
                <span>LIVE</span>
            </div>
            <div className="indices-scroll-wrapper">
                {indexList.length === 0 ? (
                    <span className="text-muted text-sm p-2">Waiting for market data...</span>
                ) : (
                    <div className="indices-scroll">
                        {duplicatedItems.map((index, i) => (
                            <div key={`${index.name}-${i}`} className="index-item">
                                <span className="index-name">{index.name.replace('SECTOR:', '')}</span>
                                <span className="index-value">{index.value.toFixed(2)}</span>
                                {index.changePercent !== undefined && index.changePercent !== 0 && (
                                    <span className={`index-change ${index.changePercent >= 0 ? 'positive' : 'negative'}`}>
                                        {index.changePercent >= 0 ? <ArrowUpRight size={12} /> : <ArrowDownRight size={12} />}
                                        {formatPercent(index.changePercent)}
                                    </span>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

export default MarketIndicesBar;
