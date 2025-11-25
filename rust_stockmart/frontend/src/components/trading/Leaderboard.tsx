import { useStore } from '../../store/useStore';
import { Trophy, Medal, User } from 'lucide-react';

export const Leaderboard = () => {
    const { leaderboard, user } = useStore();

    const getRankIcon = (rank: number) => {
        switch (rank) {
            case 1: return <Trophy size={16} className="text-yellow-400" />;
            case 2: return <Medal size={16} className="text-gray-300" />;
            case 3: return <Medal size={16} className="text-amber-600" />;
            default: return <span className="text-gray-500 font-mono w-4 text-center">{rank}</span>;
        }
    };

    return (
        <div className="bg-gray-800 p-4 rounded-xl shadow-lg">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Trophy className="text-yellow-500" /> Leaderboard
            </h2>

            <div className="space-y-2">
                {leaderboard.length === 0 ? (
                    <p className="text-gray-500 text-sm italic text-center">Calculating rankings...</p>
                ) : (
                    leaderboard.map((entry: any) => (
                        <div
                            key={entry.rank}
                            className={`flex justify-between items-center p-2 rounded-lg ${user?.name === entry.name ? 'bg-blue-900/30 border border-blue-500/50' : 'bg-gray-700/30'
                                }`}
                        >
                            <div className="flex items-center gap-3">
                                {getRankIcon(entry.rank)}
                                <span className={`text-sm font-medium ${user?.name === entry.name ? 'text-blue-300' : 'text-gray-200'}`}>
                                    {entry.name}
                                </span>
                            </div>
                            <span className="text-sm font-mono text-green-400">
                                ${(entry.net_worth / 10000).toFixed(2)}
                            </span>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};
