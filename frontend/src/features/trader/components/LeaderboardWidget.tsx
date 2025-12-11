// ============================================
// Leaderboard Widget Component
// Shows top traders by net worth
// ============================================

import React from 'react';
import { Trophy, ChevronRight, ChevronDown } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { useConfigStore } from '../../../store/configStore';
import { useAuthStore } from '../../../store/authStore';
import { Badge } from '../../../components/common';

interface LeaderboardWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

export const LeaderboardWidget: React.FC<LeaderboardWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { leaderboard } = useGameStore();
    const { user } = useAuthStore();
    const formatCurrency = useConfigStore(state => state.formatCurrency);

    return (
        <div className={`leaderboard-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <Trophy size={14} /> Leaderboard
                </span>
                <Badge variant="primary">{leaderboard.length}</Badge>
            </div>
            {!isCollapsed && (
                <div className="leaderboard-content">
                    {leaderboard.length === 0 && (
                        <div className="empty-state">Loading...</div>
                    )}
                    {leaderboard.slice(0, 10).map((entry) => {
                        const isCurrentUser = entry.name === user?.name;
                        return (
                            <div key={entry.rank} className={`leaderboard-row ${isCurrentUser ? 'current-user' : ''}`}>
                                <div className="rank-info">
                                    <span className={`rank ${entry.rank <= 3 ? 'top' : ''}`}>{entry.rank}</span>
                                    <span className="name">{entry.name}</span>
                                </div>
                                <span className="net-worth">{formatCurrency(entry.netWorth)}</span>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
};

export default LeaderboardWidget;
