// ============================================
// Trader Layout
// ============================================

import React from 'react';
import { Outlet } from 'react-router-dom';
import { WifiOff, RefreshCw } from 'lucide-react';
import { Header } from './Header';
import { ToastContainer } from '../../components/common';
import { useGameStore } from '../../store/gameStore';
import websocketService from '../../services/websocket';

// Reconnection banner component
const ReconnectionBanner: React.FC = () => {
    const { isConnected } = useGameStore();
    const [isReconnecting, setIsReconnecting] = React.useState(false);

    const handleReconnect = () => {
        setIsReconnecting(true);
        websocketService.disconnect();
        setTimeout(() => {
            websocketService.connect();
            setIsReconnecting(false);
        }, 1000);
    };

    if (isConnected) return null;

    return (
        <div className="reconnection-banner">
            <WifiOff size={16} />
            <span>Connection lost. Data may be outdated.</span>
            <button
                className="reconnect-btn"
                onClick={handleReconnect}
                disabled={isReconnecting}
            >
                <RefreshCw size={14} className={isReconnecting ? 'spin' : ''} />
                {isReconnecting ? 'Reconnecting...' : 'Reconnect'}
            </button>
        </div>
    );
};

export const TraderLayout: React.FC = () => {
    return (
        <div className="app-layout">
            <Header variant="default" />
            <ReconnectionBanner />
            <main className="trader-main-content">
                <Outlet />
            </main>
            <ToastContainer />
        </div>
    );
};

export default TraderLayout;
