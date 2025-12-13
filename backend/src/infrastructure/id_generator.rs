//! ID generation utilities.
//!
//! This module provides thread-safe ID generators that can be properly
//! initialized from persisted state and avoid the global mutable state
//! anti-pattern of static AtomicU64 counters.

#![allow(dead_code)] // Generator trait methods and initialization helpers

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// =============================================================================
// ID GENERATOR TRAIT
// =============================================================================

/// Trait for ID generators allowing different implementations.
pub trait IdGenerator: Send + Sync {
    /// Generate the next unique ID
    fn next_id(&self) -> u64;

    /// Get the current counter value without incrementing
    fn current(&self) -> u64;

    /// Reset the counter to a specific value
    fn reset(&self, value: u64);
}

// =============================================================================
// ATOMIC ID GENERATOR
// =============================================================================

/// Thread-safe atomic ID generator using AtomicU64.
///
/// This implementation is suitable for single-instance deployments
/// and provides monotonically increasing IDs.
#[derive(Debug)]
pub struct AtomicIdGenerator {
    counter: AtomicU64,
}

impl AtomicIdGenerator {
    /// Create a new generator starting from 1
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// Create a generator starting from a specific value
    pub fn starting_from(start: u64) -> Self {
        Self {
            counter: AtomicU64::new(start),
        }
    }

    /// Create a generator initialized from the maximum existing ID.
    /// The next generated ID will be max_existing_id + 1.
    pub fn from_max_id(max_existing_id: u64) -> Self {
        Self {
            counter: AtomicU64::new(max_existing_id + 1),
        }
    }
}

impl Default for AtomicIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for AtomicIdGenerator {
    fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    fn current(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }

    fn reset(&self, value: u64) {
        self.counter.store(value, Ordering::Relaxed);
    }
}

// =============================================================================
// ID GENERATORS COLLECTION
// =============================================================================

/// Collection of ID generators for different entity types.
///
/// This struct centralizes all ID generation and can be properly
/// initialized from persisted data.
#[derive(Clone)]
pub struct IdGenerators {
    pub user: Arc<dyn IdGenerator>,
    pub company: Arc<dyn IdGenerator>,
    pub order: Arc<dyn IdGenerator>,
    pub trade: Arc<dyn IdGenerator>,
    pub sync: Arc<dyn IdGenerator>,
}

impl IdGenerators {
    /// Create new ID generators all starting from 1
    pub fn new() -> Self {
        Self {
            user: Arc::new(AtomicIdGenerator::new()),
            company: Arc::new(AtomicIdGenerator::new()),
            order: Arc::new(AtomicIdGenerator::new()),
            trade: Arc::new(AtomicIdGenerator::new()),
            sync: Arc::new(AtomicIdGenerator::new()),
        }
    }

    /// Create ID generators initialized from maximum existing IDs.
    ///
    /// This should be called after loading persisted data to ensure
    /// new IDs don't conflict with existing ones.
    pub fn from_max_ids(
        max_user_id: u64,
        max_company_id: u64,
        max_order_id: u64,
        max_trade_id: u64,
    ) -> Self {
        Self {
            user: Arc::new(AtomicIdGenerator::from_max_id(max_user_id)),
            company: Arc::new(AtomicIdGenerator::from_max_id(max_company_id)),
            order: Arc::new(AtomicIdGenerator::from_max_id(max_order_id)),
            trade: Arc::new(AtomicIdGenerator::from_max_id(max_trade_id)),
            sync: Arc::new(AtomicIdGenerator::new()),
        }
    }

    /// Generate a new user ID
    pub fn next_user_id(&self) -> u64 {
        self.user.next_id()
    }

    /// Generate a new company ID
    pub fn next_company_id(&self) -> u64 {
        self.company.next_id()
    }

    /// Generate a new order ID
    pub fn next_order_id(&self) -> u64 {
        self.order.next_id()
    }

    /// Generate a new trade ID
    pub fn next_trade_id(&self) -> u64 {
        self.trade.next_id()
    }

    /// Generate a new sync ID
    pub fn next_sync_id(&self) -> u64 {
        self.sync.next_id()
    }
}

impl Default for IdGenerators {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// GLOBAL SINGLETON
// =============================================================================

use lazy_static::lazy_static;

lazy_static! {
    /// Global singleton instance of IdGenerators for use throughout the application.
    /// This provides a single source of truth for ID generation.
    static ref GLOBAL_ID_GENERATORS: IdGenerators = IdGenerators::new();
}

impl IdGenerators {
    /// Get the global IdGenerators instance.
    ///
    /// This is the preferred way to access ID generation throughout the application.
    pub fn global() -> &'static IdGenerators {
        &GLOBAL_ID_GENERATORS
    }

    /// Initialize the global generators from maximum existing IDs.
    ///
    /// Call this after loading persisted data to ensure new IDs don't conflict
    /// with existing ones. This resets the internal counters to start from
    /// max_id + 1 for each entity type.
    ///
    /// # Arguments
    /// * `max_user_id` - Maximum existing user ID (0 if no users)
    /// * `max_company_id` - Maximum existing company ID (0 if no companies)
    pub fn init_from_persisted(max_user_id: u64, max_company_id: u64) {
        // Reset the global generators to continue from the max IDs
        // Add 1 so the next generated ID is max + 1
        GLOBAL_ID_GENERATORS.user.reset(max_user_id + 1);
        GLOBAL_ID_GENERATORS.company.reset(max_company_id + 1);
        // Orders and trades are not persisted, so they start fresh
        // Sync IDs start fresh each session
        tracing::info!(
            "ID generators initialized: users will start from {}, companies from {}",
            max_user_id + 1,
            max_company_id + 1
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_generator() {
        let gen = AtomicIdGenerator::new();
        assert_eq!(gen.next_id(), 1);
        assert_eq!(gen.next_id(), 2);
        assert_eq!(gen.next_id(), 3);
        assert_eq!(gen.current(), 4);
    }

    #[test]
    fn test_from_max_id() {
        let gen = AtomicIdGenerator::from_max_id(100);
        assert_eq!(gen.next_id(), 101);
        assert_eq!(gen.next_id(), 102);
    }

    #[test]
    fn test_reset() {
        let gen = AtomicIdGenerator::new();
        gen.next_id();
        gen.next_id();
        gen.reset(1);
        assert_eq!(gen.next_id(), 1);
    }

    #[test]
    fn test_id_generators_collection() {
        let gens = IdGenerators::new();
        assert_eq!(gens.next_user_id(), 1);
        assert_eq!(gens.next_user_id(), 2);
        assert_eq!(gens.next_company_id(), 1);
        assert_eq!(gens.next_order_id(), 1);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let gen = Arc::new(AtomicIdGenerator::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let gen_clone = Arc::clone(&gen);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    gen_clone.next_id();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // 10 threads * 100 iterations = 1000 IDs generated
        // Next ID should be 1001
        assert_eq!(gen.current(), 1001);
    }
}
