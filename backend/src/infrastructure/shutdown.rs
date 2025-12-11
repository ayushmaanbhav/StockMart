//! Graceful shutdown coordination.
//!
//! Provides utilities for coordinating graceful shutdown across
//! multiple background services and tasks.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

/// Shutdown signal that can be shared across tasks.
///
/// When shutdown is triggered, all tasks holding a clone of this
/// signal will be notified and should begin their cleanup process.
#[derive(Clone)]
pub struct ShutdownSignal {
    /// Broadcast sender for shutdown notification
    sender: broadcast::Sender<()>,
    /// Flag indicating if shutdown has been initiated
    is_shutdown: Arc<AtomicBool>,
}

impl ShutdownSignal {
    /// Create a new shutdown signal
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1);
        Self {
            sender,
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Trigger the shutdown signal
    ///
    /// This will notify all listeners that shutdown has been initiated.
    pub fn trigger(&self) {
        if self
            .is_shutdown
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            info!("Shutdown signal triggered");
            // Ignore send errors - receivers may have been dropped
            let _ = self.sender.send(());
        }
    }

    /// Check if shutdown has been initiated
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }

    /// Get a receiver for the shutdown signal
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// Wait for the shutdown signal
    pub async fn wait(&self) {
        if self.is_shutdown() {
            return;
        }

        let mut rx = self.subscribe();
        // Ignore errors - we just need to know shutdown happened
        let _ = rx.recv().await;
    }
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM).
///
/// Returns when either signal is received.
pub async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            info!("Received SIGTERM signal");
        }
    }
}
