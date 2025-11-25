

type MessageHandler = (payload: any) => void;

class WebSocketService {
    private ws: WebSocket | null = null;
    private url: string = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`;
    private handlers: Map<string, Set<MessageHandler>> = new Map();
    private isConnected: boolean = false;
    private reconnectTimer: any = null;

    constructor() {
        this.connect();
    }

    private connect() {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            console.log('Connected to WebSocket');
            this.isConnected = true;
            this.dispatch('connected', {});
            // Auto-login for dev
            this.send('Auth', { token: '1' });
        };

        this.ws.onclose = () => {
            console.log('Disconnected from WebSocket');
            this.isConnected = false;
            this.dispatch('disconnected', {});
            this.scheduleReconnect();
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket Error:', error);
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                console.log('Received:', data);
                this.dispatch(data.type, data.payload);
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };
    }

    private scheduleReconnect() {
        if (this.reconnectTimer) return;
        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, 3000);
    }

    public send(type: string, payload: any) {
        if (this.ws && this.isConnected) {
            this.ws.send(JSON.stringify({ type, payload }));
        } else {
            console.warn('WebSocket not connected, cannot send:', type);
        }
    }

    public on(type: string, handler: MessageHandler) {
        if (!this.handlers.has(type)) {
            this.handlers.set(type, new Set());
        }
        this.handlers.get(type)?.add(handler);
    }

    public off(type: string, handler: MessageHandler) {
        this.handlers.get(type)?.delete(handler);
    }

    private dispatch(type: string, payload: any) {
        this.handlers.get(type)?.forEach(handler => handler(payload));
    }
}

export const socketService = new WebSocketService();
