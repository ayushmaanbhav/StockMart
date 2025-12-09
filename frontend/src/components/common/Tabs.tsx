// ============================================
// Tabs Component
// ============================================

import React, { useState } from 'react';

interface Tab {
    id: string;
    label: string;
    icon?: React.ReactNode;
    badge?: string | number;
}

interface TabsProps {
    tabs: Tab[];
    activeTab?: string;
    onChange?: (tabId: string) => void;
    variant?: 'default' | 'trading';
    className?: string;
}

export const Tabs: React.FC<TabsProps> = ({
    tabs,
    activeTab: controlledActive,
    onChange,
    variant = 'default',
    className = '',
}) => {
    const [internalActive, setInternalActive] = useState(tabs[0]?.id);
    const activeTab = controlledActive ?? internalActive;

    const handleChange = (tabId: string) => {
        if (!controlledActive) {
            setInternalActive(tabId);
        }
        onChange?.(tabId);
    };

    return (
        <div className={`tabs ${className}`}>
            {tabs.map((tab) => {
                const isActive = activeTab === tab.id;
                const tradingClass = variant === 'trading' && tab.id.toLowerCase();

                return (
                    <button
                        key={tab.id}
                        type="button"
                        className={`tab ${isActive ? 'active' : ''} ${tradingClass && isActive ? `tab-${tradingClass}` : ''}`}
                        onClick={() => handleChange(tab.id)}
                    >
                        {tab.icon}
                        <span>{tab.label}</span>
                        {tab.badge && (
                            <span className="badge badge-primary" style={{ marginLeft: '4px', fontSize: '10px' }}>
                                {tab.badge}
                            </span>
                        )}
                    </button>
                );
            })}
        </div>
    );
};

export default Tabs;
