// ============================================
// Register Page
// ============================================

import React, { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { TrendingUp, User, Lock, Hash, ChevronRight, CheckCircle } from 'lucide-react';
import { useAuthStore } from '../../store/authStore';
import { Button, Input } from '../../components/common';

interface ValidationErrors {
    regno?: string;
    name?: string;
    password?: string;
    confirmPassword?: string;
}

export const RegisterPage: React.FC = () => {
    const navigate = useNavigate();
    const { register, isAuthenticated, isLoading, error, clearError, user } = useAuthStore();

    const [regno, setRegno] = useState('');
    const [name, setName] = useState('');
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [localError, setLocalError] = useState('');
    const [validationErrors, setValidationErrors] = useState<ValidationErrors>({});
    const [touched, setTouched] = useState<Record<string, boolean>>({});

    // Redirect if already authenticated
    useEffect(() => {
        if (isAuthenticated && user) {
            navigate('/trade');
        }
    }, [isAuthenticated, user, navigate]);

    // Clear errors on input change
    useEffect(() => {
        if (error) clearError();
        // Use queueMicrotask to avoid synchronous setState in effect body
        queueMicrotask(() => setLocalError(''));
    }, [regno, name, password, confirmPassword, clearError, error]);

    const handleBlur = (field: string) => {
        setTouched(prev => ({ ...prev, [field]: true }));
        validateField(field);
    };

    const validateField = (field: string) => {
        const errors = { ...validationErrors };

        switch (field) {
            case 'regno':
                if (!regno.trim()) {
                    errors.regno = 'Registration number is required';
                } else if (regno.trim().length < 3) {
                    errors.regno = 'Must be at least 3 characters';
                } else {
                    delete errors.regno;
                }
                break;
            case 'name':
                if (!name.trim()) {
                    errors.name = 'Display name is required';
                } else if (name.trim().length < 2) {
                    errors.name = 'Must be at least 2 characters';
                } else {
                    delete errors.name;
                }
                break;
            case 'password':
                if (!password) {
                    errors.password = 'Password is required';
                } else if (password.length < 4) {
                    errors.password = 'Must be at least 4 characters';
                } else {
                    delete errors.password;
                }
                // Re-validate confirm password when password changes
                if (confirmPassword && password !== confirmPassword) {
                    errors.confirmPassword = 'Passwords do not match';
                } else if (confirmPassword) {
                    delete errors.confirmPassword;
                }
                break;
            case 'confirmPassword':
                if (!confirmPassword) {
                    errors.confirmPassword = 'Please confirm your password';
                } else if (password !== confirmPassword) {
                    errors.confirmPassword = 'Passwords do not match';
                } else {
                    delete errors.confirmPassword;
                }
                break;
        }

        setValidationErrors(errors);
    };

    const getPasswordStrength = (pwd: string): { level: number; label: string; color: string } => {
        if (pwd.length === 0) return { level: 0, label: '', color: '' };
        if (pwd.length < 4) return { level: 1, label: 'Weak', color: 'var(--color-danger)' };
        if (pwd.length < 8) return { level: 2, label: 'Fair', color: 'var(--color-warning)' };
        if (pwd.length >= 8 && /[A-Z]/.test(pwd) && /[0-9]/.test(pwd)) {
            return { level: 4, label: 'Strong', color: 'var(--color-success)' };
        }
        return { level: 3, label: 'Good', color: 'var(--color-info)' };
    };

    const passwordStrength = getPasswordStrength(password);

    const validateAllFields = (): boolean => {
        const errors: ValidationErrors = {};

        if (!regno.trim()) {
            errors.regno = 'Registration number is required';
        } else if (regno.trim().length < 3) {
            errors.regno = 'Must be at least 3 characters';
        }

        if (!name.trim()) {
            errors.name = 'Display name is required';
        } else if (name.trim().length < 2) {
            errors.name = 'Must be at least 2 characters';
        }

        if (!password) {
            errors.password = 'Password is required';
        } else if (password.length < 4) {
            errors.password = 'Must be at least 4 characters';
        }

        if (!confirmPassword) {
            errors.confirmPassword = 'Please confirm your password';
        } else if (password !== confirmPassword) {
            errors.confirmPassword = 'Passwords do not match';
        }

        setValidationErrors(errors);
        return Object.keys(errors).length === 0;
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        setTouched({ regno: true, name: true, password: true, confirmPassword: true });

        if (validateAllFields()) {
            register(regno.trim(), name.trim(), password);
        }
    };

    const displayError = localError || error;

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
                        <h1 className="auth-title">Create Account</h1>
                        <p className="auth-subtitle">Join the trading simulation</p>
                    </div>

                    {/* Error Message */}
                    {displayError && (
                        <div className="mt-4 p-3 rounded-lg" style={{
                            background: 'var(--color-danger-bg)',
                            border: '1px solid var(--color-danger)',
                            color: 'var(--color-danger)',
                            fontSize: 'var(--text-sm)'
                        }}>
                            {displayError}
                        </div>
                    )}

                    {/* Registration Form */}
                    <form onSubmit={handleSubmit} className="auth-form mt-6">
                        <Input
                            label="Registration Number"
                            placeholder="Your unique registration ID"
                            value={regno}
                            onChange={(e) => setRegno(e.target.value)}
                            onBlur={() => handleBlur('regno')}
                            icon={<Hash size={18} />}
                            helper={!validationErrors.regno ? "This will be used for login" : undefined}
                            error={touched.regno ? validationErrors.regno : undefined}
                        />

                        <Input
                            label="Display Name"
                            placeholder="How you'll appear to others"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            onBlur={() => handleBlur('name')}
                            icon={<User size={18} />}
                            error={touched.name ? validationErrors.name : undefined}
                        />

                        <div>
                            <Input
                                label="Password"
                                type="password"
                                placeholder="Create a secure password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                onBlur={() => handleBlur('password')}
                                icon={<Lock size={18} />}
                                error={touched.password ? validationErrors.password : undefined}
                            />
                            {password && !validationErrors.password && (
                                <div className="flex items-center gap-2 mt-2">
                                    <div className="flex-1 h-1 rounded-full" style={{ background: 'var(--bg-tertiary)' }}>
                                        <div
                                            className="h-full rounded-full transition-all"
                                            style={{
                                                width: `${passwordStrength.level * 25}%`,
                                                background: passwordStrength.color
                                            }}
                                        />
                                    </div>
                                    <span className="text-xs" style={{ color: passwordStrength.color }}>
                                        {passwordStrength.label}
                                    </span>
                                </div>
                            )}
                        </div>

                        <Input
                            label="Confirm Password"
                            type="password"
                            placeholder="Re-enter your password"
                            value={confirmPassword}
                            onChange={(e) => setConfirmPassword(e.target.value)}
                            onBlur={() => handleBlur('confirmPassword')}
                            icon={password === confirmPassword && confirmPassword && !validationErrors.confirmPassword ? <CheckCircle size={18} className="text-success" /> : <Lock size={18} />}
                            error={touched.confirmPassword ? validationErrors.confirmPassword : undefined}
                        />

                        <Button
                            type="submit"
                            variant="success"
                            size="lg"
                            loading={isLoading}
                            className="w-full mt-2"
                        >
                            Create Account
                            <ChevronRight size={18} />
                        </Button>
                    </form>

                    {/* Info Box */}
                    <div className="mt-6 p-4 rounded-lg" style={{
                        background: 'rgba(99, 102, 241, 0.1)',
                        border: '1px solid rgba(99, 102, 241, 0.2)',
                    }}>
                        <p className="text-sm" style={{ color: 'var(--text-secondary)', margin: 0 }}>
                            <strong>💰 Starting Balance:</strong> You'll receive $100,000 in virtual cash to start trading!
                        </p>
                    </div>

                    {/* Footer */}
                    <div className="auth-footer">
                        Already have an account?{' '}
                        <Link to="/login">Sign in</Link>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default RegisterPage;
