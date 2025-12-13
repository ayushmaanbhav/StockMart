// ============================================
// Login Page
// ============================================

import React, { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { TrendingUp, User, Lock, Shield, ChevronRight } from 'lucide-react';
import { useAuthStore } from '../../store/authStore';
import { Button, Input, Tabs } from '../../components/common';

type LoginMode = 'trader' | 'admin';

interface ValidationErrors {
    regno?: string;
    password?: string;
    adminUsername?: string;
    adminPassword?: string;
}

export const LoginPage: React.FC = () => {
    const navigate = useNavigate();
    const { login, loginAdmin, isAuthenticated, isLoading, error, clearError, user } = useAuthStore();

    const [mode, setMode] = useState<LoginMode>('trader');
    const [regno, setRegno] = useState('');
    const [password, setPassword] = useState('');
    const [adminUsername, setAdminUsername] = useState('');
    const [adminPassword, setAdminPassword] = useState('');
    const [validationErrors, setValidationErrors] = useState<ValidationErrors>({});
    const [touched, setTouched] = useState<Record<string, boolean>>({});

    // Redirect if already authenticated
    useEffect(() => {
        if (isAuthenticated && user) {
            if (user.role === 'admin') {
                navigate('/admin');
            } else {
                navigate('/trade');
            }
        }
    }, [isAuthenticated, user, navigate]);

    // Clear errors on mode change
    useEffect(() => {
        clearError();
        // Use queueMicrotask to avoid synchronous setState in effect body
        queueMicrotask(() => {
            setValidationErrors({});
            setTouched({});
        });
    }, [mode, clearError]);

    // Validate trader form
    const validateTraderForm = (): boolean => {
        const errors: ValidationErrors = {};

        if (!regno.trim()) {
            errors.regno = 'Registration number is required';
        } else if (regno.trim().length < 3) {
            errors.regno = 'Registration number must be at least 3 characters';
        }

        if (!password) {
            errors.password = 'Password is required';
        } else if (password.length < 4) {
            errors.password = 'Password must be at least 4 characters';
        }

        setValidationErrors(errors);
        return Object.keys(errors).length === 0;
    };

    // Validate admin form
    const validateAdminForm = (): boolean => {
        const errors: ValidationErrors = {};

        if (!adminUsername.trim()) {
            errors.adminUsername = 'Username is required';
        }

        if (!adminPassword) {
            errors.adminPassword = 'Password is required';
        }

        setValidationErrors(errors);
        return Object.keys(errors).length === 0;
    };

    const handleBlur = (field: string) => {
        setTouched(prev => ({ ...prev, [field]: true }));
    };

    const handleTraderLogin = (e: React.FormEvent) => {
        e.preventDefault();
        setTouched({ regno: true, password: true });
        if (validateTraderForm()) {
            login(regno.trim(), password);
        }
    };

    const handleAdminLogin = (e: React.FormEvent) => {
        e.preventDefault();
        setTouched({ adminUsername: true, adminPassword: true });
        if (validateAdminForm()) {
            loginAdmin(adminUsername.trim(), adminPassword);
        }
    };

    const tabs = [
        { id: 'trader', label: 'Trader', icon: <User size={16} /> },
        { id: 'admin', label: 'Admin', icon: <Shield size={16} /> },
    ];

    return (
        <div className="auth-layout">
            <div className="auth-container">
                <div className="auth-card">
                    {/* Logo */}
                    <div className="auth-header">
                        <div className="auth-logo">
                            <TrendingUp size={40} />
                            <span>StockMart</span>
                        </div>
                        <h1 className="auth-title">Welcome Back</h1>
                        <p className="auth-subtitle">Sign in to continue trading</p>
                    </div>

                    {/* Role Tabs */}
                    <Tabs
                        tabs={tabs}
                        activeTab={mode}
                        onChange={(id) => setMode(id as LoginMode)}
                    />

                    {/* Error Message */}
                    {error && (
                        <div className="mt-4 p-3 rounded-lg" style={{
                            background: 'var(--color-danger-bg)',
                            border: '1px solid var(--color-danger)',
                            color: 'var(--color-danger)',
                            fontSize: 'var(--text-sm)'
                        }}>
                            {error}
                        </div>
                    )}

                    {/* Trader Login Form */}
                    {mode === 'trader' && (
                        <form onSubmit={handleTraderLogin} className="auth-form mt-6">
                            <Input
                                label="Registration Number"
                                placeholder="Enter your registration ID"
                                value={regno}
                                onChange={(e) => {
                                    setRegno(e.target.value);
                                    if (touched.regno) {
                                        setValidationErrors(prev => ({ ...prev, regno: undefined }));
                                    }
                                }}
                                onBlur={() => handleBlur('regno')}
                                icon={<User size={18} />}
                                autoComplete="username"
                                error={touched.regno ? validationErrors.regno : undefined}
                            />

                            <Input
                                label="Password"
                                type="password"
                                placeholder="Enter your password"
                                value={password}
                                onChange={(e) => {
                                    setPassword(e.target.value);
                                    if (touched.password) {
                                        setValidationErrors(prev => ({ ...prev, password: undefined }));
                                    }
                                }}
                                onBlur={() => handleBlur('password')}
                                icon={<Lock size={18} />}
                                autoComplete="current-password"
                                error={touched.password ? validationErrors.password : undefined}
                            />

                            <Button
                                type="submit"
                                variant="primary"
                                size="lg"
                                loading={isLoading}
                                className="w-full mt-2"
                            >
                                Sign In
                                <ChevronRight size={18} />
                            </Button>
                        </form>
                    )}

                    {/* Admin Login Form */}
                    {mode === 'admin' && (
                        <form onSubmit={handleAdminLogin} className="auth-form mt-6">
                            <Input
                                label="Username"
                                placeholder="Admin username"
                                value={adminUsername}
                                onChange={(e) => {
                                    setAdminUsername(e.target.value);
                                    if (touched.adminUsername) {
                                        setValidationErrors(prev => ({ ...prev, adminUsername: undefined }));
                                    }
                                }}
                                onBlur={() => handleBlur('adminUsername')}
                                icon={<Shield size={18} />}
                                autoComplete="username"
                                error={touched.adminUsername ? validationErrors.adminUsername : undefined}
                            />

                            <Input
                                label="Password"
                                type="password"
                                placeholder="Admin password"
                                value={adminPassword}
                                onChange={(e) => {
                                    setAdminPassword(e.target.value);
                                    if (touched.adminPassword) {
                                        setValidationErrors(prev => ({ ...prev, adminPassword: undefined }));
                                    }
                                }}
                                onBlur={() => handleBlur('adminPassword')}
                                icon={<Lock size={18} />}
                                autoComplete="current-password"
                                error={touched.adminPassword ? validationErrors.adminPassword : undefined}
                            />

                            <Button
                                type="submit"
                                variant="danger"
                                size="lg"
                                loading={isLoading}
                                className="w-full mt-2"
                            >
                                Admin Login
                                <Shield size={18} />
                            </Button>
                        </form>
                    )}

                    {/* Footer */}
                    {mode === 'trader' && (
                        <div className="auth-footer">
                            Don't have an account?{' '}
                            <Link to="/register">Register here</Link>
                        </div>
                    )}

                    {mode === 'admin' && (
                        <div className="auth-footer" style={{ color: 'var(--text-muted)', fontSize: 'var(--text-xs)' }}>
                            Admin access is restricted.
                            <br />
                            Contact system administrator for credentials.
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

export default LoginPage;
