// ============================================
// Utilities Barrel Export
// ============================================

export { createLogger, loggers } from './logger';

// Quick logger for inline usage
// Usage: logger.debug('Context', 'message', data)
export const logger = {
    debug: (context: string, message: string, ...args: unknown[]) => {
        if (import.meta.env.DEV) {
            console.log(`[${context}]`, message, ...args);
        }
    },
    info: (context: string, message: string, ...args: unknown[]) => {
        if (import.meta.env.DEV) {
            console.info(`[${context}]`, message, ...args);
        }
    },
    warn: (context: string, message: string, ...args: unknown[]) => {
        console.warn(`[${context}]`, message, ...args);
    },
    error: (context: string, message: string, ...args: unknown[]) => {
        console.error(`[${context}]`, message, ...args);
    },
};
