//! Secure authentication token service.
//!
//! This module provides cryptographically secure token generation and validation
//! for user authentication. Tokens are opaque, random strings that map to user sessions.

use dashmap::DashMap;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Length of the token in bytes (before hex encoding)
const TOKEN_BYTES: usize = 32;

/// A secure authentication token (64 hex characters)
pub type AuthToken = String;

/// User ID type
pub type UserId = u64;

/// Token metadata stored in the service
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub user_id: UserId,
    pub created_at: i64,
    pub last_used: i64,
}

/// Secure token service for authentication.
///
/// Generates cryptographically random tokens and maintains a mapping
/// from tokens to user sessions. Tokens are opaque and unpredictable.
pub struct TokenService {
    /// Maps token -> token info
    tokens: Arc<DashMap<AuthToken, TokenInfo>>,
    /// Maps user_id -> active tokens (for cleanup and single-session enforcement)
    user_tokens: Arc<DashMap<UserId, Vec<AuthToken>>>,
    /// Maximum tokens per user (0 = unlimited)
    max_tokens_per_user: u32,
}

impl TokenService {
    /// Create a new token service.
    ///
    /// # Arguments
    /// * `max_tokens_per_user` - Maximum active tokens per user (0 = unlimited)
    pub fn new(max_tokens_per_user: u32) -> Self {
        info!(
            "TokenService initialized with max {} tokens per user",
            if max_tokens_per_user == 0 {
                "unlimited".to_string()
            } else {
                max_tokens_per_user.to_string()
            }
        );
        Self {
            tokens: Arc::new(DashMap::new()),
            user_tokens: Arc::new(DashMap::new()),
            max_tokens_per_user,
        }
    }

    /// Generate a new secure token for a user.
    ///
    /// If the user exceeds their token limit, old tokens are revoked.
    ///
    /// # Returns
    /// The new token and a list of revoked token strings
    pub fn create_token(&self, user_id: UserId) -> (AuthToken, Vec<AuthToken>) {
        let token = Self::generate_secure_token();
        let now = chrono::Utc::now().timestamp();

        let token_info = TokenInfo {
            user_id,
            created_at: now,
            last_used: now,
        };

        let mut revoked = Vec::new();

        // Enforce token limit per user
        let mut user_tokens = self.user_tokens.entry(user_id).or_insert_with(Vec::new);

        if self.max_tokens_per_user > 0 {
            while user_tokens.len() >= self.max_tokens_per_user as usize {
                if let Some(old_token) = user_tokens.first().cloned() {
                    user_tokens.remove(0);
                    self.tokens.remove(&old_token);
                    revoked.push(old_token.clone());
                    info!(
                        "Revoked old token for user {} (max tokens enforced)",
                        user_id
                    );
                }
            }
        }

        // Add new token
        user_tokens.push(token.clone());
        drop(user_tokens);

        self.tokens.insert(token.clone(), token_info);
        debug!("Created new token for user {}", user_id);

        (token, revoked)
    }

    /// Validate a token and return the associated user ID.
    ///
    /// Also updates the last_used timestamp.
    ///
    /// # Returns
    /// `Some(user_id)` if valid, `None` if invalid or expired
    pub fn validate_token(&self, token: &str) -> Option<UserId> {
        if let Some(mut token_info) = self.tokens.get_mut(token) {
            // Update last used time
            token_info.last_used = chrono::Utc::now().timestamp();
            let user_id = token_info.user_id;
            debug!("Token validated for user {}", user_id);
            Some(user_id)
        } else {
            warn!("Invalid token attempted");
            None
        }
    }

    /// Revoke a specific token.
    ///
    /// # Returns
    /// `true` if the token existed and was revoked
    pub fn revoke_token(&self, token: &str) -> bool {
        if let Some((_, token_info)) = self.tokens.remove(token) {
            // Remove from user's token list
            if let Some(mut user_tokens) = self.user_tokens.get_mut(&token_info.user_id) {
                user_tokens.retain(|t| t != token);
            }
            info!("Token revoked for user {}", token_info.user_id);
            true
        } else {
            false
        }
    }

    /// Revoke all tokens for a user.
    ///
    /// # Returns
    /// Number of tokens revoked
    pub fn revoke_all_user_tokens(&self, user_id: UserId) -> usize {
        if let Some((_, tokens)) = self.user_tokens.remove(&user_id) {
            let count = tokens.len();
            for token in tokens {
                self.tokens.remove(&token);
            }
            info!("Revoked {} tokens for user {}", count, user_id);
            count
        } else {
            0
        }
    }

    /// Get the user ID associated with a token without updating last_used.
    ///
    /// Useful for checking ownership without marking as active.
    pub fn get_user_id(&self, token: &str) -> Option<UserId> {
        self.tokens.get(token).map(|info| info.user_id)
    }

    /// Check if a token is valid without updating last_used.
    pub fn is_valid(&self, token: &str) -> bool {
        self.tokens.contains_key(token)
    }

    /// Get the number of active tokens for a user.
    pub fn user_token_count(&self, user_id: UserId) -> usize {
        self.user_tokens
            .get(&user_id)
            .map(|tokens| tokens.len())
            .unwrap_or(0)
    }

    /// Get total number of active tokens across all users.
    pub fn total_token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Generate a cryptographically secure random token.
    ///
    /// Returns a 64-character hex string (256 bits of entropy).
    fn generate_secure_token() -> AuthToken {
        let mut rng = thread_rng();
        let mut bytes = [0u8; TOKEN_BYTES];
        rng.fill(&mut bytes);
        hex::encode(bytes)
    }

    /// Clean up expired tokens (tokens not used in `max_age_secs` seconds).
    ///
    /// This should be called periodically to prevent unbounded growth.
    ///
    /// # Returns
    /// Number of tokens cleaned up
    #[allow(dead_code)]
    pub fn cleanup_expired(&self, max_age_secs: i64) -> usize {
        let cutoff = chrono::Utc::now().timestamp() - max_age_secs;
        let mut removed = 0;

        // Collect tokens to remove
        let to_remove: Vec<(AuthToken, UserId)> = self
            .tokens
            .iter()
            .filter(|entry| entry.last_used < cutoff)
            .map(|entry| (entry.key().clone(), entry.user_id))
            .collect();

        for (token, user_id) in to_remove {
            self.tokens.remove(&token);
            if let Some(mut user_tokens) = self.user_tokens.get_mut(&user_id) {
                user_tokens.retain(|t| t != &token);
            }
            removed += 1;
        }

        if removed > 0 {
            info!("Cleaned up {} expired tokens", removed);
        }

        removed
    }
}

impl Default for TokenService {
    fn default() -> Self {
        Self::new(1) // Default to single token per user
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_uniqueness() {
        let service = TokenService::new(0);
        let mut tokens = std::collections::HashSet::new();

        for _ in 0..1000 {
            let (token, _) = service.create_token(1);
            assert!(tokens.insert(token), "Token collision detected!");
        }
    }

    #[test]
    fn test_token_length() {
        let service = TokenService::new(0);
        let (token, _) = service.create_token(1);
        assert_eq!(token.len(), TOKEN_BYTES * 2); // Hex encoding doubles length
    }

    #[test]
    fn test_token_validation() {
        let service = TokenService::new(0);
        let (token, _) = service.create_token(42);

        assert_eq!(service.validate_token(&token), Some(42));
        assert_eq!(service.validate_token("invalid_token"), None);
    }

    #[test]
    fn test_token_revocation() {
        let service = TokenService::new(0);
        let (token, _) = service.create_token(1);

        assert!(service.is_valid(&token));
        assert!(service.revoke_token(&token));
        assert!(!service.is_valid(&token));
        assert!(!service.revoke_token(&token)); // Already revoked
    }

    #[test]
    fn test_max_tokens_enforcement() {
        let service = TokenService::new(2);

        let (token1, revoked1) = service.create_token(1);
        assert!(revoked1.is_empty());

        let (token2, revoked2) = service.create_token(1);
        assert!(revoked2.is_empty());

        // Third token should revoke first
        let (token3, revoked3) = service.create_token(1);
        assert_eq!(revoked3.len(), 1);
        assert_eq!(revoked3[0], token1);

        // Token1 should be invalid, token2 and token3 valid
        assert!(!service.is_valid(&token1));
        assert!(service.is_valid(&token2));
        assert!(service.is_valid(&token3));
    }

    #[test]
    fn test_revoke_all_user_tokens() {
        let service = TokenService::new(0);

        service.create_token(1);
        service.create_token(1);
        service.create_token(2);

        assert_eq!(service.user_token_count(1), 2);
        assert_eq!(service.user_token_count(2), 1);

        let revoked = service.revoke_all_user_tokens(1);
        assert_eq!(revoked, 2);
        assert_eq!(service.user_token_count(1), 0);
        assert_eq!(service.user_token_count(2), 1); // User 2 unaffected
    }

    #[test]
    fn test_different_users_different_tokens() {
        let service = TokenService::new(0);

        let (token1, _) = service.create_token(1);
        let (token2, _) = service.create_token(2);

        assert_ne!(token1, token2);
        assert_eq!(service.validate_token(&token1), Some(1));
        assert_eq!(service.validate_token(&token2), Some(2));
    }
}
