// ============================================
// Open Orders Widget Component
// Shows and manages open orders
// ============================================

import React from 'react';
import { BarChart2, ChevronDown, ChevronRight, X, XCircle } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';
import { Badge } from '../../../components/common';

interface OpenOrdersWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

export const OpenOrdersWidget: React.FC<OpenOrdersWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { openOrders, cancelOrder } = useGameStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    const handleCancelAll = (e: React.MouseEvent) => {
        e.stopPropagation(); // Prevent toggle when clicking cancel all
        openOrders.forEach(order => {
            cancelOrder(order.symbol, order.id);
        });
    };

    return (
        <div className={`open-orders-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <BarChart2 size={14} /> Open Orders
                </span>
                <div className="widget-header-actions">
                    {openOrders.length > 0 && (
                        <>
                            <Badge variant="primary">{openOrders.length}</Badge>
                            <button
                                className="cancel-all-btn"
                                onClick={handleCancelAll}
                                title="Cancel All Orders"
                            >
                                <XCircle size={14} />
                                Cancel All
                            </button>
                        </>
                    )}
                </div>
            </div>
            {!isCollapsed && (
                <div className="orders-list">
                    {openOrders.length === 0 && <div className="empty-state">No open orders</div>}
                    {openOrders.map((order) => (
                        <div key={order.id} className="order-row">
                            <div className="order-info">
                                <Badge variant={order.side === 'Buy' ? 'buy' : order.side === 'Sell' ? 'sell' : 'short'}>
                                    {order.side}
                                </Badge>
                                <span className="symbol">{order.symbol}</span>
                            </div>
                            <div className="order-details">
                                <span className="qty-price">{order.qty} @ {formatCurrency(order.price)}</span>
                                <span className="filled">{order.filledQty || 0} filled</span>
                            </div>
                            <button className="cancel-btn" onClick={() => cancelOrder(order.symbol, order.id)}>
                                <X size={14} />
                            </button>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};

export default OpenOrdersWidget;
