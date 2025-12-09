// ============================================
// Badge Component
// ============================================

import React from 'react';

interface BadgeProps {
    children: React.ReactNode;
    variant?: 'primary' | 'success' | 'danger' | 'warning' | 'buy' | 'sell' | 'short';
    pulse?: boolean;
    className?: string;
}

export const Badge: React.FC<BadgeProps> = ({
    children,
    variant = 'primary',
    pulse = false,
    className = '',
}) => {
    const classes = [
        'badge',
        `badge-${variant}`,
        pulse && 'badge-pulse',
        className,
    ].filter(Boolean).join(' ');

    return <span className={classes}>{children}</span>;
};

export default Badge;
