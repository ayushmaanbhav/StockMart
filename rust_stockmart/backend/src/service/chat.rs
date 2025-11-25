use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use crate::domain::models::ChatMessage;

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

    pub fn get_history(&self) -> Vec<ChatMessage> {
        self.history.lock().unwrap().clone()
    }
}
