use crate::domain::models::ChatMessage;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct ChatService {
    tx: broadcast::Sender<ChatMessage>,
    history: Arc<Mutex<Vec<ChatMessage>>>,
}

impl ChatService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChatMessage> {
        self.tx.subscribe()
    }

    pub fn broadcast_message(&self, message: ChatMessage) {
        // Add to history
        {
            let mut history = self.history.lock().unwrap();
            history.push(message.clone());
            if history.len() > 50 {
                history.remove(0);
            }
        }

        // Broadcast
        let _ = self.tx.send(message);
    }

    #[allow(dead_code)] // API method for retrieving full chat history
    pub fn get_history(&self) -> Vec<ChatMessage> {
        self.history.lock().unwrap().clone()
    }

    /// Get recent chat messages for state sync
    pub fn get_recent(&self, count: usize) -> Vec<ChatMessage> {
        let history = self.history.lock().unwrap();
        history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}
