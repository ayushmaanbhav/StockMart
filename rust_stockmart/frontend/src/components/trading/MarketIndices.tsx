import { useStore } from '../../store/useStore';
import { TrendingUp, Activity } from 'lucide-react';

export const MarketIndices = () => {
    const { indices } = useStore();

    const renderIndex = (name: string, value: number) => {
        const isVix = name === 'VIX';
        // For VIX, lower is usually better for stability, but higher means fear.
        // For sectors, we just show value.

        return (
            <div key={name} className="bg-gray-700/50 p-3 rounded-lg flex items-center justify-between min-w-[150px] gap-4">
                <div className="flex flex-col justify-center">
                    <p className="text-xs text-gray-400 uppercase font-bold leading-tight">{name.replace('SECTOR:', '')}</p>
                    <p className="text-lg font-mono font-semibold text-white leading-tight mt-1">{value.toFixed(2)}</p>
                </div>
                <div className="flex items-center justify-center h-full">
                    {isVix ? <Activity size={24} className="text-purple-400" /> : <TrendingUp size={24} className="text-blue-400" />}
                </div>
            </div>
        );
    };

    return (
        <div className="flex gap-4 overflow-x-auto pb-2 mb-6 scrollbar-thin scrollbar-thumb-gray-600">
            {Object.entries(indices).map(([name, value]) => renderIndex(name, value))}
            {Object.keys(indices).length === 0 && (
                <div className="text-gray-500 text-sm italic">Waiting for market indices...</div>
            )}
        </div>
    );
};
