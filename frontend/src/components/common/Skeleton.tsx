// ============================================
// Skeleton Loading Components
// Placeholder UI while content is loading
// ============================================

import React from 'react';

interface SkeletonProps {
    width?: string | number;
    height?: string | number;
    variant?: 'text' | 'circular' | 'rectangular' | 'rounded';
    className?: string;
    animation?: 'pulse' | 'wave' | 'none';
}

export const Skeleton: React.FC<SkeletonProps> = ({
    width,
    height,
    variant = 'text',
    className = '',
    animation = 'pulse',
}) => {
    const style: React.CSSProperties = {
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
    };

    return (
        <div
            className={`skeleton skeleton-${variant} skeleton-${animation} ${className}`}
            style={style}
        />
    );
};

// Pre-built skeleton patterns for common use cases

export const SkeletonText: React.FC<{ lines?: number; className?: string }> = ({
    lines = 3,
    className = '',
}) => (
    <div className={`skeleton-text ${className}`}>
        {Array.from({ length: lines }).map((_, i) => (
            <Skeleton
                key={i}
                variant="text"
                width={i === lines - 1 ? '60%' : '100%'}
            />
        ))}
    </div>
);

export const SkeletonCard: React.FC<{ className?: string }> = ({ className = '' }) => (
    <div className={`skeleton-card ${className}`}>
        <Skeleton variant="rectangular" height={120} />
        <div className="skeleton-card-content">
            <Skeleton variant="text" width="80%" />
            <Skeleton variant="text" width="60%" />
        </div>
    </div>
);

export const SkeletonTable: React.FC<{ rows?: number; cols?: number; className?: string }> = ({
    rows = 5,
    cols = 4,
    className = '',
}) => (
    <div className={`skeleton-table ${className}`}>
        <div className="skeleton-table-header">
            {Array.from({ length: cols }).map((_, i) => (
                <Skeleton key={i} variant="text" height={20} />
            ))}
        </div>
        {Array.from({ length: rows }).map((_, rowIndex) => (
            <div key={rowIndex} className="skeleton-table-row">
                {Array.from({ length: cols }).map((_, colIndex) => (
                    <Skeleton key={colIndex} variant="text" height={16} />
                ))}
            </div>
        ))}
    </div>
);

// Deterministic heights for skeleton chart bars (avoids layout shifts)
const SKELETON_BAR_HEIGHTS = [65, 82, 48, 91, 57, 74, 43, 88, 62, 79, 51, 95];

export const SkeletonChart: React.FC<{ height?: number; className?: string }> = ({
    height = 200,
    className = '',
}) => (
    <div className={`skeleton-chart ${className}`} style={{ height }}>
        <div className="skeleton-chart-bars">
            {SKELETON_BAR_HEIGHTS.map((barHeight, i) => (
                <Skeleton
                    key={i}
                    variant="rectangular"
                    width={20}
                    height={barHeight}
                />
            ))}
        </div>
    </div>
);

export const SkeletonOrderBook: React.FC<{ className?: string }> = ({ className = '' }) => (
    <div className={`skeleton-orderbook ${className}`}>
        <div className="skeleton-orderbook-side">
            <Skeleton variant="text" width="40%" height={14} />
            {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="skeleton-orderbook-row">
                    <Skeleton variant="text" width="45%" height={12} />
                    <Skeleton variant="text" width="35%" height={12} />
                </div>
            ))}
        </div>
        <div className="skeleton-orderbook-side">
            <Skeleton variant="text" width="40%" height={14} />
            {Array.from({ length: 6 }).map((_, i) => (
                <div key={i} className="skeleton-orderbook-row">
                    <Skeleton variant="text" width="45%" height={12} />
                    <Skeleton variant="text" width="35%" height={12} />
                </div>
            ))}
        </div>
    </div>
);

export const SkeletonLeaderboard: React.FC<{ rows?: number; className?: string }> = ({
    rows = 10,
    className = '',
}) => (
    <div className={`skeleton-leaderboard ${className}`}>
        {Array.from({ length: rows }).map((_, i) => (
            <div key={i} className="skeleton-leaderboard-row">
                <Skeleton variant="circular" width={24} height={24} />
                <Skeleton variant="text" width="50%" height={14} />
                <Skeleton variant="text" width="25%" height={14} />
            </div>
        ))}
    </div>
);

export const SkeletonPortfolio: React.FC<{ className?: string }> = ({ className = '' }) => (
    <div className={`skeleton-portfolio ${className}`}>
        <div className="skeleton-portfolio-stats">
            <div className="skeleton-stat">
                <Skeleton variant="text" width="60%" height={12} />
                <Skeleton variant="text" width="80%" height={20} />
            </div>
            <div className="skeleton-stat">
                <Skeleton variant="text" width="60%" height={12} />
                <Skeleton variant="text" width="80%" height={20} />
            </div>
        </div>
        <div className="skeleton-portfolio-holdings">
            {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="skeleton-holding-row">
                    <Skeleton variant="text" width="30%" height={14} />
                    <Skeleton variant="text" width="25%" height={14} />
                    <Skeleton variant="text" width="20%" height={14} />
                </div>
            ))}
        </div>
    </div>
);

export default Skeleton;
