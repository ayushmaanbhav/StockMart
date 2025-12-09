import { useGameStore } from '../../store/gameStore';
import { useAuthStore } from '../../store/authStore';
import { useConfigStore } from '../../store/configStore';
import { Briefcase, DollarSign, Lock } from 'lucide-react';

export const Portfolio = () => {
    const { portfolio, money, lockedMoney, marginLocked, netWorth } = useGameStore();
    const { user } = useAuthStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    if (!user) return null;

    return (
        <div className="bg-gray-800 p-6 rounded-xl shadow-lg">
            <h2 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Briefcase className="text-blue-400" /> Your Portfolio
            </h2>

            <div className="grid grid-cols-2 gap-4 mb-6">
                <div className="bg-gray-700/50 p-3 rounded-lg">
                    <p className="text-sm text-gray-400 flex items-center gap-1">
                        <DollarSign size={14} /> Available Cash
                    </p>
                    <p className="text-2xl font-mono font-bold text-green-400">
                        {formatCurrency(money)}
                    </p>
                </div>
                <div className="bg-gray-700/50 p-3 rounded-lg">
                    <p className="text-sm text-gray-400 flex items-center gap-1">
                        <Lock size={14} /> Locked
                    </p>
                    <p className="text-2xl font-mono font-bold text-orange-400">
                        {formatCurrency(lockedMoney + marginLocked)}
                    </p>
                </div>
            </div>

            <div className="bg-gray-700/30 p-3 rounded-lg mb-4">
                <p className="text-sm text-gray-400">Net Worth</p>
                <p className="text-3xl font-mono font-bold text-blue-400">
                    {formatCurrency(netWorth)}
                </p>
            </div>

            <div className="space-y-2">
                <h3 className="text-sm font-medium text-gray-400 uppercase mb-2">Holdings</h3>
                {portfolio.length === 0 ? (
                    <p className="text-gray-500 text-sm italic">No stocks owned yet.</p>
                ) : (
                    portfolio.map((item) => (
                        <div key={item.symbol} className="bg-gray-700/30 p-3 rounded flex justify-between items-center">
                            <div>
                                <span className="font-bold text-white">{item.symbol}</span>
                                <div className="text-xs text-gray-400">
                                    Avg: {formatCurrency(item.averageBuyPrice)}
                                </div>
                            </div>
                            <div className="text-right">
                                <div className="font-mono text-lg">
                                    {item.qty > 0 && <span className="text-green-400">{item.qty}</span>}
                                    {item.shortQty > 0 && <span className="text-red-400 ml-2">-{item.shortQty}</span>}
                                </div>
                                {item.lockedQty > 0 && (
                                    <div className="text-xs text-orange-400 flex items-center justify-end gap-1">
                                        <Lock size={10} /> {item.lockedQty} locked
                                    </div>
                                )}
                            </div>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};
