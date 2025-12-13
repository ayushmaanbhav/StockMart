// ============================================
// Configuration Module
// Handles game configuration including registration settings
// ============================================

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use tracing::{debug, info, warn};

/// Registration mode determines how new users can register
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationMode {
    /// Anyone can register with any regno
    Free,
    /// Only regnos in the allowed list can register
    Whitelist,
    /// Registration is disabled
    Disabled,
}

impl Default for RegistrationMode {
    fn default() -> Self {
        RegistrationMode::Free
    }
}

/// Currency formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConfig {
    /// Currency symbol (e.g., "$", "₹", "€", "£")
    #[serde(default = "default_currency_symbol")]
    pub symbol: String,

    /// Currency code for Intl.NumberFormat (e.g., "USD", "INR", "EUR")
    #[serde(default = "default_currency_code")]
    pub code: String,

    /// Locale for number formatting (e.g., "en-US", "en-IN", "de-DE")
    #[serde(default = "default_locale")]
    pub locale: String,

    /// Number of decimal places
    #[serde(default = "default_decimals")]
    pub decimals: u8,

    /// Symbol position: "before" or "after"
    #[serde(default = "default_symbol_position")]
    pub symbol_position: String,
}

fn default_currency_symbol() -> String {
    "$".to_string()
}

fn default_currency_code() -> String {
    "USD".to_string()
}

fn default_locale() -> String {
    "en-US".to_string()
}

fn default_decimals() -> u8 {
    2
}

fn default_symbol_position() -> String {
    "before".to_string()
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self {
            symbol: default_currency_symbol(),
            code: default_currency_code(),
            locale: default_locale(),
            decimals: 2,
            symbol_position: default_symbol_position(),
        }
    }
}

/// Game configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Registration mode
    #[serde(default)]
    pub registration_mode: RegistrationMode,

    /// List of allowed registration numbers (only used when mode is Whitelist)
    #[serde(default)]
    pub allowed_regnos: Vec<String>,

    /// Admin credentials
    #[serde(default = "default_admin_username")]
    pub admin_username: String,

    #[serde(default = "default_admin_password")]
    pub admin_password: String,

    /// Default starting money for new registrations (before game init)
    #[serde(default = "default_starting_money")]
    pub default_starting_money: i64,

    /// Enable/disable chat globally
    #[serde(default = "default_true")]
    pub chat_enabled: bool,

    /// Maximum concurrent sessions per user (0 = unlimited)
    #[serde(default = "default_one")]
    pub max_sessions_per_user: u32,

    /// Currency formatting configuration
    #[serde(default)]
    pub currency: CurrencyConfig,
}

fn default_admin_username() -> String {
    // Read from environment variable, or use a secure default that must be changed
    std::env::var("STOCKMART_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string())
}

fn default_admin_password() -> String {
    // CRITICAL: Read from environment variable
    // If not set, use a secure random-looking default that will fail login attempts
    // This ensures admin credentials MUST be configured before production use
    std::env::var("STOCKMART_ADMIN_PASSWORD")
        .unwrap_or_else(|_| {
            tracing::warn!("STOCKMART_ADMIN_PASSWORD not set! Using insecure default. Set this env var for production.");
            "change_me_in_production".to_string()
        })
}

fn default_starting_money() -> i64 {
    crate::domain::constants::user::DEFAULT_STARTING_MONEY
}

fn default_true() -> bool {
    true
}

fn default_one() -> u32 {
    1
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            registration_mode: RegistrationMode::Free,
            allowed_regnos: Vec::new(),
            admin_username: default_admin_username(),
            admin_password: default_admin_password(),
            default_starting_money: default_starting_money(),
            chat_enabled: true,
            max_sessions_per_user: 1,
            currency: CurrencyConfig::default(),
        }
    }
}

/// Configuration service for runtime access
pub struct ConfigService {
    config: RwLock<GameConfig>,
    allowed_regnos_set: RwLock<HashSet<String>>,
    config_path: String,
}

impl ConfigService {
    pub fn new(config_path: String) -> Self {
        let config = Self::load_config(&config_path);
        let allowed_set: HashSet<String> = config.allowed_regnos.iter().cloned().collect();

        info!("ConfigService initialized:");
        info!("  Registration mode: {:?}", config.registration_mode);
        info!("  Allowed regnos: {} entries", config.allowed_regnos.len());
        info!("  Max sessions per user: {}", config.max_sessions_per_user);

        Self {
            config: RwLock::new(config),
            allowed_regnos_set: RwLock::new(allowed_set),
            config_path,
        }
    }

    fn load_config(path: &str) -> GameConfig {
        let config_file = format!("{}/config.json", path);
        debug!("Loading config from: {}", config_file);

        match std::fs::read_to_string(&config_file) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(config) => {
                    info!("Config loaded successfully from {}", config_file);
                    config
                }
                Err(e) => {
                    warn!("Failed to parse config file: {}. Using defaults.", e);
                    GameConfig::default()
                }
            },
            Err(e) => {
                info!(
                    "Config file not found ({}): {}. Using defaults and creating file.",
                    config_file, e
                );
                let default_config = GameConfig::default();
                // Try to create default config file
                if let Ok(json) = serde_json::to_string_pretty(&default_config) {
                    if let Err(e) = std::fs::write(&config_file, json) {
                        warn!("Failed to write default config: {}", e);
                    } else {
                        info!("Created default config file: {}", config_file);
                    }
                }
                default_config
            }
        }
    }

    /// Check if a registration number is allowed
    pub fn is_regno_allowed(&self, regno: &str) -> Result<(), String> {
        let config = self.config.read().unwrap();

        match config.registration_mode {
            RegistrationMode::Free => {
                debug!("Registration mode: Free - allowing regno {}", regno);
                Ok(())
            }
            RegistrationMode::Whitelist => {
                let allowed = self.allowed_regnos_set.read().unwrap();
                if allowed.contains(regno) {
                    debug!("Registration mode: Whitelist - regno {} is allowed", regno);
                    Ok(())
                } else {
                    warn!(
                        "Registration mode: Whitelist - regno {} is NOT allowed",
                        regno
                    );
                    Err(
                        "Registration number not in allowed list. Contact administrator."
                            .to_string(),
                    )
                }
            }
            RegistrationMode::Disabled => {
                warn!("Registration is disabled - rejecting regno {}", regno);
                Err("Registration is currently disabled.".to_string())
            }
        }
    }

    /// Get the current config (for API responses)
    pub fn get_config(&self) -> GameConfig {
        self.config.read().unwrap().clone()
    }

    /// Get public config (safe to send to clients)
    pub fn get_public_config(&self) -> PublicConfig {
        let config = self.config.read().unwrap();
        PublicConfig {
            registration_mode: config.registration_mode.clone(),
            chat_enabled: config.chat_enabled,
            currency: config.currency.clone(),
        }
    }

    /// Reload config from disk
    pub fn reload(&self) {
        let new_config = Self::load_config(&self.config_path);
        let new_allowed: HashSet<String> = new_config.allowed_regnos.iter().cloned().collect();

        *self.config.write().unwrap() = new_config;
        *self.allowed_regnos_set.write().unwrap() = new_allowed;

        info!("Config reloaded successfully");
    }

    /// Get max sessions per user
    pub fn max_sessions_per_user(&self) -> u32 {
        self.config.read().unwrap().max_sessions_per_user
    }

    /// Get default starting money
    pub fn default_starting_money(&self) -> i64 {
        self.config.read().unwrap().default_starting_money
    }

    /// Verify admin credentials
    pub fn verify_admin(&self, username: &str, password: &str) -> bool {
        let config = self.config.read().unwrap();
        config.admin_username == username && config.admin_password == password
    }
}

/// Public config that's safe to send to clients
#[derive(Debug, Clone, Serialize)]
pub struct PublicConfig {
    pub registration_mode: RegistrationMode,
    pub chat_enabled: bool,
    pub currency: CurrencyConfig,
}

/// Frontend UI constants sent to clients
/// These values configure the frontend behavior
#[derive(Debug, Clone, Serialize)]
pub struct FrontendConstants {
    /// Data limits
    pub limits: FrontendLimits,
    /// UI display settings
    pub ui: FrontendUISettings,
    /// Animation settings
    pub animation: FrontendAnimationSettings,
    /// Trading parameters
    pub trading: FrontendTradingSettings,
    /// Game initialization defaults (for admin UI)
    pub game_defaults: FrontendGameDefaults,
    /// Polling intervals
    pub polling: FrontendPollingSettings,
    /// Company form validation
    pub company_form: CompanyFormSettings,
    /// Sector options
    pub sectors: Vec<String>,
    /// UI labels and text
    pub labels: FrontendLabels,
    /// Validation rules
    pub validation: ValidationRules,
}

/// UI Labels and text strings
#[derive(Debug, Clone, Serialize)]
pub struct FrontendLabels {
    /// Application branding
    pub app_name: String,
    pub app_tagline: String,

    /// Auth page labels
    pub auth: AuthLabels,

    /// Trading labels
    pub trading: TradingLabels,

    /// Admin labels
    pub admin: AdminLabels,

    /// Common labels
    pub common: CommonLabels,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthLabels {
    pub login_title: String,
    pub login_subtitle: String,
    pub register_title: String,
    pub register_subtitle: String,
    pub regno_label: String,
    pub regno_placeholder: String,
    pub password_label: String,
    pub password_placeholder: String,
    pub confirm_password_label: String,
    pub name_label: String,
    pub name_placeholder: String,
    pub login_button: String,
    pub register_button: String,
    pub no_account_text: String,
    pub has_account_text: String,
    pub starting_balance_info: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TradingLabels {
    pub order_book: String,
    pub portfolio: String,
    pub open_orders: String,
    pub trade_history: String,
    pub buy: String,
    pub sell: String,
    pub short: String,
    pub market: String,
    pub limit: String,
    pub quantity: String,
    pub price: String,
    pub total: String,
    pub bids: String,
    pub asks: String,
    pub no_bids: String,
    pub no_asks: String,
    pub no_trades: String,
    pub no_orders: String,
    pub cancel_order: String,
    pub cancel_all: String,
    pub confirm_order: String,
    pub gtc: String,
    pub gtc_full: String,
    pub ioc: String,
    pub ioc_full: String,
    pub short_margin_warning: String,
    pub market_order_info: String,
    pub no_liquidity: String,
    pub positions: String,
    pub holdings: String,
    pub cash: String,
    pub net_worth: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminLabels {
    pub dashboard: String,
    pub game_control: String,
    pub traders: String,
    pub companies: String,
    pub diagnostics: String,
    pub market_open: String,
    pub market_closed: String,
    pub open_market: String,
    pub close_market: String,
    pub initialize_game: String,
    pub ban_trader: String,
    pub unban_trader: String,
    pub mute_trader: String,
    pub unmute_trader: String,
    pub create_company: String,
    pub mark_bankrupt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommonLabels {
    pub loading: String,
    pub error: String,
    pub success: String,
    pub cancel: String,
    pub confirm: String,
    pub save: String,
    pub refresh: String,
    pub search: String,
    pub no_results: String,
    pub connected: String,
    pub disconnected: String,
    pub reconnecting: String,
    pub live: String,
    pub offline: String,
}

/// Validation rules for forms
#[derive(Debug, Clone, Serialize)]
pub struct ValidationRules {
    pub regno_min_length: usize,
    pub regno_max_length: usize,
    pub password_min_length: usize,
    pub name_min_length: usize,
    pub name_max_length: usize,
    pub chat_message_max_length: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendLimits {
    pub trades_history: usize,
    pub candles_per_symbol: usize,
    pub chat_messages: usize,
    pub news_items: usize,
    pub trade_history_page_size: usize,
    pub stock_trades_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendUISettings {
    pub orderbook_depth: usize,
    pub leaderboard_entries: usize,
    pub chat_messages_visible: usize,
    pub trade_history_widget: usize,
    pub stock_trades_widget: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendAnimationSettings {
    pub news_ticker_base_duration: u32,
    pub news_ticker_per_item: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendTradingSettings {
    pub default_order_qty: u32,
    pub short_margin_percent: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendGameDefaults {
    pub target_networth: i64,
    pub shares_per_trader: u64,
    pub trading_start_time: String,
    pub trading_end_time: String,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_duration: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontendPollingSettings {
    pub dashboard_metrics_interval: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompanyFormSettings {
    pub symbol_max_length: usize,
    pub volatility_min: f64,
    pub volatility_max: f64,
    pub volatility_step: f64,
    pub default_total_shares: u64,
    pub default_initial_price: i64,
}

impl Default for FrontendConstants {
    fn default() -> Self {
        use crate::domain::constants;

        Self {
            limits: FrontendLimits {
                trades_history: constants::market::MAX_CANDLES_HISTORY / 10, // 100
                candles_per_symbol: 200,
                chat_messages: constants::chat::SYNC_MESSAGE_COUNT,
                news_items: constants::news::SYNC_NEWS_COUNT / 2, // 10
                trade_history_page_size: constants::pagination::DEFAULT_PAGE_SIZE as usize * 2, // 50
                stock_trades_count: constants::market::DEFAULT_RECENT_TRADES_COUNT,
            },
            ui: FrontendUISettings {
                orderbook_depth: constants::websocket::DEFAULT_DEPTH_LEVELS,
                leaderboard_entries: 10,
                chat_messages_visible: 20,
                trade_history_widget: 20,
                stock_trades_widget: 15,
            },
            animation: FrontendAnimationSettings {
                news_ticker_base_duration: 20,
                news_ticker_per_item: 5,
            },
            trading: FrontendTradingSettings {
                default_order_qty: 10,
                short_margin_percent: constants::trading::SHORT_MARGIN_PERCENT as u32,
            },
            game_defaults: FrontendGameDefaults {
                target_networth: 200_000 * constants::PRICE_SCALE,
                shares_per_trader: constants::user::DEFAULT_SHARES_PER_COMPANY,
                trading_start_time: "09:00".to_string(),
                trading_end_time: "16:00".to_string(),
                circuit_breaker_threshold: constants::circuit_breaker::THRESHOLD_PERCENT as u32,
                circuit_breaker_duration: constants::circuit_breaker::HALT_DURATION_SECS as u32,
            },
            polling: FrontendPollingSettings {
                dashboard_metrics_interval: 10000,
            },
            company_form: CompanyFormSettings {
                symbol_max_length: 5,
                volatility_min: 0.1,
                volatility_max: 0.5,
                volatility_step: 0.05,
                default_total_shares: constants::admin::DEFAULT_IPO_SHARES,
                default_initial_price: constants::admin::BASE_STOCK_PRICE,
            },
            sectors: vec![
                "Tech".to_string(),
                "Finance".to_string(),
                "Healthcare".to_string(),
                "Energy".to_string(),
                "Consumer".to_string(),
                "Industrial".to_string(),
            ],
            labels: FrontendLabels::default(),
            validation: ValidationRules::default(),
        }
    }
}

impl Default for FrontendLabels {
    fn default() -> Self {
        Self {
            app_name: "StockMart".to_string(),
            app_tagline: "Virtual Trading Simulation".to_string(),
            auth: AuthLabels {
                login_title: "Welcome Back".to_string(),
                login_subtitle: "Sign in to continue trading".to_string(),
                register_title: "Create Account".to_string(),
                register_subtitle: "Join the trading simulation".to_string(),
                regno_label: "Registration Number".to_string(),
                regno_placeholder: "Enter your registration ID".to_string(),
                password_label: "Password".to_string(),
                password_placeholder: "Enter your password".to_string(),
                confirm_password_label: "Confirm Password".to_string(),
                name_label: "Display Name".to_string(),
                name_placeholder: "How you'll appear to others".to_string(),
                login_button: "Sign In".to_string(),
                register_button: "Create Account".to_string(),
                no_account_text: "Don't have an account?".to_string(),
                has_account_text: "Already have an account?".to_string(),
                starting_balance_info: "You'll receive $100,000 in virtual cash to start trading!"
                    .to_string(),
            },
            trading: TradingLabels {
                order_book: "Order Book".to_string(),
                portfolio: "Portfolio".to_string(),
                open_orders: "Open Orders".to_string(),
                trade_history: "Trade History".to_string(),
                buy: "Buy".to_string(),
                sell: "Sell".to_string(),
                short: "Short".to_string(),
                market: "Market".to_string(),
                limit: "Limit".to_string(),
                quantity: "Quantity".to_string(),
                price: "Price".to_string(),
                total: "Total".to_string(),
                bids: "Bids".to_string(),
                asks: "Asks".to_string(),
                no_bids: "No bids".to_string(),
                no_asks: "No asks".to_string(),
                no_trades: "No trades yet".to_string(),
                no_orders: "No open orders".to_string(),
                cancel_order: "Cancel".to_string(),
                cancel_all: "Cancel All".to_string(),
                confirm_order: "Confirm Order".to_string(),
                gtc: "GTC".to_string(),
                gtc_full: "Good 'Til Cancelled".to_string(),
                ioc: "IOC".to_string(),
                ioc_full: "Immediate or Cancel".to_string(),
                short_margin_warning: "Short selling requires 150% margin coverage".to_string(),
                market_order_info: "Market orders execute immediately at best available price"
                    .to_string(),
                no_liquidity: "No Market Liquidity".to_string(),
                positions: "Positions".to_string(),
                holdings: "Holdings".to_string(),
                cash: "Cash".to_string(),
                net_worth: "Net Worth".to_string(),
            },
            admin: AdminLabels {
                dashboard: "Dashboard".to_string(),
                game_control: "Game Control".to_string(),
                traders: "Traders".to_string(),
                companies: "Companies".to_string(),
                diagnostics: "Diagnostics".to_string(),
                market_open: "Market Open".to_string(),
                market_closed: "Market Closed".to_string(),
                open_market: "Open Market".to_string(),
                close_market: "Close Market".to_string(),
                initialize_game: "Initialize Game".to_string(),
                ban_trader: "Ban Trader".to_string(),
                unban_trader: "Unban Trader".to_string(),
                mute_trader: "Mute".to_string(),
                unmute_trader: "Unmute".to_string(),
                create_company: "Create Company".to_string(),
                mark_bankrupt: "Mark Bankrupt".to_string(),
            },
            common: CommonLabels {
                loading: "Loading...".to_string(),
                error: "Error".to_string(),
                success: "Success".to_string(),
                cancel: "Cancel".to_string(),
                confirm: "Confirm".to_string(),
                save: "Save".to_string(),
                refresh: "Refresh".to_string(),
                search: "Search".to_string(),
                no_results: "No results found".to_string(),
                connected: "Connected".to_string(),
                disconnected: "Disconnected".to_string(),
                reconnecting: "Reconnecting...".to_string(),
                live: "Live".to_string(),
                offline: "Offline".to_string(),
            },
        }
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            regno_min_length: 3,
            regno_max_length: 20,
            password_min_length: 4,
            name_min_length: 2,
            name_max_length: 50,
            chat_message_max_length: 500,
        }
    }
}

impl ConfigService {
    /// Get frontend constants
    pub fn get_frontend_constants(&self) -> FrontendConstants {
        FrontendConstants::default()
    }
}
