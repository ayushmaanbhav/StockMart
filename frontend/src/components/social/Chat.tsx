import React, { useState, useEffect, useRef } from 'react';
import { useGameStore } from '../../store/gameStore';
import { useAuthStore } from '../../store/authStore';
import { Send, MessageSquare } from 'lucide-react';

export const Chat = () => {
    const { chatMessages, sendChatMessage } = useGameStore();
    const { user } = useAuthStore();
    const [message, setMessage] = useState('');
    const messagesContainerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        // Scroll within the container instead of using scrollIntoView
        // to avoid affecting the entire page scroll position
        if (messagesContainerRef.current) {
            messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
        }
    }, [chatMessages]);

    const handleSend = (e: React.FormEvent) => {
        e.preventDefault();
        if (message.trim()) {
            sendChatMessage(message.trim());
            setMessage('');
        }
    };

    return (
        <div className="bg-gray-800 rounded-xl shadow-lg flex flex-col h-[400px]">
            <div className="p-4 border-b border-gray-700 flex items-center gap-2">
                <MessageSquare className="text-blue-400" size={20} />
                <h2 className="text-lg font-semibold text-gray-100">Global Chat</h2>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar" ref={messagesContainerRef}>
                {chatMessages.length === 0 ? (
                    <p className="text-gray-500 text-sm text-center italic mt-10">No messages yet. Say hello!</p>
                ) : (
                    chatMessages.map((msg: any) => (
                        <div key={msg.id} className={`flex flex-col ${msg.username === user?.name ? 'items-end' : 'items-start'}`}>
                            <div className={`max-w-[80%] rounded-lg p-2 ${msg.username === user?.name
                                    ? 'bg-blue-600 text-white rounded-br-none'
                                    : 'bg-gray-700 text-gray-200 rounded-bl-none'
                                }`}>
                                <p className="text-xs font-bold mb-1 opacity-75">{msg.username}</p>
                                <p className="text-sm break-words">{msg.message}</p>
                            </div>
                            <span className="text-[10px] text-gray-500 mt-1">
                                {new Date(msg.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                            </span>
                        </div>
                    ))
                )}
            </div>

            <form onSubmit={handleSend} className="p-4 border-t border-gray-700 flex gap-2">
                <input
                    type="text"
                    value={message}
                    onChange={(e) => setMessage(e.target.value)}
                    placeholder="Type a message..."
                    className="flex-1 bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 text-sm text-gray-100 focus:outline-none focus:border-blue-500"
                />
                <button
                    type="submit"
                    disabled={!message.trim()}
                    className="bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-white p-2 rounded-lg transition-colors"
                >
                    <Send size={18} />
                </button>
            </form>
        </div>
    );
};
