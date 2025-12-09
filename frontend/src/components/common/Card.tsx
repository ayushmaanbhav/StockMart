// ============================================
// Card Component
// ============================================

import React from 'react';

interface CardProps {
    children: React.ReactNode;
    className?: string;
    variant?: 'default' | 'elevated' | 'glass';
    hover?: boolean;
    onClick?: () => void;
}

export const Card: React.FC<CardProps> = ({
    children,
    className = '',
    variant = 'default',
    hover = false,
    onClick,
}) => {
    const classes = [
        'card',
        variant === 'elevated' && 'card-elevated',
        variant === 'glass' && 'card-glass',
        hover && 'card-hover',
        onClick && 'cursor-pointer',
        className,
    ].filter(Boolean).join(' ');

    return (
        <div className={classes} onClick={onClick}>
            {children}
        </div>
    );
};

interface CardHeaderProps {
    title: React.ReactNode;
    icon?: React.ReactNode;
    actions?: React.ReactNode;
    className?: string;
}

export const CardHeader: React.FC<CardHeaderProps> = ({
    title,
    icon,
    actions,
    className = '',
}) => {
    return (
        <div className={`card-header ${className}`}>
            <div className="card-title">
                {icon}
                {typeof title === 'string' ? <span>{title}</span> : title}
            </div>
            {actions && <div className="flex gap-2">{actions}</div>}
        </div>
    );
};

interface CardBodyProps {
    children: React.ReactNode;
    className?: string;
    noPadding?: boolean;
}

export const CardBody: React.FC<CardBodyProps> = ({
    children,
    className = '',
    noPadding = false,
}) => {
    return (
        <div className={`${noPadding ? '' : 'p-6'} ${className}`}>
            {children}
        </div>
    );
};

export default Card;
