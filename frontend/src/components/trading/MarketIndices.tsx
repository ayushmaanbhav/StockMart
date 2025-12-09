import { useGameStore } from '../../store/gameStore';
import type { MarketIndex } from '../../types/models';
import { TrendingUp, Activity } from 'lucide-react';

export const MarketIndices = () => {
    const { indices } = useGameStore();

    const renderIndex = (name: string, index: MarketIndex) => {
        const isVix = name === 'VIX';
        const change = index.change ?? 0;
        const changePercent = index.changePercent ?? 0;
        const isPositive = change >= 0;

        return (
            <div key={name} className="bg-gray-700/50 p-3 rounded-lg flex items-center justify-between min-w-[150px] gap-4">
                <div className="flex flex-col justify-center">
                    <p className="text-xs text-gray-400 uppercase font-bold leading-tight">{name.replace('SECTOR:', '')}</p>
                    <p className="text-lg font-mono font-semibold text-white leading-tight mt-1">{index.value.toFixed(2)}</p>
                    {change !== 0 && (
                        <p className={`text-xs font-mono ${isPositive ? 'text-green-400' : 'text-red-400'}`}>
                            {isPositive ? '+' : ''}{changePercent.toFixed(2)}%
                        </p>
                    )}
                </div>
                <div className="flex items-center justify-center h-full">
                    {isVix ? <Activity size={24} className="text-purple-400" /> : <TrendingUp size={24} className={isPositive ? 'text-green-400' : 'text-red-400'} />}
                </div>
            </div>
        );
    };

    return (
        <div className="flex gap-4 overflow-x-auto pb-2 mb-6 scrollbar-thin scrollbar-thumb-gray-600">
            {Object.entries(indices).map(([name, index]) => renderIndex(name, index))}
            {Object.keys(indices).length === 0 && (
                <div className="text-gray-500 text-sm italic">Waiting for market indices...</div>
            )}
        </div>
    );
};
