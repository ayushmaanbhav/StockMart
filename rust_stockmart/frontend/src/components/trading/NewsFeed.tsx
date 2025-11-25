import { useStore } from '../../store/useStore';
import { Newspaper, TrendingUp, TrendingDown, Minus } from 'lucide-react';

export const NewsFeed = () => {
    const { news } = useStore();

    const getSentimentIcon = (sentiment: string) => {
        switch (sentiment) {
            case 'Bullish': return <TrendingUp size={16} className="text-green-500" />;
            case 'Bearish': return <TrendingDown size={16} className="text-red-500" />;
            default: return <Minus size={16} className="text-gray-500" />;
        }
    };

    return (
        <div className="bg-gray-800 p-4 rounded-xl shadow-lg h-full flex flex-col">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Newspaper className="text-yellow-400" /> Market News
            </h2>

            <div className="flex-1 overflow-y-auto space-y-3 pr-2 custom-scrollbar">
                {news.length === 0 ? (
                    <p className="text-gray-500 text-sm italic text-center mt-10">Waiting for news...</p>
                ) : (
                    news.map((item: any) => (
                        <div key={item.id} className="bg-gray-700/30 p-3 rounded-lg border-l-4 border-gray-600 hover:bg-gray-700/50 transition-colors">
                            <div className="flex justify-between items-start mb-1">
                                <span className="text-xs text-gray-400">
                                    {new Date(item.timestamp * 1000).toLocaleTimeString()}
                                </span>
                                {getSentimentIcon(item.sentiment)}
                            </div>
                            <h3 className="text-sm font-medium text-gray-200">{item.headline}</h3>
                            {item.symbol && (
                                <span className="text-xs font-bold text-blue-400 mt-1 inline-block">
                                    #{item.symbol}
                                </span>
                            )}
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};
