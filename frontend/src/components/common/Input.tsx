// ============================================
// Input Component
// ============================================

import React, { forwardRef, useId } from 'react';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
    label?: string;
    error?: string;
    helper?: string;
    icon?: React.ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(({
    label,
    error,
    helper,
    icon,
    className = '',
    id,
    ...props
}, ref) => {
    // Use React's useId hook for stable, SSR-safe ID generation
    const generatedId = useId();
    const inputId = id || `input-${generatedId}`;
    const hasIcon = Boolean(icon);
    const hasError = Boolean(error);

    return (
        <div className="input-group">
            {label && (
                <label htmlFor={inputId} className="input-label">
                    {label}
                </label>
            )}

            <div className={hasIcon ? 'input-with-icon' : ''}>
                {hasIcon && <span className="input-icon">{icon}</span>}
                <input
                    ref={ref}
                    id={inputId}
                    className={`input ${hasError ? 'input-error' : ''} ${className}`}
                    {...props}
                />
            </div>

            {error && <span className="input-error-message">{error}</span>}
            {!error && helper && <span className="input-helper">{helper}</span>}
        </div>
    );
});

Input.displayName = 'Input';

export default Input;
