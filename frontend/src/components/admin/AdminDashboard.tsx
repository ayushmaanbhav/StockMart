import { useState } from 'react';
import websocketService from '../../services/websocket';
import { Shield, Power, TrendingUp } from 'lucide-react';

export const AdminDashboard = () => {
    const [marketOpen, setMarketOpen] = useState(true);
    const [volatility, setVolatility] = useState('10');
    const [targetSymbol, setTargetSymbol] = useState('AAPL');

    const toggleMarket = () => {
        const newState = !marketOpen;
        setMarketOpen(newState);
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'ToggleMarket',
                payload: { open: newState }
            }
        });
    };

    const updateVolatility = () => {
        websocketService.send({
            type: 'AdminAction',
            payload: {
                action: 'SetVolatility',
                payload: {
                    symbol: targetSymbol,
                    volatility: parseInt(volatility)
                }
            }
        });
    };

    return (
        <div className="bg-gray-800 p-6 rounded-xl shadow-lg border border-red-900/50">
            <h2 className="text-xl font-semibold mb-6 flex items-center gap-2 text-red-400">
                <Shield size={24} /> Admin Control Panel
            </h2>

            <div className="space-y-6">
                {/* Market Status */}
                <div className="flex items-center justify-between p-4 bg-gray-700/50 rounded-lg">
                    <div>
                        <h3 className="font-medium">Market Status</h3>
                        <p className="text-sm text-gray-400">
                            {marketOpen ? 'Market is currently OPEN' : 'Market is currently CLOSED'}
                        </p>
                    </div>
                    <button
                        onClick={toggleMarket}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-bold transition-colors ${marketOpen
                            ? 'bg-red-600 hover:bg-red-700 text-white'
                            : 'bg-green-600 hover:bg-green-700 text-white'
                            }`}
                    >
                        <Power size={18} />
                        {marketOpen ? 'Close Market' : 'Open Market'}
                    </button>
                </div>

                {/* Volatility Control */}
                <div className="p-4 bg-gray-700/50 rounded-lg space-y-4">
                    <h3 className="font-medium flex items-center gap-2">
                        <TrendingUp size={18} /> Market Manipulation
                    </h3>

                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="block text-sm text-gray-400 mb-1">Target Symbol</label>
                            <input
                                type="text"
                                value={targetSymbol}
                                onChange={e => setTargetSymbol(e.target.value)}
                                className="w-full bg-gray-600 border border-gray-500 rounded px-3 py-2 focus:outline-none focus:border-red-500"
                            />
                        </div>
                        <div>
                            <label className="block text-sm text-gray-400 mb-1">Volatility Factor</label>
                            <input
                                type="number"
                                value={volatility}
                                onChange={e => setVolatility(e.target.value)}
                                className="w-full bg-gray-600 border border-gray-500 rounded px-3 py-2 focus:outline-none focus:border-red-500"
                            />
                        </div>
                    </div>

                    <button
                        onClick={updateVolatility}
                        className="w-full bg-red-600/80 hover:bg-red-600 text-white font-medium py-2 rounded transition-colors"
                    >
                        Update Volatility
                    </button>
                </div>

                {/* IPO Section */}
                <div className="p-4 bg-gray-700/50 rounded-lg space-y-4">
                    <h3 className="font-medium flex items-center gap-2">
                        <TrendingUp size={18} /> IPO (Create Company)
                    </h3>
                    <div className="grid grid-cols-2 gap-4">
                        <input type="text" placeholder="Symbol" id="ipo-symbol" className="bg-gray-600 border border-gray-500 rounded px-3 py-2" />
                        <input type="text" placeholder="Name" id="ipo-name" className="bg-gray-600 border border-gray-500 rounded px-3 py-2" />
                        <input type="text" placeholder="Sector" id="ipo-sector" className="bg-gray-600 border border-gray-500 rounded px-3 py-2" />
                        <input type="number" placeholder="Volatility" id="ipo-vol" defaultValue="10" className="bg-gray-600 border border-gray-500 rounded px-3 py-2" />
                    </div>
                    <button
                        onClick={() => {
                            const symbol = (document.getElementById('ipo-symbol') as HTMLInputElement).value;
                            const name = (document.getElementById('ipo-name') as HTMLInputElement).value;
                            const sector = (document.getElementById('ipo-sector') as HTMLInputElement).value;
                            const vol = (document.getElementById('ipo-vol') as HTMLInputElement).value;
                            if (symbol && name && sector) {
                                websocketService.send({
                                    type: 'AdminAction',
                                    payload: {
                                        action: 'CreateCompany',
                                        payload: { symbol, name, sector, volatility: parseInt(vol) }
                                    }
                                });
                                alert(`IPO launched for ${symbol}`);
                            }
                        }}
                        className="w-full bg-blue-600/80 hover:bg-blue-600 text-white font-medium py-2 rounded transition-colors"
                    >
                        Launch IPO
                    </button>
                </div>
            </div>
        </div>
    );
};
