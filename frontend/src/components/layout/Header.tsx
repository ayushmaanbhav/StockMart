// ============================================
// Header Component
// ============================================

import React from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { TrendingUp, Sun, Moon, Wifi, WifiOff, LogOut } from 'lucide-react';
import { useAuthStore } from '../../store/authStore';
import { useGameStore } from '../../store/gameStore';
import { useUIStore } from '../../store/uiStore';
import { useConfigStore } from '../../store/configStore';
import { Badge } from '../../components/common';

interface HeaderProps {
    showNav?: boolean;
    variant?: 'default' | 'admin';
}

export const Header: React.FC<HeaderProps> = ({ showNav = true, variant = 'default' }) => {
    const navigate = useNavigate();
    const { user, logout, isAuthenticated } = useAuthStore();
    const { isConnected, marketOpen, netWorth } = useGameStore();
    const { theme, toggleTheme } = useUIStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    const handleLogout = () => {
        logout();
        navigate('/login');
    };

    return (
        <header className="header">
            {/* Brand */}
            <div className="header-brand">
                <Link to={user?.role === 'admin' ? '/admin' : '/trade'} className="header-logo">
                    <TrendingUp size={28} />
                    <span>StockMart</span>
                </Link>

                {variant === 'admin' && (
                    <Badge variant="danger">ADMIN</Badge>
                )}

                {isAuthenticated && !marketOpen && (
                    <Badge variant="warning" pulse>MARKET CLOSED</Badge>
                )}
            </div>

            {/* Center Stats - Trader Only */}
            {showNav && variant !== 'admin' && isAuthenticated && (
                <div className="header-nav" style={{ gap: 'var(--space-6)' }}>
                    <div className="text-center">
                        <div className="text-xs text-muted">Net Worth</div>
                        <div className="font-bold text-mono" style={{
                            color: 'var(--color-success)',
                            fontSize: 'var(--text-lg)'
                        }}>
                            {formatCurrency(netWorth)}
                        </div>
                    </div>
                </div>
            )}

            {/* Right Actions */}
            <div className="header-actions">
                {/* Connection Status */}
                <div className={`header-status ${isConnected ? 'connected' : 'disconnected'}`}>
                    <span className="header-status-dot" />
                    {isConnected ? <Wifi size={14} /> : <WifiOff size={14} />}
                    <span className="text-xs font-medium">
                        {isConnected ? 'Live' : 'Offline'}
                    </span>
                </div>

                {/* Theme Toggle */}
                <button className="theme-toggle" onClick={toggleTheme} title="Toggle theme">
                    {theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}
                </button>

                {/* User Menu */}
                {isAuthenticated && user && (
                    <div className="header-user">
                        <div className="header-user-avatar">
                            {user.name.charAt(0).toUpperCase()}
                        </div>
                        <div className="header-user-info">
                            <span className="header-user-name">{user.name}</span>
                            <span className="header-user-role">{user.role === 'admin' ? 'Administrator' : 'Trader'}</span>
                        </div>
                    </div>
                )}

                {/* Logout Button */}
                {isAuthenticated && (
                    <button
                        className="btn btn-ghost btn-icon"
                        onClick={handleLogout}
                        title="Logout"
                    >
                        <LogOut size={18} />
                    </button>
                )}
            </div>
        </header>
    );
};

export default Header;
