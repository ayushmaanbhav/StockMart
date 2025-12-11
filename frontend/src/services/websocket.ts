// ============================================
// WebSocket Service
// Enhanced with typed messages, session management,
// auto-reconnect with exponential backoff
// ============================================

import type { ClientMessage, ServerMessage, ServerMessageType } from '../types/api';
import { loggers } from '../utils';

const log = loggers.websocket;

type MessageHandler<T = unknown> = (payload: T) => void;
type ConnectionHandler = () => void;

interface WebSocketConfig {
    url?: string;
    reconnectAttempts?: number;
    reconnectBaseDelay?: number;
    heartbeatInterval?: number;
}

interface QueuedMessage {
    message: ClientMessage;
    timestamp: number;
}

class WebSocketService {
    private ws: WebSocket | null = null;
    private url: string;
    private handlers: Map<string, Set<MessageHandler>> = new Map();
    private connectionHandlers: {
        connect: Set<ConnectionHandler>;
        disconnect: Set<ConnectionHandler>;
    } = {
            connect: new Set(),
            disconnect: new Set(),
        };

    private isConnected = false;
    private reconnectAttempts = 0;
    private maxReconnectAttempts: number;
    private reconnectBaseDelay: number;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
    private heartbeatInterval: number;

    private messageQueue: QueuedMessage[] = [];
    private sessionToken: string | null = null;
    private autoReconnect = true;

    constructor(config: WebSocketConfig = {}) {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const host = window.location.host;

        this.url = config.url || `${protocol}//${host}/ws`;
        this.maxReconnectAttempts = config.reconnectAttempts ?? 10;
        this.reconnectBaseDelay = config.reconnectBaseDelay ?? 1000;
        this.heartbeatInterval = config.heartbeatInterval ?? 30000;
    }

    // === Connection Management ===

    connect(): void {
        if (this.ws?.readyState === WebSocket.OPEN) {
            log.debug('Already connected');
            return;
        }

        if (this.ws?.readyState === WebSocket.CONNECTING) {
            log.debug('Connection in progress');
            return;
        }

        log.debug('Connecting to', this.url);
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            log.debug('Connected');
            this.isConnected = true;
            const isReconnect = this.reconnectAttempts > 0;
            this.reconnectAttempts = 0;

            // Notify handlers
            this.connectionHandlers.connect.forEach(handler => handler());

            // Start heartbeat
            this.startHeartbeat();

            // Flush message queue
            this.flushMessageQueue();

            // Re-authenticate if we have a token (auth triggers full state sync on backend)
            if (this.sessionToken) {
                this.send({ type: 'Auth', payload: { token: this.sessionToken } });
            } else if (isReconnect) {
                // If no session token but this is a reconnect, request full state sync
                // This handles guest/non-authenticated users
                this.send({ type: 'RequestSync', payload: {} });
            }
        };

        this.ws.onclose = (event) => {
            log.debug('Disconnected', event.code, event.reason);
            this.isConnected = false;
            this.stopHeartbeat();

            // Notify handlers
            this.connectionHandlers.disconnect.forEach(handler => handler());

            // Schedule reconnect if appropriate
            if (this.autoReconnect && event.code !== 1000) {
                this.scheduleReconnect();
            }
        };

        this.ws.onerror = (error) => {
            log.error('Error:', error);
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data) as ServerMessage;
                log.debug('Received:', data.type, data.payload);
                this.dispatch(data.type, data.payload);
            } catch (e) {
                log.error('Failed to parse message:', e);
            }
        };
    }

    disconnect(): void {
        this.autoReconnect = false;
        this.stopHeartbeat();

        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }

        if (this.ws) {
            this.ws.close(1000, 'Client disconnect');
            this.ws = null;
        }

        this.isConnected = false;
    }

    private scheduleReconnect(): void {
        if (this.reconnectTimer) return;
        if (this.reconnectAttempts >= this.maxReconnectAttempts) {
            log.error('Max reconnect attempts reached');
            return;
        }

        // Exponential backoff with jitter
        const delay = Math.min(
            this.reconnectBaseDelay * Math.pow(2, this.reconnectAttempts) + Math.random() * 1000,
            30000 // Max 30 seconds
        );

        log.debug(`Reconnecting in ${Math.round(delay)}ms (attempt ${this.reconnectAttempts + 1})`);

        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.reconnectAttempts++;
            this.connect();
        }, delay);
    }

    private startHeartbeat(): void {
        this.stopHeartbeat();
        this.heartbeatTimer = setInterval(() => {
            if (this.isConnected) {
                this.send({ type: 'Ping', payload: {} });
            }
        }, this.heartbeatInterval);
    }

    private stopHeartbeat(): void {
        if (this.heartbeatTimer) {
            clearInterval(this.heartbeatTimer);
            this.heartbeatTimer = null;
        }
    }

    // === Message Sending ===

    send(message: ClientMessage): boolean {
        if (this.ws?.readyState === WebSocket.OPEN) {
            log.debug('Sending:', message.type, message.payload);
            this.ws.send(JSON.stringify(message));
            return true;
        }

        // Queue message for later
        log.debug('Queuing message:', message.type);
        this.messageQueue.push({ message, timestamp: Date.now() });

        // Clean old messages (older than 30 seconds)
        const now = Date.now();
        this.messageQueue = this.messageQueue.filter(m => now - m.timestamp < 30000);

        return false;
    }

    private flushMessageQueue(): void {
        log.debug(`Flushing ${this.messageQueue.length} queued messages`);
        while (this.messageQueue.length > 0) {
            const queued = this.messageQueue.shift();
            if (queued) {
                this.send(queued.message);
            }
        }
    }

    // === Event Handling ===

    on<T = unknown>(type: ServerMessageType | 'connected' | 'disconnected', handler: MessageHandler<T>): () => void {
        if (type === 'connected') {
            this.connectionHandlers.connect.add(handler as ConnectionHandler);
            return () => this.connectionHandlers.connect.delete(handler as ConnectionHandler);
        }

        if (type === 'disconnected') {
            this.connectionHandlers.disconnect.add(handler as ConnectionHandler);
            return () => this.connectionHandlers.disconnect.delete(handler as ConnectionHandler);
        }

        if (!this.handlers.has(type)) {
            this.handlers.set(type, new Set());
        }
        this.handlers.get(type)!.add(handler as MessageHandler);

        return () => {
            this.handlers.get(type)?.delete(handler as MessageHandler);
        };
    }

    off(type: string, handler: MessageHandler): void {
        this.handlers.get(type)?.delete(handler);
    }

    private dispatch(type: string, payload: unknown): void {
        this.handlers.get(type)?.forEach(handler => {
            try {
                handler(payload);
            } catch (e) {
                console.error(`[WS] Error in handler for ${type}:`, e);
            }
        });
    }

    // === Session Management ===

    setSessionToken(token: string): void {
        this.sessionToken = token;
        localStorage.setItem('session_token', token);
    }

    clearSessionToken(): void {
        this.sessionToken = null;
        localStorage.removeItem('session_token');
    }

    getSessionToken(): string | null {
        if (!this.sessionToken) {
            this.sessionToken = localStorage.getItem('session_token');
        }
        return this.sessionToken;
    }

    // === Status ===

    getConnectionStatus(): boolean {
        return this.isConnected;
    }
}

// Singleton instance
export const websocketService = new WebSocketService();

// Auto-connect when module loads
if (typeof window !== 'undefined') {
    websocketService.connect();
}

export default websocketService;
