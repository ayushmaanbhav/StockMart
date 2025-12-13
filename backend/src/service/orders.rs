//! Orders service for tracking open orders.

use crate::domain::models::{Order, OrderId, OrderStatus, Quantity, UserId};
use crate::domain::ui_models::OpenOrderUI;
use dashmap::DashMap;

/// Service for tracking all open orders across all users.
/// Provides fast lookup by user_id, order_id, and symbol for state sync.
pub struct OrdersService {
    /// user_id -> Vec<Order> (open orders only)
    user_orders: DashMap<UserId, Vec<Order>>,
    /// order_id -> (user_id, symbol) for quick lookup
    order_index: DashMap<OrderId, (UserId, String)>,
}

impl OrdersService {
    pub fn new() -> Self {
        Self {
            user_orders: DashMap::new(),
            order_index: DashMap::new(),
        }
    }

    /// Add a new order to tracking
    pub fn add_order(&self, order: Order) {
        let user_id = order.user_id;
        let order_id = order.id;
        let symbol = order.symbol.clone();

        // Add to user's orders
        self.user_orders
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(order);

        // Add to index
        self.order_index.insert(order_id, (user_id, symbol));
    }

    /// Update an order's filled quantity and status
    pub fn update_order(&self, order_id: OrderId, filled_qty: Quantity, status: OrderStatus) {
        if let Some((user_id, _)) = self.order_index.get(&order_id).map(|r| r.clone()) {
            if let Some(mut orders) = self.user_orders.get_mut(&user_id) {
                if let Some(order) = orders.iter_mut().find(|o| o.id == order_id) {
                    order.filled_qty = filled_qty;
                    order.status = status;
                }
            }
        }
    }

    /// Remove an order from tracking (when filled or cancelled)
    pub fn remove_order(&self, order_id: OrderId) -> Option<Order> {
        if let Some((_, (user_id, _))) = self.order_index.remove(&order_id) {
            if let Some(mut orders) = self.user_orders.get_mut(&user_id) {
                if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                    return Some(orders.remove(pos));
                }
            }
        }
        None
    }

    /// Get all open orders for a user as UI-ready structs
    pub fn get_user_orders(&self, user_id: UserId) -> Vec<OpenOrderUI> {
        self.user_orders
            .get(&user_id)
            .map(|orders| {
                orders
                    .iter()
                    .filter(|o| o.is_active())
                    .map(order_to_ui)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all open orders across all users
    #[allow(dead_code)] // API method for admin use
    pub fn get_all_orders(&self) -> Vec<OpenOrderUI> {
        let mut all_orders = Vec::new();
        for entry in self.user_orders.iter() {
            for order in entry.value().iter() {
                if order.is_active() {
                    all_orders.push(order_to_ui(order));
                }
            }
        }
        // Sort by timestamp descending (newest first)
        all_orders.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_orders
    }

    /// Get all open orders for a specific symbol
    #[allow(dead_code)] // API method for symbol-specific queries
    pub fn get_orders_by_symbol(&self, symbol: &str) -> Vec<OpenOrderUI> {
        let mut symbol_orders = Vec::new();
        for entry in self.user_orders.iter() {
            for order in entry.value().iter() {
                if order.symbol == symbol && order.is_active() {
                    symbol_orders.push(order_to_ui(order));
                }
            }
        }
        // Sort by timestamp descending
        symbol_orders.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        symbol_orders
    }

    /// Get order count for a user
    #[allow(dead_code)] // API method for user stats
    pub fn get_user_order_count(&self, user_id: UserId) -> usize {
        self.user_orders
            .get(&user_id)
            .map(|orders| orders.iter().filter(|o| o.is_active()).count())
            .unwrap_or(0)
    }

    /// Clear all orders for a user (used when user disconnects or for reset)
    #[allow(dead_code)] // API method for session cleanup
    pub fn clear_user_orders(&self, user_id: UserId) {
        if let Some((_, orders)) = self.user_orders.remove(&user_id) {
            for order in orders {
                self.order_index.remove(&order.id);
            }
        }
    }

    /// Clear all orders (used for game reset)
    pub fn clear_all(&self) {
        self.user_orders.clear();
        self.order_index.clear();
    }

    /// Check if an order exists
    #[allow(dead_code)] // API method for order validation
    pub fn order_exists(&self, order_id: OrderId) -> bool {
        self.order_index.contains_key(&order_id)
    }

    /// Get a specific order by ID
    pub fn get_order(&self, order_id: OrderId) -> Option<Order> {
        if let Some((user_id, _)) = self.order_index.get(&order_id).map(|r| r.clone()) {
            if let Some(orders) = self.user_orders.get(&user_id) {
                return orders.iter().find(|o| o.id == order_id).cloned();
            }
        }
        None
    }

    /// Get all open orders for admin with user info
    pub fn get_all_orders_admin(
        &self,
        symbol_filter: Option<&str>,
        user_names: &std::collections::HashMap<u64, String>,
    ) -> Vec<crate::domain::ui_models::AdminOpenOrderUI> {
        let mut all_orders = Vec::new();
        for entry in self.user_orders.iter() {
            let user_id = *entry.key();
            let user_name = user_names
                .get(&user_id)
                .cloned()
                .unwrap_or_else(|| format!("User#{}", user_id));

            for order in entry.value().iter() {
                if !order.is_active() {
                    continue;
                }
                if let Some(sym) = symbol_filter {
                    if order.symbol != sym {
                        continue;
                    }
                }
                all_orders.push(crate::domain::ui_models::AdminOpenOrderUI {
                    order_id: order.id,
                    user_id,
                    user_name: user_name.clone(),
                    symbol: order.symbol.clone(),
                    side: order.side,
                    order_type: order.order_type,
                    qty: order.qty,
                    filled_qty: order.filled_qty,
                    remaining_qty: order.remaining_qty(),
                    price: order.price,
                    status: order.status,
                    timestamp: order.timestamp,
                    time_in_force: order.time_in_force,
                });
            }
        }
        // Sort by timestamp descending (newest first)
        all_orders.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        all_orders
    }

    /// Get total open orders count
    pub fn get_total_open_orders_count(&self) -> usize {
        let mut count = 0;
        for entry in self.user_orders.iter() {
            count += entry.value().iter().filter(|o| o.is_active()).count();
        }
        count
    }
}

/// Convert internal Order to UI-ready OpenOrderUI
fn order_to_ui(order: &Order) -> OpenOrderUI {
    OpenOrderUI {
        order_id: order.id,
        symbol: order.symbol.clone(),
        side: order.side,
        order_type: order.order_type,
        qty: order.qty,
        filled_qty: order.filled_qty,
        remaining_qty: order.remaining_qty(),
        price: order.price,
        status: order.status,
        timestamp: order.timestamp,
        time_in_force: order.time_in_force,
    }
}

impl Default for OrdersService {
    fn default() -> Self {
        Self::new()
    }
}
