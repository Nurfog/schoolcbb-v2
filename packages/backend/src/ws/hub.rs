use tokio::sync::broadcast;

#[derive(Clone)]
pub struct WsHub {
    tx: broadcast::Sender<String>,
}

impl WsHub {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }

    pub fn broadcast(&self, message: &str) {
        if self.tx.send(message.to_string()).is_err() {
            tracing::warn!("No active WebSocket listeners");
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}
