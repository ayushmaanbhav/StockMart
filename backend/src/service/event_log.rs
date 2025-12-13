// ============================================
// Event Logger Service
// Logs all game state changes and transactions to JSON
// ============================================

#![allow(dead_code)] // Logger API includes methods for various event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{error, info};

/// Types of events that can be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum GameEvent {
    // User events
    UserRegistered {
        user_id: u64,
        regno: String,
        name: String,
        initial_cash: i64,
        initial_portfolio_value: i64,
    },
    UserLogin {
        user_id: u64,
        regno: String,
        name: String,
    },

    // Order events
    OrderPlaced {
        order_id: u64,
        user_id: u64,
        symbol: String,
        side: String,
        order_type: String,
        qty: u64,
        price: i64,
        time_in_force: String,
    },
    OrderCancelled {
        order_id: u64,
        user_id: u64,
        symbol: String,
        reason: String,
    },
    OrderRejected {
        user_id: u64,
        symbol: String,
        side: String,
        qty: u64,
        price: i64,
        reason: String,
    },

    // Trade events
    TradeExecuted {
        trade_id: u64,
        symbol: String,
        buyer_id: u64,
        seller_id: u64,
        qty: u64,
        price: i64,
        buyer_order_id: u64,
        seller_order_id: u64,
    },

    // Portfolio events
    PortfolioUpdate {
        user_id: u64,
        cash: i64,
        locked_cash: i64,
        margin_locked: i64,
        positions: Vec<PositionSnapshot>,
        net_worth: i64,
    },

    // Market events
    MarketOpened,
    MarketClosed,
    CircuitBreakerTriggered {
        symbol: String,
        reason: String,
        halted_until: i64,
    },

    // Admin events
    GameInitialized {
        num_traders: usize,
        starting_money: i64,
        share_allocation_per_trader: i64,
    },
    GameReset {
        reason: String,
    },
    CompanyCreated {
        symbol: String,
        name: String,
        sector: String,
        initial_price: i64,
    },
    CompanyBankrupt {
        symbol: String,
    },
    VolatilityChanged {
        symbol: String,
        old_volatility: i64,
        new_volatility: i64,
    },
    TraderBanned {
        user_id: u64,
        reason: String,
    },
    TraderUnbanned {
        user_id: u64,
    },
    TraderChatMuted {
        user_id: u64,
    },
    TraderChatUnmuted {
        user_id: u64,
    },

    // Chat events (optional, can be disabled)
    ChatMessage {
        user_id: u64,
        username: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub symbol: String,
    pub qty: u64,
    pub short_qty: u64,
    pub locked_qty: u64,
    pub average_buy_price: i64,
}

/// A logged event with timestamp and sequence number
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    pub timestamp_unix: i64,
    #[serde(flatten)]
    pub event: GameEvent,
}

/// Event logger that writes to a JSON Lines file
pub struct EventLogger {
    writer: Mutex<Option<BufWriter<File>>>,
    sequence: Mutex<u64>,
    log_path: PathBuf,
    log_chat: bool,
}

impl EventLogger {
    pub fn new(data_dir: &str, log_chat: bool) -> Self {
        let log_path = PathBuf::from(data_dir).join("game_events.jsonl");

        let writer = match Self::open_log_file(&log_path) {
            Ok(w) => {
                info!("Event logger initialized: {}", log_path.display());
                Some(w)
            }
            Err(e) => {
                error!("Failed to open event log file: {}. Logging disabled.", e);
                None
            }
        };

        Self {
            writer: Mutex::new(writer),
            sequence: Mutex::new(0),
            log_path,
            log_chat,
        }
    }

    fn open_log_file(path: &PathBuf) -> std::io::Result<BufWriter<File>> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(BufWriter::new(file))
    }

    /// Log an event
    pub fn log(&self, event: GameEvent) {
        // Skip chat messages if logging is disabled
        if !self.log_chat {
            if let GameEvent::ChatMessage { .. } = event {
                return;
            }
        }

        let mut seq_guard = self.sequence.lock().unwrap();
        *seq_guard += 1;
        let seq = *seq_guard;
        drop(seq_guard);

        let now = Utc::now();
        let entry = LogEntry {
            seq,
            timestamp: now,
            timestamp_unix: now.timestamp(),
            event,
        };

        let mut writer_guard = self.writer.lock().unwrap();
        if let Some(ref mut writer) = *writer_guard {
            match serde_json::to_string(&entry) {
                Ok(json) => {
                    if let Err(e) = writeln!(writer, "{}", json) {
                        error!("Failed to write event log: {}", e);
                    }
                    // Flush after each write to ensure durability
                    if let Err(e) = writer.flush() {
                        error!("Failed to flush event log: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize event: {}", e);
                }
            }
        }
    }

    /// Rotate the log file (create a new one with timestamp)
    pub fn rotate(&self) -> std::io::Result<()> {
        let mut writer_guard = self.writer.lock().unwrap();

        // Close current file
        if let Some(mut w) = writer_guard.take() {
            w.flush()?;
        }

        // Rename old file with timestamp
        if self.log_path.exists() {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let backup_name = format!("game_events_{}.jsonl", timestamp);
            let backup_path = self.log_path.parent().unwrap().join(backup_name);
            std::fs::rename(&self.log_path, backup_path)?;
        }

        // Open new file
        *writer_guard = Some(Self::open_log_file(&self.log_path)?);

        // Reset sequence
        let mut seq_guard = self.sequence.lock().unwrap();
        *seq_guard = 0;

        info!("Event log rotated");
        Ok(())
    }

    // === Convenience methods for common events ===

    pub fn log_user_registered(
        &self,
        user_id: u64,
        regno: &str,
        name: &str,
        initial_cash: i64,
        initial_portfolio_value: i64,
    ) {
        self.log(GameEvent::UserRegistered {
            user_id,
            regno: regno.to_string(),
            name: name.to_string(),
            initial_cash,
            initial_portfolio_value,
        });
    }

    pub fn log_user_login(&self, user_id: u64, regno: &str, name: &str) {
        self.log(GameEvent::UserLogin {
            user_id,
            regno: regno.to_string(),
            name: name.to_string(),
        });
    }

    pub fn log_order_placed(
        &self,
        order_id: u64,
        user_id: u64,
        symbol: &str,
        side: &str,
        order_type: &str,
        qty: u64,
        price: i64,
        time_in_force: &str,
    ) {
        self.log(GameEvent::OrderPlaced {
            order_id,
            user_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            order_type: order_type.to_string(),
            qty,
            price,
            time_in_force: time_in_force.to_string(),
        });
    }

    pub fn log_order_cancelled(&self, order_id: u64, user_id: u64, symbol: &str, reason: &str) {
        self.log(GameEvent::OrderCancelled {
            order_id,
            user_id,
            symbol: symbol.to_string(),
            reason: reason.to_string(),
        });
    }

    pub fn log_order_rejected(
        &self,
        user_id: u64,
        symbol: &str,
        side: &str,
        qty: u64,
        price: i64,
        reason: &str,
    ) {
        self.log(GameEvent::OrderRejected {
            user_id,
            symbol: symbol.to_string(),
            side: side.to_string(),
            qty,
            price,
            reason: reason.to_string(),
        });
    }

    pub fn log_trade_executed(
        &self,
        trade_id: u64,
        symbol: &str,
        buyer_id: u64,
        seller_id: u64,
        qty: u64,
        price: i64,
        buyer_order_id: u64,
        seller_order_id: u64,
    ) {
        self.log(GameEvent::TradeExecuted {
            trade_id,
            symbol: symbol.to_string(),
            buyer_id,
            seller_id,
            qty,
            price,
            buyer_order_id,
            seller_order_id,
        });
    }

    pub fn log_portfolio_update(
        &self,
        user_id: u64,
        cash: i64,
        locked_cash: i64,
        margin_locked: i64,
        positions: Vec<PositionSnapshot>,
        net_worth: i64,
    ) {
        self.log(GameEvent::PortfolioUpdate {
            user_id,
            cash,
            locked_cash,
            margin_locked,
            positions,
            net_worth,
        });
    }

    pub fn log_market_opened(&self) {
        self.log(GameEvent::MarketOpened);
    }

    pub fn log_market_closed(&self) {
        self.log(GameEvent::MarketClosed);
    }

    pub fn log_circuit_breaker(&self, symbol: &str, reason: &str, halted_until: i64) {
        self.log(GameEvent::CircuitBreakerTriggered {
            symbol: symbol.to_string(),
            reason: reason.to_string(),
            halted_until,
        });
    }

    pub fn log_game_initialized(
        &self,
        num_traders: usize,
        starting_money: i64,
        share_allocation_per_trader: i64,
    ) {
        self.log(GameEvent::GameInitialized {
            num_traders,
            starting_money,
            share_allocation_per_trader,
        });
    }

    pub fn log_game_reset(&self, reason: &str) {
        self.log(GameEvent::GameReset {
            reason: reason.to_string(),
        });
    }

    pub fn log_company_created(&self, symbol: &str, name: &str, sector: &str, initial_price: i64) {
        self.log(GameEvent::CompanyCreated {
            symbol: symbol.to_string(),
            name: name.to_string(),
            sector: sector.to_string(),
            initial_price,
        });
    }

    pub fn log_company_bankrupt(&self, symbol: &str) {
        self.log(GameEvent::CompanyBankrupt {
            symbol: symbol.to_string(),
        });
    }

    pub fn log_volatility_changed(&self, symbol: &str, old_volatility: i64, new_volatility: i64) {
        self.log(GameEvent::VolatilityChanged {
            symbol: symbol.to_string(),
            old_volatility,
            new_volatility,
        });
    }

    pub fn log_trader_banned(&self, user_id: u64, reason: &str) {
        self.log(GameEvent::TraderBanned {
            user_id,
            reason: reason.to_string(),
        });
    }

    pub fn log_trader_unbanned(&self, user_id: u64) {
        self.log(GameEvent::TraderUnbanned { user_id });
    }

    pub fn log_trader_chat_muted(&self, user_id: u64) {
        self.log(GameEvent::TraderChatMuted { user_id });
    }

    pub fn log_trader_chat_unmuted(&self, user_id: u64) {
        self.log(GameEvent::TraderChatUnmuted { user_id });
    }

    pub fn log_chat_message(&self, user_id: u64, username: &str, message: &str) {
        self.log(GameEvent::ChatMessage {
            user_id,
            username: username.to_string(),
            message: message.to_string(),
        });
    }
}
