import { useStore } from '../../store/useStore';
import { Briefcase, DollarSign, Lock } from 'lucide-react';

export const Portfolio = () => {
    const { user } = useStore();

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
                        ${user.money.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </p>
                </div>
                {/* We can add locked money here if we update the User interface to include it */}
            </div>

            <div className="space-y-2">
                <h3 className="text-sm font-medium text-gray-400 uppercase mb-2">Holdings</h3>
                {user.portfolio.length === 0 ? (
                    <p className="text-gray-500 text-sm italic">No stocks owned yet.</p>
                ) : (
                    user.portfolio.map((item: any) => (
                        <div key={item.symbol} className="bg-gray-700/30 p-3 rounded flex justify-between items-center">
                            <div>
                                <span className="font-bold text-white">{item.symbol}</span>
                                <div className="text-xs text-gray-400">
                                    Avg: ${(item.average_buy_price / 10000).toFixed(2)}
                                </div>
                            </div>
                            <div className="text-right">
                                <div className="font-mono text-lg">{item.qty}</div>
                                {item.locked_qty > 0 && (
                                    <div className="text-xs text-orange-400 flex items-center justify-end gap-1">
                                        <Lock size={10} /> {item.locked_qty} locked
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
