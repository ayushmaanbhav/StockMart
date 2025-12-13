// ============================================
// Toast Hook
// ============================================

import { useUIStore } from '../../store/uiStore';

// Convenience hook for showing toasts
export const useToast = () => {
    const { showToast } = useUIStore();

    return {
        success: (message: string, title?: string) =>
            showToast({ type: 'success', message, title }),
        error: (message: string, title?: string) =>
            showToast({ type: 'error', message, title }),
        warning: (message: string, title?: string) =>
            showToast({ type: 'warning', message, title }),
        info: (message: string, title?: string) =>
            showToast({ type: 'info', message, title }),
    };
};
