// ============================================
// UI Types
// ============================================

export type Theme = 'light' | 'dark';

export interface Toast {
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    title?: string;
    message: string;
    duration?: number;
}

export interface ModalConfig {
    id: string;
    title?: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    closable?: boolean;
}

export interface WizardStep {
    id: string;
    title: string;
    description?: string;
    isValid?: boolean;
    isComplete?: boolean;
}

export interface TabItem {
    id: string;
    label: string;
    icon?: React.ReactNode;
    badge?: string | number;
}

export interface SelectOption {
    value: string;
    label: string;
    disabled?: boolean;
}

export interface TableColumn<T> {
    key: keyof T | string;
    header: string;
    width?: string;
    align?: 'left' | 'center' | 'right';
    sortable?: boolean;
    render?: (value: unknown, row: T) => React.ReactNode;
}

export interface PaginationState {
    page: number;
    pageSize: number;
    total: number;
}

// Navigation
export interface NavItem {
    path: string;
    label: string;
    icon?: React.ReactNode;
    children?: NavItem[];
    adminOnly?: boolean;
}

// Forms
export interface FormField {
    name: string;
    label: string;
    type: 'text' | 'password' | 'number' | 'email' | 'select' | 'checkbox';
    placeholder?: string;
    required?: boolean;
    validation?: (value: unknown) => string | undefined;
    options?: SelectOption[];
}

// Trading
export interface QuickTradeForm {
    symbol: string;
    side: 'Buy' | 'Sell' | 'Short';
    orderType: 'Market' | 'Limit';
    quantity: number;
    price: number;
    timeInForce: 'GTC' | 'IOC';
}

export interface OrderWizardState {
    step: number;
    symbol: string | null;
    side: 'Buy' | 'Sell' | 'Short' | null;
    orderType: 'Market' | 'Limit' | null;
    quantity: number | null;
    price: number | null;
    timeInForce: 'GTC' | 'IOC';
    confirmed: boolean;
}
