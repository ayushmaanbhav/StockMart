// ============================================
// Logger Utility
// Centralized logging with environment-aware output
// ============================================

const isDev = import.meta.env.DEV;

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerConfig {
    enabled: boolean;
    prefix: string;
}

const defaultConfig: LoggerConfig = {
    enabled: isDev,
    prefix: '',
};

/**
 * Create a namespaced logger
 * @param namespace - Logger namespace (e.g., 'GameStore', 'WebSocket')
 */
export const createLogger = (namespace: string) => {
    const config: LoggerConfig = {
        ...defaultConfig,
        prefix: `[${namespace}]`,
    };

    const log = (level: LogLevel, ...args: unknown[]) => {
        if (!config.enabled) return;

        const method = level === 'debug' ? 'log' : level;
        console[method](config.prefix, ...args);
    };

    return {
        debug: (...args: unknown[]) => log('debug', ...args),
        info: (...args: unknown[]) => log('info', ...args),
        warn: (...args: unknown[]) => log('warn', ...args),
        error: (...args: unknown[]) => log('error', ...args),

        /** Enable/disable this logger */
        setEnabled: (enabled: boolean) => {
            config.enabled = enabled;
        },
    };
};

// Pre-configured loggers for common modules
export const loggers = {
    gameStore: createLogger('GameStore'),
    websocket: createLogger('WebSocket'),
    auth: createLogger('Auth'),
    config: createLogger('Config'),
    admin: createLogger('Admin'),
    trading: createLogger('Trading'),
    chat: createLogger('Chat'),
};

// Default export for quick usage
export default createLogger;
