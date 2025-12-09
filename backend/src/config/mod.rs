// ============================================
// Configuration Module
// Handles game configuration including registration settings
// ============================================

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
    "admin".to_string()
}

fn default_admin_password() -> String {
    "admin".to_string()
}

fn default_starting_money() -> i64 {
    100_000 * 10_000 // $100,000 in scaled format
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
            Ok(contents) => {
                match serde_json::from_str(&contents) {
                    Ok(config) => {
                        info!("Config loaded successfully from {}", config_file);
                        config
                    }
                    Err(e) => {
                        warn!("Failed to parse config file: {}. Using defaults.", e);
                        GameConfig::default()
                    }
                }
            }
            Err(e) => {
                info!("Config file not found ({}): {}. Using defaults and creating file.", config_file, e);
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
                    warn!("Registration mode: Whitelist - regno {} is NOT allowed", regno);
                    Err("Registration number not in allowed list. Contact administrator.".to_string())
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
