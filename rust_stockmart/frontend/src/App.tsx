import { useState } from 'react';
import { useStore } from './store/useStore';
import { socketService } from './services/socket';
import { TrendingUp, Wifi, WifiOff } from 'lucide-react';

import { Chart } from './components/charts/Chart';
import { AdminDashboard } from './components/admin/AdminDashboard';
import { MarketIndices } from './components/trading/MarketIndices';
import { Portfolio } from './components/trading/Portfolio';
import { NewsFeed } from './components/trading/NewsFeed';
import { Leaderboard } from './components/trading/Leaderboard';
import { Chat } from './components/social/Chat';

function App() {
  const { user, isConnected, trades, halted } = useStore();
  const [symbol, setSymbol] = useState('AAPL');
  const [price, setPrice] = useState('150');
  const [qty, setQty] = useState('10');
  const [orderSide, setOrderSide] = useState<'Buy' | 'Sell' | 'Short'>('Buy');

  const isHalted = halted[symbol] && halted[symbol] > Date.now() / 1000;

  const handleOrder = () => {
    if (isHalted) {
      alert('Trading is currently halted for this symbol due to volatility.');
      return;
    }
    socketService.send('PlaceOrder', {
      symbol,
      side: orderSide,
      order_type: 'Limit',
      qty: parseInt(qty),
      price: parseFloat(price) * 10000 // Scale up
    });
  };

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 font-sans">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 p-4 flex justify-between items-center">
        <div className="flex items-center gap-2">
          <TrendingUp className="text-blue-500" />
          <h1 className="text-xl font-bold">StockMart</h1>
          {isHalted && (
            <span className="bg-red-600 text-white text-xs font-bold px-2 py-1 rounded animate-pulse">
              MARKET HALTED: {symbol}
            </span>
          )}
        </div>
        <div className="flex items-center gap-4">
          {user && <span className="text-gray-300">Welcome, {user.name}</span>}
          <div className={`flex items - center gap - 2 px - 3 py - 1 rounded - full ${isConnected ? 'bg-green-900 text-green-300' : 'bg-red-900 text-red-300'} `}>
            {isConnected ? <Wifi size={16} /> : <WifiOff size={16} />}
            <span className="text-sm font-medium">{isConnected ? 'Connected' : 'Disconnected'}</span>
          </div>
        </div>
      </header>

      <main className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Chart Section */}
        <div className="lg:col-span-2 space-y-8">
          <MarketIndices />
          <Chart />

          {/* Trading Panel */}
          <div className="bg-gray-800 p-6 rounded-xl shadow-lg">
            <h2 className="text-xl font-semibold mb-4">Place Order</h2>

            {/* Order Side Tabs */}
            <div className="flex gap-2 mb-4 bg-gray-700 p-1 rounded-lg">
              {['Buy', 'Sell', 'Short'].map((side) => (
                <button
                  key={side}
                  onClick={() => setOrderSide(side as any)}
                  className={`flex - 1 py - 1 px - 3 rounded - md text - sm font - medium transition - colors ${orderSide === side
                    ? (side === 'Buy' ? 'bg-green-600 text-white' : side === 'Sell' ? 'bg-red-600 text-white' : 'bg-purple-600 text-white')
                    : 'text-gray-400 hover:text-white hover:bg-gray-600'
                    } `}
                >
                  {side}
                </button>
              ))}
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Symbol</label>
                <input
                  type="text"
                  value={symbol}
                  onChange={e => setSymbol(e.target.value)}
                  className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Price</label>
                  <input
                    type="number"
                    value={price}
                    onChange={e => setPrice(e.target.value)}
                    className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Quantity</label>
                  <input
                    type="number"
                    value={qty}
                    onChange={e => setQty(e.target.value)}
                    className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:outline-none focus:border-blue-500"
                  />
                </div>
              </div>
              <button
                onClick={handleOrder}
                className={`w - full font - bold py - 2 px - 4 rounded transition - colors text - white ${orderSide === 'Buy' ? 'bg-green-600 hover:bg-green-700' :
                  orderSide === 'Sell' ? 'bg-red-600 hover:bg-red-700' :
                    'bg-purple-600 hover:bg-purple-700'
                  } `}
              >
                {orderSide} {symbol}
              </button>
            </div>
          </div>

          <Portfolio />

          {/* Admin Dashboard */}
          {user?.id === 1 && <AdminDashboard />}
        </div>

        {/* Right Column */}
        <div className="space-y-8">
          <Leaderboard />
          <Chat />
          <NewsFeed />

          {/* Market Feed */}
          <div className="bg-gray-800 p-6 rounded-xl shadow-lg h-fit">
            <h2 className="text-xl font-semibold mb-4">Market Feed</h2>
            <div className="space-y-2 max-h-[400px] overflow-y-auto">
              {trades.length === 0 ? (
                <p className="text-gray-500 text-center py-8">No trades yet...</p>
              ) : (
                trades.map((trade, i) => (
                  <div key={i} className="flex justify-between items-center p-3 bg-gray-700/50 rounded border-l-4 border-blue-500">
                    <span className="font-bold">{trade.symbol}</span>
                    <div className="flex gap-4 text-sm">
                      <span>{trade.qty} shares</span>
                      <span className="font-mono text-blue-300">${trade.price.toFixed(2)}</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
