use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct StreamService {
    // symbol -> list of senders for that symbol
    connections: Arc<DashMap<String, Vec<broadcast::Sender<String>>>>,
}

impl StreamService {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    pub fn add_connection(&self, symbol: &str) -> broadcast::Receiver<String> {
        let (tx, rx) = broadcast::channel(100);
        
        self.connections
            .entry(symbol.to_string())
            .or_insert_with(Vec::new)
            .push(tx);

        rx
    }

    pub fn broadcast_to_symbol(&self, symbol: &str, message: &str) {
        if let Some(mut senders) = self.connections.get_mut(symbol) {
            senders.retain(|sender| {
                sender.send(message.to_string()).is_ok()
            });
        }
    }
} 