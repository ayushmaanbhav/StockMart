//! Domain error hierarchy.
//!
//! This module provides a comprehensive error type system for the domain layer,
//! following the principle of making invalid states unrepresentable and
//! providing rich error context.

use thiserror::Error;

// =============================================================================
// DOMAIN ERROR (TOP-LEVEL)
// =============================================================================

/// Top-level domain error that encompasses all domain-specific errors.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Trading-related errors
    #[error("Trading error: {0}")]
    Trading(#[from] TradingError),

    /// User-related errors
    #[error("User error: {0}")]
    User(#[from] UserError),

    /// Market-related errors
    #[error("Market error: {0}")]
    Market(#[from] MarketError),

    /// Repository/persistence errors
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// Configuration errors
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
}

// =============================================================================
// TRADING ERRORS
// =============================================================================

/// Errors related to trading operations.
#[derive(Debug, Error, Clone)]
pub enum TradingError {
    /// Market is currently closed
    #[error("Market is currently closed")]
    MarketClosed,

    /// Insufficient funds for the operation
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: i64, available: i64 },

    /// Insufficient shares for the operation
    #[error("Insufficient shares: required {required}, available {available}")]
    InsufficientShares { required: u64, available: u64 },

    /// Insufficient margin for short selling
    #[error("Insufficient margin: required {required}, available {available}")]
    InsufficientMargin { required: i64, available: i64 },

    /// Order not found
    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: u64 },

    /// Invalid order parameters
    #[error("Invalid order: {reason}")]
    InvalidOrder { reason: String },

    /// Symbol not found
    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },

    /// Trading halted for symbol (circuit breaker)
    #[error("Trading halted for {symbol} until {until}")]
    TradingHalted { symbol: String, until: i64 },

    /// Order ownership mismatch
    #[error("Order does not belong to user")]
    NotOrderOwner,
}

impl TradingError {
    /// Get an error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            TradingError::MarketClosed => "MARKET_CLOSED",
            TradingError::InsufficientFunds { .. } => "INSUFFICIENT_FUNDS",
            TradingError::InsufficientShares { .. } => "INSUFFICIENT_SHARES",
            TradingError::InsufficientMargin { .. } => "INSUFFICIENT_MARGIN",
            TradingError::OrderNotFound { .. } => "ORDER_NOT_FOUND",
            TradingError::InvalidOrder { .. } => "INVALID_ORDER",
            TradingError::SymbolNotFound { .. } => "SYMBOL_NOT_FOUND",
            TradingError::TradingHalted { .. } => "TRADING_HALTED",
            TradingError::NotOrderOwner => "NOT_ORDER_OWNER",
        }
    }
}

// =============================================================================
// USER ERRORS
// =============================================================================

/// Errors related to user operations.
#[derive(Debug, Error, Clone)]
#[allow(dead_code)] // Comprehensive error variants for API completeness
pub enum UserError {
    /// User not found
    #[error("User not found: {user_id}")]
    NotFound { user_id: u64 },

    /// User not found by registration number
    #[error("User not found with regno: {regno}")]
    NotFoundByRegno { regno: String },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthFailed { reason: String },

    /// User is banned
    #[error("User is banned: {user_id}")]
    Banned { user_id: u64 },

    /// Registration number already exists
    #[error("Registration number already exists: {regno}")]
    RegnoExists { regno: String },

    /// Invalid registration data
    #[error("Invalid registration data: {reason}")]
    InvalidRegistration { reason: String },

    /// Session limit exceeded
    #[error("Maximum session limit exceeded for user {user_id}")]
    SessionLimitExceeded { user_id: u64 },

    /// Not authenticated
    #[error("Not authenticated")]
    NotAuthenticated,

    /// Permission denied
    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    /// Chat disabled for user
    #[error("Chat is disabled for user {user_id}")]
    ChatDisabled { user_id: u64 },
}

impl UserError {
    /// Get an error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            UserError::NotFound { .. } => "USER_NOT_FOUND",
            UserError::NotFoundByRegno { .. } => "USER_NOT_FOUND",
            UserError::AuthFailed { .. } => "AUTH_FAILED",
            UserError::Banned { .. } => "USER_BANNED",
            UserError::RegnoExists { .. } => "REGNO_EXISTS",
            UserError::InvalidRegistration { .. } => "INVALID_REGISTRATION",
            UserError::SessionLimitExceeded { .. } => "SESSION_LIMIT",
            UserError::NotAuthenticated => "NOT_AUTHENTICATED",
            UserError::PermissionDenied { .. } => "PERMISSION_DENIED",
            UserError::ChatDisabled { .. } => "CHAT_DISABLED",
        }
    }
}

// =============================================================================
// MARKET ERRORS
// =============================================================================

/// Errors related to market operations.
#[derive(Debug, Error, Clone)]
#[allow(dead_code)] // Comprehensive error variants for API completeness
pub enum MarketError {
    /// Company not found
    #[error("Company not found: {symbol}")]
    CompanyNotFound { symbol: String },

    /// Company is bankrupt
    #[error("Company is bankrupt: {symbol}")]
    CompanyBankrupt { symbol: String },

    /// Invalid symbol format
    #[error("Invalid symbol format: {symbol}")]
    InvalidSymbol { symbol: String },

    /// No market data available
    #[error("No market data available for {symbol}")]
    NoMarketData { symbol: String },

    /// Symbol already exists
    #[error("Symbol already exists: {symbol}")]
    SymbolExists { symbol: String },
}

impl MarketError {
    /// Get an error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            MarketError::CompanyNotFound { .. } => "COMPANY_NOT_FOUND",
            MarketError::CompanyBankrupt { .. } => "COMPANY_BANKRUPT",
            MarketError::InvalidSymbol { .. } => "INVALID_SYMBOL",
            MarketError::NoMarketData { .. } => "NO_MARKET_DATA",
            MarketError::SymbolExists { .. } => "SYMBOL_EXISTS",
        }
    }
}

// =============================================================================
// REPOSITORY ERRORS
// =============================================================================

/// Errors related to data persistence and repositories.
#[derive(Debug, Error)]
#[allow(dead_code)] // Comprehensive error variants for API completeness
pub enum RepositoryError {
    /// Entity not found
    #[error("Entity not found")]
    NotFound,

    /// Failed to save entity
    #[error("Failed to save: {reason}")]
    SaveFailed { reason: String },

    /// Failed to load data
    #[error("Failed to load: {reason}")]
    LoadFailed { reason: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for RepositoryError {
    fn from(err: std::io::Error) -> Self {
        RepositoryError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(err: serde_json::Error) -> Self {
        RepositoryError::Serialization(err.to_string())
    }
}

// =============================================================================
// CONFIG ERRORS
// =============================================================================

/// Errors related to configuration.
#[derive(Debug, Error, Clone)]
#[allow(dead_code)] // Comprehensive error variants for API completeness
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Invalid configuration
    #[error("Invalid configuration: {reason}")]
    Invalid { reason: String },

    /// Missing required field
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },
}

// =============================================================================
// RESULT TYPE ALIASES
// =============================================================================

/// Result type alias for domain operations
#[allow(dead_code)] // Convenience type alias
pub type DomainResult<T> = Result<T, DomainError>;

/// Result type alias for trading operations
#[allow(dead_code)] // Convenience type alias
pub type TradingResult<T> = Result<T, TradingError>;

/// Result type alias for user operations
#[allow(dead_code)] // Convenience type alias
pub type UserResult<T> = Result<T, UserError>;

/// Result type alias for market operations
#[allow(dead_code)] // Convenience type alias
pub type MarketResult<T> = Result<T, MarketError>;

/// Result type alias for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

// =============================================================================
// ERROR RESPONSE FOR API
// =============================================================================

/// Error response structure for WebSocket API
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    #[allow(dead_code)] // Constructor for API responses
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<TradingError> for ErrorResponse {
    fn from(err: TradingError) -> Self {
        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
        }
    }
}

impl From<UserError> for ErrorResponse {
    fn from(err: UserError) -> Self {
        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
        }
    }
}

impl From<MarketError> for ErrorResponse {
    fn from(err: MarketError) -> Self {
        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
        }
    }
}

impl From<DomainError> for ErrorResponse {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Trading(e) => e.into(),
            DomainError::User(e) => e.into(),
            DomainError::Market(e) => e.into(),
            DomainError::Repository(e) => Self {
                code: "REPOSITORY_ERROR".to_string(),
                message: e.to_string(),
            },
            DomainError::Config(e) => Self {
                code: "CONFIG_ERROR".to_string(),
                message: e.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // TRADING ERROR TESTS
    // =========================================================================

    #[test]
    fn test_trading_error_codes() {
        assert_eq!(TradingError::MarketClosed.error_code(), "MARKET_CLOSED");
        assert_eq!(
            TradingError::InsufficientFunds {
                required: 100,
                available: 50
            }
            .error_code(),
            "INSUFFICIENT_FUNDS"
        );
        assert_eq!(
            TradingError::InsufficientShares {
                required: 10,
                available: 5
            }
            .error_code(),
            "INSUFFICIENT_SHARES"
        );
        assert_eq!(
            TradingError::InsufficientMargin {
                required: 100,
                available: 50
            }
            .error_code(),
            "INSUFFICIENT_MARGIN"
        );
        assert_eq!(
            TradingError::OrderNotFound { order_id: 123 }.error_code(),
            "ORDER_NOT_FOUND"
        );
        assert_eq!(
            TradingError::InvalidOrder {
                reason: "test".to_string()
            }
            .error_code(),
            "INVALID_ORDER"
        );
        assert_eq!(
            TradingError::SymbolNotFound {
                symbol: "AAPL".to_string()
            }
            .error_code(),
            "SYMBOL_NOT_FOUND"
        );
        assert_eq!(
            TradingError::TradingHalted {
                symbol: "AAPL".to_string(),
                until: 12345
            }
            .error_code(),
            "TRADING_HALTED"
        );
        assert_eq!(TradingError::NotOrderOwner.error_code(), "NOT_ORDER_OWNER");
    }

    #[test]
    fn test_trading_error_display() {
        assert_eq!(
            TradingError::MarketClosed.to_string(),
            "Market is currently closed"
        );
        assert!(TradingError::InsufficientFunds {
            required: 100,
            available: 50
        }
        .to_string()
        .contains("100"));
        assert!(TradingError::InsufficientShares {
            required: 10,
            available: 5
        }
        .to_string()
        .contains("10"));
        assert!(TradingError::OrderNotFound { order_id: 123 }
            .to_string()
            .contains("123"));
        assert!(TradingError::InvalidOrder {
            reason: "bad order".to_string()
        }
        .to_string()
        .contains("bad order"));
        assert!(TradingError::SymbolNotFound {
            symbol: "XYZ".to_string()
        }
        .to_string()
        .contains("XYZ"));
        assert!(TradingError::TradingHalted {
            symbol: "HALT".to_string(),
            until: 999
        }
        .to_string()
        .contains("HALT"));
        assert_eq!(
            TradingError::NotOrderOwner.to_string(),
            "Order does not belong to user"
        );
    }

    // =========================================================================
    // USER ERROR TESTS
    // =========================================================================

    #[test]
    fn test_user_error_codes() {
        assert_eq!(
            UserError::NotFound { user_id: 1 }.error_code(),
            "USER_NOT_FOUND"
        );
        assert_eq!(
            UserError::NotFoundByRegno {
                regno: "REG123".to_string()
            }
            .error_code(),
            "USER_NOT_FOUND"
        );
        assert_eq!(
            UserError::AuthFailed {
                reason: "bad password".to_string()
            }
            .error_code(),
            "AUTH_FAILED"
        );
        assert_eq!(UserError::Banned { user_id: 1 }.error_code(), "USER_BANNED");
        assert_eq!(
            UserError::RegnoExists {
                regno: "REG123".to_string()
            }
            .error_code(),
            "REGNO_EXISTS"
        );
        assert_eq!(
            UserError::InvalidRegistration {
                reason: "test".to_string()
            }
            .error_code(),
            "INVALID_REGISTRATION"
        );
        assert_eq!(
            UserError::SessionLimitExceeded { user_id: 1 }.error_code(),
            "SESSION_LIMIT"
        );
        assert_eq!(
            UserError::NotAuthenticated.error_code(),
            "NOT_AUTHENTICATED"
        );
        assert_eq!(
            UserError::PermissionDenied {
                action: "admin".to_string()
            }
            .error_code(),
            "PERMISSION_DENIED"
        );
        assert_eq!(
            UserError::ChatDisabled { user_id: 1 }.error_code(),
            "CHAT_DISABLED"
        );
    }

    #[test]
    fn test_user_error_display() {
        assert!(UserError::NotFound { user_id: 42 }
            .to_string()
            .contains("42"));
        assert!(UserError::NotFoundByRegno {
            regno: "X123".to_string()
        }
        .to_string()
        .contains("X123"));
        assert!(UserError::AuthFailed {
            reason: "wrong".to_string()
        }
        .to_string()
        .contains("wrong"));
        assert!(UserError::Banned { user_id: 99 }.to_string().contains("99"));
        assert!(UserError::RegnoExists {
            regno: "DUP".to_string()
        }
        .to_string()
        .contains("DUP"));
        assert!(UserError::InvalidRegistration {
            reason: "empty".to_string()
        }
        .to_string()
        .contains("empty"));
        assert!(UserError::SessionLimitExceeded { user_id: 5 }
            .to_string()
            .contains("5"));
        assert_eq!(UserError::NotAuthenticated.to_string(), "Not authenticated");
        assert!(UserError::PermissionDenied {
            action: "delete".to_string()
        }
        .to_string()
        .contains("delete"));
        assert!(UserError::ChatDisabled { user_id: 7 }
            .to_string()
            .contains("7"));
    }

    // =========================================================================
    // MARKET ERROR TESTS
    // =========================================================================

    #[test]
    fn test_market_error_codes() {
        assert_eq!(
            MarketError::CompanyNotFound {
                symbol: "XYZ".to_string()
            }
            .error_code(),
            "COMPANY_NOT_FOUND"
        );
        assert_eq!(
            MarketError::CompanyBankrupt {
                symbol: "FAIL".to_string()
            }
            .error_code(),
            "COMPANY_BANKRUPT"
        );
        assert_eq!(
            MarketError::InvalidSymbol {
                symbol: "123".to_string()
            }
            .error_code(),
            "INVALID_SYMBOL"
        );
        assert_eq!(
            MarketError::NoMarketData {
                symbol: "NEW".to_string()
            }
            .error_code(),
            "NO_MARKET_DATA"
        );
        assert_eq!(
            MarketError::SymbolExists {
                symbol: "AAPL".to_string()
            }
            .error_code(),
            "SYMBOL_EXISTS"
        );
    }

    #[test]
    fn test_market_error_display() {
        assert!(MarketError::CompanyNotFound {
            symbol: "GONE".to_string()
        }
        .to_string()
        .contains("GONE"));
        assert!(MarketError::CompanyBankrupt {
            symbol: "BROKE".to_string()
        }
        .to_string()
        .contains("BROKE"));
        assert!(MarketError::InvalidSymbol {
            symbol: "BAD".to_string()
        }
        .to_string()
        .contains("BAD"));
        assert!(MarketError::NoMarketData {
            symbol: "EMPTY".to_string()
        }
        .to_string()
        .contains("EMPTY"));
        assert!(MarketError::SymbolExists {
            symbol: "DUP".to_string()
        }
        .to_string()
        .contains("DUP"));
    }

    // =========================================================================
    // REPOSITORY ERROR TESTS
    // =========================================================================

    #[test]
    fn test_repository_error_display() {
        assert_eq!(RepositoryError::NotFound.to_string(), "Entity not found");
        assert!(RepositoryError::SaveFailed {
            reason: "disk full".to_string()
        }
        .to_string()
        .contains("disk full"));
        assert!(RepositoryError::LoadFailed {
            reason: "corrupt".to_string()
        }
        .to_string()
        .contains("corrupt"));
        assert!(RepositoryError::Serialization("parse error".to_string())
            .to_string()
            .contains("parse error"));
        assert!(RepositoryError::Io("permission denied".to_string())
            .to_string()
            .contains("permission denied"));
    }

    #[test]
    fn test_repository_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let repo_err: RepositoryError = io_err.into();
        assert!(repo_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_repository_error_from_json() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let repo_err: RepositoryError = json_err.into();
        assert!(repo_err.to_string().contains("Serialization"));
    }

    // =========================================================================
    // CONFIG ERROR TESTS
    // =========================================================================

    #[test]
    fn test_config_error_display() {
        assert!(ConfigError::FileNotFound {
            path: "/etc/app.conf".to_string()
        }
        .to_string()
        .contains("/etc/app.conf"));
        assert!(ConfigError::Invalid {
            reason: "bad format".to_string()
        }
        .to_string()
        .contains("bad format"));
        assert!(ConfigError::MissingField {
            field: "api_key".to_string()
        }
        .to_string()
        .contains("api_key"));
    }

    // =========================================================================
    // DOMAIN ERROR TESTS
    // =========================================================================

    #[test]
    fn test_domain_error_from_trading() {
        let trading_err = TradingError::MarketClosed;
        let domain_err: DomainError = trading_err.into();
        assert!(domain_err.to_string().contains("Trading error"));
    }

    #[test]
    fn test_domain_error_from_user() {
        let user_err = UserError::NotAuthenticated;
        let domain_err: DomainError = user_err.into();
        assert!(domain_err.to_string().contains("User error"));
    }

    #[test]
    fn test_domain_error_from_market() {
        let market_err = MarketError::CompanyNotFound {
            symbol: "XYZ".to_string(),
        };
        let domain_err: DomainError = market_err.into();
        assert!(domain_err.to_string().contains("Market error"));
    }

    #[test]
    fn test_domain_error_from_repository() {
        let repo_err = RepositoryError::NotFound;
        let domain_err: DomainError = repo_err.into();
        assert!(domain_err.to_string().contains("Repository error"));
    }

    #[test]
    fn test_domain_error_from_config() {
        let config_err = ConfigError::Invalid {
            reason: "test".to_string(),
        };
        let domain_err: DomainError = config_err.into();
        assert!(domain_err.to_string().contains("Config error"));
    }

    // =========================================================================
    // ERROR RESPONSE TESTS
    // =========================================================================

    #[test]
    fn test_error_response_new() {
        let resp = ErrorResponse::new("TEST_CODE", "Test message");
        assert_eq!(resp.code, "TEST_CODE");
        assert_eq!(resp.message, "Test message");
    }

    #[test]
    fn test_error_response_from_trading() {
        let err = TradingError::MarketClosed;
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "MARKET_CLOSED");
        assert!(resp.message.contains("closed"));
    }

    #[test]
    fn test_error_response_from_user() {
        let err = UserError::NotAuthenticated;
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "NOT_AUTHENTICATED");
    }

    #[test]
    fn test_error_response_from_market() {
        let err = MarketError::CompanyNotFound {
            symbol: "TEST".to_string(),
        };
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "COMPANY_NOT_FOUND");
    }

    #[test]
    fn test_error_response_from_domain_trading() {
        let err = DomainError::Trading(TradingError::NotOrderOwner);
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "NOT_ORDER_OWNER");
    }

    #[test]
    fn test_error_response_from_domain_user() {
        let err = DomainError::User(UserError::Banned { user_id: 1 });
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "USER_BANNED");
    }

    #[test]
    fn test_error_response_from_domain_market() {
        let err = DomainError::Market(MarketError::SymbolExists {
            symbol: "X".to_string(),
        });
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "SYMBOL_EXISTS");
    }

    #[test]
    fn test_error_response_from_domain_repository() {
        let err = DomainError::Repository(RepositoryError::NotFound);
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "REPOSITORY_ERROR");
    }

    #[test]
    fn test_error_response_from_domain_config() {
        let err = DomainError::Config(ConfigError::Invalid {
            reason: "x".to_string(),
        });
        let resp: ErrorResponse = err.into();
        assert_eq!(resp.code, "CONFIG_ERROR");
    }

    // Test InsufficientMargin display
    #[test]
    fn test_insufficient_margin_display() {
        let err = TradingError::InsufficientMargin {
            required: 500,
            available: 200,
        };
        assert!(err.to_string().contains("500"));
        assert!(err.to_string().contains("200"));
    }
}
