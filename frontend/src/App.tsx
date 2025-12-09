// ============================================
// StockMart App
// Main Application with Routing
// ============================================

import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';

// Layouts
import { TraderLayout, AdminLayout } from './components/layout';

// Features
import { LoginPage, RegisterPage, AuthGuard } from './features/auth';
import { TradingDeskPage } from './features/trader/TradingDeskPage';
import { AdminDashboardPage } from './features/admin/DashboardPage';
import { GameControlPage } from './features/admin/GameControlPage';
import { TradersPage } from './features/admin/TradersPage';
import { CompaniesPage } from './features/admin/CompaniesPage';
import { DiagnosticsPage } from './features/admin/DiagnosticsPage';
import { TradesPage } from './features/admin/TradesPage';
import { OrdersPage } from './features/admin/OrdersPage';
import { OrderbookPage } from './features/admin/OrderbookPage';

// Common Components
import { ErrorBoundary } from './components/common';

// Styles
import './styles/index.css';
import './styles/components.css';
import './styles/layout.css';
import './styles/trading.css';

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          {/* Public Routes */}
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />

          {/* Trader Routes */}
          <Route
            path="/trade"
            element={
              <AuthGuard requiredRole="trader">
                <TraderLayout />
              </AuthGuard>
            }
          >
            <Route index element={<TradingDeskPage />} />
          </Route>

          {/* Admin Routes */}
          <Route
            path="/admin"
            element={
              <AuthGuard requiredRole="admin">
                <AdminLayout />
              </AuthGuard>
            }
          >
            <Route index element={<AdminDashboardPage />} />
            <Route path="game" element={<GameControlPage />} />
            <Route path="traders" element={<TradersPage />} />
            <Route path="companies" element={<CompaniesPage />} />
            <Route path="trades" element={<TradesPage />} />
            <Route path="orders" element={<OrdersPage />} />
            <Route path="orderbook" element={<OrderbookPage />} />
            <Route path="diagnostics" element={<DiagnosticsPage />} />
          </Route>

          {/* Default Redirect */}
          <Route path="/" element={<Navigate to="/login" replace />} />
          <Route path="*" element={<Navigate to="/login" replace />} />
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default App;
