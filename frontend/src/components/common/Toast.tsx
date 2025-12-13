// ============================================
// Toast Component
// ============================================

import React from 'react';
import { X, CheckCircle, XCircle, AlertTriangle, Info } from 'lucide-react';
import { useUIStore } from '../../store/uiStore';
import type { Toast as ToastType } from '../../types/ui';

const icons = {
    success: <CheckCircle size={20} className="text-success" />,
    error: <XCircle size={20} className="text-danger" />,
    warning: <AlertTriangle size={20} className="text-warning" />,
    info: <Info size={20} style={{ color: 'var(--color-info)' }} />,
};

interface ToastItemProps {
    toast: ToastType;
}

const ToastItem: React.FC<ToastItemProps> = ({ toast }) => {
    const { removeToast } = useUIStore();

    return (
        <div className={`toast toast-${toast.type}`}>
            <span className="toast-icon">{icons[toast.type]}</span>
            <div className="toast-content">
                {toast.title && <div className="toast-title">{toast.title}</div>}
                <div className={toast.title ? "toast-message" : "toast-title"}>{toast.message}</div>
            </div>
            <button className="toast-close" onClick={() => removeToast(toast.id)}>
                <X size={16} />
            </button>
        </div>
    );
};

export const ToastContainer: React.FC = () => {
    const { toasts } = useUIStore();

    if (toasts.length === 0) return null;

    return (
        <div className="toast-container">
            {toasts.map((toast) => (
                <ToastItem key={toast.id} toast={toast} />
            ))}
        </div>
    );
};

export default ToastContainer;
