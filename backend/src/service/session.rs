// ============================================
// Session Management Service
// Tracks active sessions per user to enforce single-session policy
// ============================================

#![allow(dead_code)] // SessionInfo fields and query methods for session management

use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

/// Unique session ID
pub type SessionId = u64;

/// User ID type
pub type UserId = u64;

/// Global session counter
static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

fn next_session_id() -> SessionId {
    NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
}

/// Session info stored per active connection
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub connected_at: i64,
    pub last_activity: i64,
}

/// Session manager tracks active sessions per user
pub struct SessionManager {
    /// Maps user_id -> active session IDs
    user_sessions: DashMap<UserId, Vec<SessionId>>,
    /// Maps session_id -> session info
    sessions: DashMap<SessionId, SessionInfo>,
    /// Maximum sessions allowed per user (0 = unlimited)
    max_sessions_per_user: u32,
}

impl SessionManager {
    pub fn new(max_sessions_per_user: u32) -> Self {
        info!(
            "SessionManager initialized with max {} sessions per user",
            if max_sessions_per_user == 0 {
                "unlimited".to_string()
            } else {
                max_sessions_per_user.to_string()
            }
        );
        Self {
            user_sessions: DashMap::new(),
            sessions: DashMap::new(),
            max_sessions_per_user,
        }
    }

    /// Try to create a new session for a user
    /// Returns (session_id, kicked_session_ids) on success
    /// kicked_session_ids contains any old sessions that were terminated
    pub fn create_session(&self, user_id: UserId) -> (SessionId, Vec<SessionId>) {
        let session_id = next_session_id();
        let now = chrono::Utc::now().timestamp();

        let session_info = SessionInfo {
            session_id,
            user_id,
            connected_at: now,
            last_activity: now,
        };

        let mut kicked_sessions = Vec::new();

        // Check if user already has sessions
        let mut user_sessions = self.user_sessions.entry(user_id).or_insert_with(Vec::new);

        // If max sessions enforced, kick old sessions
        if self.max_sessions_per_user > 0 {
            while user_sessions.len() >= self.max_sessions_per_user as usize {
                if let Some(old_session_id) = user_sessions.first().cloned() {
                    user_sessions.remove(0);
                    self.sessions.remove(&old_session_id);
                    kicked_sessions.push(old_session_id);
                    info!(
                        "Kicked old session {} for user {} (max sessions enforced)",
                        old_session_id, user_id
                    );
                }
            }
        }

        // Add new session
        user_sessions.push(session_id);
        drop(user_sessions);

        self.sessions.insert(session_id, session_info);
        debug!("Created session {} for user {}", session_id, user_id);

        (session_id, kicked_sessions)
    }

    /// Remove a session when user disconnects
    pub fn remove_session(&self, session_id: SessionId) {
        if let Some((_, session_info)) = self.sessions.remove(&session_id) {
            // Remove from user's session list
            if let Some(mut user_sessions) = self.user_sessions.get_mut(&session_info.user_id) {
                user_sessions.retain(|&id| id != session_id);
                debug!(
                    "Removed session {} for user {} ({} sessions remaining)",
                    session_id,
                    session_info.user_id,
                    user_sessions.len()
                );
            }
        }
    }

    /// Get session info
    pub fn get_session(&self, session_id: SessionId) -> Option<SessionInfo> {
        self.sessions.get(&session_id).map(|s| s.clone())
    }

    /// Check if a user has any active sessions
    pub fn has_active_session(&self, user_id: UserId) -> bool {
        self.user_sessions
            .get(&user_id)
            .map(|sessions| !sessions.is_empty())
            .unwrap_or(false)
    }

    /// Get all sessions for a user
    pub fn get_user_sessions(&self, user_id: UserId) -> Vec<SessionId> {
        self.user_sessions
            .get(&user_id)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Update last activity for a session
    pub fn touch_session(&self, session_id: SessionId) {
        if let Some(mut session) = self.sessions.get_mut(&session_id) {
            session.last_activity = chrono::Utc::now().timestamp();
        }
    }

    /// Get total active session count
    pub fn total_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Get total unique users with active sessions
    pub fn total_users(&self) -> usize {
        self.user_sessions
            .iter()
            .filter(|r| !r.value().is_empty())
            .count()
    }

    /// Get count of active sessions (alias for admin dashboard)
    pub fn active_session_count(&self) -> usize {
        self.total_users()
    }

    /// Get all active sessions (for admin dashboard)
    pub fn get_all_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.iter().map(|r| r.value().clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_session_enforcement() {
        let manager = SessionManager::new(1);

        // Create first session
        let (session1, kicked1) = manager.create_session(100);
        assert!(kicked1.is_empty());
        assert!(manager.has_active_session(100));

        // Create second session - should kick first
        let (session2, kicked2) = manager.create_session(100);
        assert_eq!(kicked2.len(), 1);
        assert_eq!(kicked2[0], session1);

        // Session 1 should be gone, session 2 should exist
        assert!(manager.get_session(session1).is_none());
        assert!(manager.get_session(session2).is_some());
    }

    #[test]
    fn test_unlimited_sessions() {
        let manager = SessionManager::new(0);

        // Create multiple sessions
        let (session1, _) = manager.create_session(100);
        let (session2, kicked) = manager.create_session(100);

        assert!(kicked.is_empty());
        assert!(manager.get_session(session1).is_some());
        assert!(manager.get_session(session2).is_some());
        assert_eq!(manager.get_user_sessions(100).len(), 2);
    }
}
