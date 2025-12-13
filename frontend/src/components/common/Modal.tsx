// ============================================
// Modal Component
// ============================================

import React, { useEffect, useCallback } from 'react';
import { X } from 'lucide-react';
import { useUIStore } from '../../store/uiStore';

interface ModalProps {
    id?: string;
    title?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    closable?: boolean;
    children: React.ReactNode;
    footer?: React.ReactNode;
    onClose?: () => void;
    // For inline/standalone modals (not using store)
    isOpen?: boolean;
}

export const Modal: React.FC<ModalProps> = ({
    id,
    title,
    size = 'md',
    closable = true,
    children,
    footer,
    onClose,
    isOpen: isOpenProp,
}) => {
    const { activeModal, closeModal } = useUIStore();
    // Support both store-based and prop-based open state
    const isOpen = isOpenProp !== undefined ? isOpenProp : (id ? activeModal === id : false);

    const handleClose = useCallback(() => {
        if (closable) {
            closeModal();
            onClose?.();
        }
    }, [closable, closeModal, onClose]);

    // Handle escape key
    useEffect(() => {
        if (!isOpen) return;

        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && closable) {
                handleClose();
            }
        };

        document.addEventListener('keydown', handleEscape);
        return () => document.removeEventListener('keydown', handleEscape);
    }, [isOpen, closable, handleClose]);

    if (!isOpen) return null;

    const sizeClass = {
        sm: 'max-w-sm',
        md: 'max-w-md',
        lg: 'max-w-lg',
        xl: 'max-w-xl',
    }[size];

    return (
        <div className="modal-backdrop" onClick={handleClose}>
            <div
                className={`modal ${sizeClass}`}
                style={{ maxWidth: size === 'sm' ? '380px' : size === 'lg' ? '600px' : size === 'xl' ? '800px' : '500px' }}
                onClick={(e) => e.stopPropagation()}
                role="dialog"
                aria-modal="true"
                aria-labelledby={title ? `modal-title-${id}` : undefined}
            >
                {title && (
                    <div className="modal-header">
                        <h2 id={`modal-title-${id}`} className="modal-title">
                            {title}
                        </h2>
                        {closable && (
                            <button className="modal-close" onClick={handleClose} aria-label="Close">
                                <X size={20} />
                            </button>
                        )}
                    </div>
                )}

                <div className="modal-body">
                    {children}
                </div>

                {footer && (
                    <div className="modal-footer">
                        {footer}
                    </div>
                )}
            </div>
        </div>
    );
};

export default Modal;
