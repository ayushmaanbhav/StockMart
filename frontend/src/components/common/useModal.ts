// ============================================
// Modal Hook
// ============================================

import { useUIStore } from '../../store/uiStore';

// Hook for easy modal control
export const useModal = (id: string) => {
    const { openModal, closeModal, activeModal, modalData } = useUIStore();

    return {
        isOpen: activeModal === id,
        data: modalData,
        open: (data?: Record<string, unknown>) => openModal(id, data),
        close: closeModal,
    };
};
