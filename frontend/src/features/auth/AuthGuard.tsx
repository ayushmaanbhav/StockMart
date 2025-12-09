// ============================================
// Auth Guard Component
// Protects routes requiring authentication
// ============================================

import React from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '../../store/authStore';
import type { UserRole } from '../../types/models';

interface AuthGuardProps {
    children: React.ReactNode;
    requiredRole?: UserRole;
}

export const AuthGuard: React.FC<AuthGuardProps> = ({ children, requiredRole }) => {
    const { isAuthenticated, user, isLoading } = useAuthStore();
    const location = useLocation();

    // Show loading state
    if (isLoading) {
        return (
            <div className="min-h-screen flex items-center justify-center" style={{ background: 'var(--bg-primary)' }}>
                <div className="spinner spinner-lg" />
            </div>
        );
    }

    // Redirect to login if not authenticated
    if (!isAuthenticated || !user) {
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    // Check role if required
    if (requiredRole && user.role !== requiredRole) {
        // Redirect based on actual role
        if (user.role === 'admin') {
            return <Navigate to="/admin" replace />;
        } else {
            return <Navigate to="/trade" replace />;
        }
    }

    return <>{children}</>;
};

export default AuthGuard;
