// ============================================
// Error Boundary Component
// Catches JavaScript errors in child components
// ============================================

import React from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';

interface ErrorBoundaryProps {
    children: React.ReactNode;
    fallback?: React.ReactNode;
}

interface ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
    errorInfo: React.ErrorInfo | null;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props);
        this.state = {
            hasError: false,
            error: null,
            errorInfo: null,
        };
    }

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error('ErrorBoundary caught an error:', error, errorInfo);
        this.setState({ errorInfo });
    }

    handleReload = () => {
        window.location.reload();
    };

    handleGoHome = () => {
        window.location.href = '/';
    };

    handleReset = () => {
        this.setState({
            hasError: false,
            error: null,
            errorInfo: null,
        });
    };

    render() {
        if (this.state.hasError) {
            if (this.props.fallback) {
                return this.props.fallback;
            }

            return (
                <div className="error-boundary">
                    <div className="error-boundary-content">
                        <div className="error-boundary-icon">
                            <AlertTriangle size={48} />
                        </div>
                        <h1 className="error-boundary-title">Something went wrong</h1>
                        <p className="error-boundary-message">
                            An unexpected error occurred. Please try refreshing the page or returning to the home page.
                        </p>

                        {import.meta.env.DEV && this.state.error && (
                            <details className="error-boundary-details">
                                <summary>Error Details</summary>
                                <pre className="error-boundary-stack">
                                    {this.state.error.toString()}
                                    {this.state.errorInfo?.componentStack}
                                </pre>
                            </details>
                        )}

                        <div className="error-boundary-actions">
                            <button
                                className="error-boundary-btn secondary"
                                onClick={this.handleGoHome}
                            >
                                <Home size={16} />
                                Go Home
                            </button>
                            <button
                                className="error-boundary-btn primary"
                                onClick={this.handleReload}
                            >
                                <RefreshCw size={16} />
                                Refresh Page
                            </button>
                        </div>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

// Smaller inline error boundary for widget-level errors
interface WidgetErrorBoundaryState {
    hasError: boolean;
}

export class WidgetErrorBoundary extends React.Component<
    { children: React.ReactNode; widgetName?: string },
    WidgetErrorBoundaryState
> {
    constructor(props: { children: React.ReactNode; widgetName?: string }) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError(): WidgetErrorBoundaryState {
        return { hasError: true };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error(`Widget error (${this.props.widgetName || 'unknown'}):`, error, errorInfo);
    }

    handleRetry = () => {
        this.setState({ hasError: false });
    };

    render() {
        if (this.state.hasError) {
            return (
                <div className="widget-error">
                    <AlertTriangle size={24} />
                    <p>Failed to load {this.props.widgetName || 'widget'}</p>
                    <button onClick={this.handleRetry}>
                        <RefreshCw size={14} />
                        Retry
                    </button>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
