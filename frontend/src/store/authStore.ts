// ============================================
// Authentication Store
// ============================================

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User, UserRole } from '../types/models';
import websocketService from '../services/websocket';

interface AuthState {
    // State
    user: User | null;
    isAuthenticated: boolean;
    isLoading: boolean;
    error: string | null;

    // Actions
    login: (regno: string, password: string) => Promise<void>;
    loginAdmin: (username: string, password: string) => Promise<void>;
    register: (regno: string, name: string, password: string) => Promise<void>;
    logout: () => void;
    setUser: (user: Partial<User>) => void;
    clearError: () => void;

    // Internal
    _handleAuthSuccess: (userId: number, name: string, role: UserRole) => void;
    _handleAuthFailed: (reason: string) => void;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,

            login: async (regno: string, password: string) => {
                set({ isLoading: true, error: null });

                // Use Login message for username+password authentication
                websocketService.send({
                    type: 'Login',
                    payload: { regno, password }
                });

                // Wait for response via WebSocket handlers
                // The actual success/fail is handled by _handleAuthSuccess/_handleAuthFailed
            },

            loginAdmin: async (username: string, password: string) => {
                set({ isLoading: true, error: null });

                // Use Login message for admin authentication - role is validated server-side
                websocketService.send({
                    type: 'Login',
                    payload: { regno: username, password }
                });
            },

            register: async (regno: string, name: string, password: string) => {
                set({ isLoading: true, error: null });

                websocketService.send({
                    type: 'Register',
                    payload: { regno, name, password }
                });
            },

            logout: () => {
                websocketService.clearSessionToken();
                set({
                    user: null,
                    isAuthenticated: false,
                    isLoading: false,
                    error: null
                });
            },

            setUser: (updates: Partial<User>) => {
                const currentUser = get().user;
                if (currentUser) {
                    set({
                        user: { ...currentUser, ...updates }
                    });
                }
            },

            clearError: () => set({ error: null }),

            _handleAuthSuccess: (userId: number, name: string, role: UserRole) => {
                websocketService.setSessionToken(String(userId));
                set({
                    user: {
                        id: userId,
                        regno: '',
                        name,
                        role,
                        money: 0,
                        lockedMoney: 0,
                        marginLocked: 0,
                        netWorth: 0,
                        portfolio: [],
                        chatEnabled: true,
                        banned: false,
                        createdAt: Date.now()
                    },
                    isAuthenticated: true,
                    isLoading: false,
                    error: null
                });
            },

            _handleAuthFailed: (reason: string) => {
                set({
                    isLoading: false,
                    error: reason
                });
            }
        }),
        {
            name: 'auth-storage',
            partialize: (state) => ({
                // Only persist minimal auth info
                user: state.user ? { id: state.user.id, name: state.user.name, role: state.user.role } : null,
                isAuthenticated: state.isAuthenticated
            })
        }
    )
);

// === WebSocket Event Bindings ===

websocketService.on('AuthSuccess', (payload: { user_id: number; name: string; role: string }) => {
    // Use role from server response
    const role: UserRole = payload.role === 'admin' ? 'admin' : 'trader';
    useAuthStore.getState()._handleAuthSuccess(payload.user_id, payload.name, role);
});

websocketService.on('AuthFailed', (payload: { reason: string }) => {
    useAuthStore.getState()._handleAuthFailed(payload.reason);
});

websocketService.on('RegisterSuccess', (payload: { user_id: number; name: string; role: string }) => {
    // Use role from server response
    const role: UserRole = payload.role === 'admin' ? 'admin' : 'trader';
    useAuthStore.getState()._handleAuthSuccess(payload.user_id, payload.name, role);
});

websocketService.on('RegisterFailed', (payload: { reason: string }) => {
    useAuthStore.getState()._handleAuthFailed(payload.reason);
});

// Auto-login on reconnect if we have stored auth
websocketService.on('connected', () => {
    const token = websocketService.getSessionToken();
    if (token) {
        websocketService.send({ type: 'Auth', payload: { token } });
    }
});

export default useAuthStore;
