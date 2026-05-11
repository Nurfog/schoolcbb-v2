use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: String,
    pub source: String,
    pub prospect_id: Option<String>,
    pub student_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: SystemEvent);
}

pub struct BroadcastBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl BroadcastBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }
}

#[async_trait]
impl EventBus for BroadcastBus {
    async fn publish(&self, event: SystemEvent) {
        if let Err(e) = self.tx.send(event) {
            tracing::warn!("Event bus send error: {}", e);
        }
    }
}
