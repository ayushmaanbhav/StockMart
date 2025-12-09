// ============================================
// UI State Store
// Manages theme, modals, toasts, UI preferences
// ============================================

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Theme, Toast } from '../types/ui';

interface UIState {
    // Theme
    theme: Theme;
    toggleTheme: () => void;
    setTheme: (theme: Theme) => void;

    // Modal
    activeModal: string | null;
    modalData: Record<string, unknown>;
    openModal: (id: string, data?: Record<string, unknown>) => void;
    closeModal: () => void;

    // Toasts
    toasts: Toast[];
    showToast: (toast: Omit<Toast, 'id'>) => void;
    removeToast: (id: string) => void;

    // Sidebar
    sidebarCollapsed: boolean;
    toggleSidebar: () => void;

    // Mobile
    mobileMenuOpen: boolean;
    toggleMobileMenu: () => void;
    closeMobileMenu: () => void;
}

export const useUIStore = create<UIState>()(
    persist(
        (set, get) => ({
            // Theme
            theme: 'dark',

            toggleTheme: () => {
                const newTheme = get().theme === 'dark' ? 'light' : 'dark';
                document.documentElement.setAttribute('data-theme', newTheme);
                set({ theme: newTheme });
            },

            setTheme: (theme) => {
                document.documentElement.setAttribute('data-theme', theme);
                set({ theme });
            },

            // Modal
            activeModal: null,
            modalData: {},

            openModal: (id, data = {}) => {
                set({ activeModal: id, modalData: data });
                // Prevent body scroll
                document.body.style.overflow = 'hidden';
            },

            closeModal: () => {
                set({ activeModal: null, modalData: {} });
                // Restore body scroll
                document.body.style.overflow = '';
            },

            // Toasts
            toasts: [],

            showToast: (toast) => {
                const id = crypto.randomUUID();
                const newToast = { ...toast, id };

                set((state) => ({
                    toasts: [...state.toasts, newToast]
                }));

                // Auto-remove after duration
                const duration = toast.duration || 5000;
                setTimeout(() => {
                    get().removeToast(id);
                }, duration);
            },

            removeToast: (id) => {
                set((state) => ({
                    toasts: state.toasts.filter(t => t.id !== id)
                }));
            },

            // Sidebar
            sidebarCollapsed: false,

            toggleSidebar: () => {
                set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed }));
            },

            // Mobile
            mobileMenuOpen: false,

            toggleMobileMenu: () => {
                set((state) => ({ mobileMenuOpen: !state.mobileMenuOpen }));
            },

            closeMobileMenu: () => {
                set({ mobileMenuOpen: false });
            },
        }),
        {
            name: 'ui-storage',
            partialize: (state) => ({
                theme: state.theme,
                sidebarCollapsed: state.sidebarCollapsed,
            }),
            onRehydrateStorage: () => (state) => {
                // Apply theme on load
                if (state?.theme) {
                    document.documentElement.setAttribute('data-theme', state.theme);
                }
            },
        }
    )
);

// Apply initial theme
if (typeof window !== 'undefined') {
    const storedTheme = localStorage.getItem('ui-storage');
    if (storedTheme) {
        try {
            const parsed = JSON.parse(storedTheme);
            if (parsed.state?.theme) {
                document.documentElement.setAttribute('data-theme', parsed.state.theme);
            }
        } catch {
            document.documentElement.setAttribute('data-theme', 'dark');
        }
    } else {
        document.documentElement.setAttribute('data-theme', 'dark');
    }
}

export default useUIStore;
