// ============================================
// Admin Layout with Sidebar
// ============================================

import React from 'react';
import { Outlet, NavLink } from 'react-router-dom';
import {
    LayoutDashboard,
    Gamepad2,
    Users,
    Activity,
    Building2,
    ChevronLeft,
    ChevronRight,
    WifiOff,
    RefreshCw,
    ArrowRightLeft,
    BookOpen,
    BarChart3
} from 'lucide-react';
import { Header } from './Header';
import { ToastContainer } from '../../components/common';
import { useUIStore } from '../../store/uiStore';
import { useGameStore } from '../../store/gameStore';
import websocketService from '../../services/websocket';

interface NavItem {
    path: string;
    label: string;
    icon: React.ReactNode;
}

const navItems: NavItem[] = [
    { path: '/admin', label: 'Dashboard', icon: <LayoutDashboard size={20} /> },
    { path: '/admin/game', label: 'Game Control', icon: <Gamepad2 size={20} /> },
    { path: '/admin/traders', label: 'Traders', icon: <Users size={20} /> },
    { path: '/admin/companies', label: 'Companies', icon: <Building2 size={20} /> },
    { path: '/admin/trades', label: 'Trade History', icon: <ArrowRightLeft size={20} /> },
    { path: '/admin/orders', label: 'Open Orders', icon: <BookOpen size={20} /> },
    { path: '/admin/orderbook', label: 'Orderbook', icon: <BarChart3 size={20} /> },
    { path: '/admin/diagnostics', label: 'Diagnostics', icon: <Activity size={20} /> },
];

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

export const AdminLayout: React.FC = () => {
    const { sidebarCollapsed, toggleSidebar } = useUIStore();

    return (
        <div className="app-layout">
            <Header variant="admin" />
            <ReconnectionBanner />
            <div className="main-with-sidebar">
                {/* Sidebar */}
                <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
                    <nav className="sidebar-nav">
                        <div className="sidebar-section-title">Navigation</div>

                        {navItems.map((item) => (
                            <NavLink
                                key={item.path}
                                to={item.path}
                                end={item.path === '/admin'}
                                className={({ isActive }) =>
                                    `sidebar-nav-item ${isActive ? 'active' : ''}`
                                }
                            >
                                {item.icon}
                                <span>{item.label}</span>
                            </NavLink>
                        ))}
                    </nav>

                    {/* Collapse Toggle */}
                    <div className="mt-auto p-4">
                        <button
                            className="btn btn-ghost w-full justify-center"
                            onClick={toggleSidebar}
                        >
                            {sidebarCollapsed ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
                            {!sidebarCollapsed && <span>Collapse</span>}
                        </button>
                    </div>
                </aside>

                {/* Main Content */}
                <main className="main-content">
                    <Outlet />
                </main>
            </div>
            <ToastContainer />
        </div>
    );
};

export default AdminLayout;
