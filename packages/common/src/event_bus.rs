use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Evento del sistema para comunicación interna entre módulos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_type: String,
    pub source: String,
    pub prospect_id: Option<String>,
    pub student_id: Option<String>,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

/// Trait para publicar eventos del sistema.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publica un evento en el bus.
    async fn publish(&self, event: SystemEvent);
}

/// Implementación concreta de [`EventBus`] basada en canales `tokio::sync::broadcast`.
pub struct BroadcastBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl BroadcastBus {
    /// Crea un nuevo bus con la capacidad de buffer especificada.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Suscribe un nuevo receptor al bus de eventos.
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
