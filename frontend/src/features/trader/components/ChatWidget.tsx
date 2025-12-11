// ============================================
// Chat Widget Component
// Real-time chat with other traders
// ============================================

import React, { useRef, useEffect } from 'react';
import { MessageCircle, ChevronRight, ChevronDown } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { Badge } from '../../../components/common';

interface ChatWidgetProps {
    isCollapsed: boolean;
    onToggle: () => void;
}

export const ChatWidget: React.FC<ChatWidgetProps> = ({ isCollapsed, onToggle }) => {
    const { chatMessages, sendChatMessage } = useGameStore();
    const [message, setMessage] = React.useState('');
    const messagesContainerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        // Scroll within the container instead of using scrollIntoView
        // to avoid affecting the entire page scroll position
        if (messagesContainerRef.current) {
            messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
        }
    }, [chatMessages]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (message.trim()) {
            sendChatMessage(message.trim());
            setMessage('');
        }
    };

    return (
        <div className={`chat-widget ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="widget-header clickable" onClick={onToggle}>
                <span className="widget-title">
                    {isCollapsed ? <ChevronRight size={14} /> : <ChevronDown size={14} />}
                    <MessageCircle size={14} /> Chat
                </span>
                <Badge variant="primary">{chatMessages.length}</Badge>
            </div>
            {!isCollapsed && (
                <>
                    <div className="chat-messages" ref={messagesContainerRef}>
                        {chatMessages.length === 0 && (
                            <div className="empty-state" style={{ fontSize: '12px' }}>No messages yet</div>
                        )}
                        {chatMessages.slice(-20).map((msg) => (
                            <div key={msg.id} className="chat-message" style={{ fontSize: '12px', marginBottom: '4px' }}>
                                <span className="chat-username" style={{ fontWeight: 600, color: 'var(--color-primary)' }}>{msg.username}: </span>
                                <span className="chat-text">{msg.message}</span>
                            </div>
                        ))}
                    </div>
                    <form onSubmit={handleSubmit} className="chat-input-form" style={{ padding: '8px', borderTop: '1px solid var(--border-secondary)' }}>
                        <input
                            type="text"
                            placeholder="Type a message..."
                            value={message}
                            onChange={(e) => setMessage(e.target.value)}
                            maxLength={500}
                            style={{ flex: 1, padding: '6px 8px', fontSize: '12px' }}
                        />
                        <button type="submit" style={{ padding: '6px 12px', fontSize: '12px' }}>Send</button>
                    </form>
                </>
            )}
        </div>
    );
};

export default ChatWidget;
