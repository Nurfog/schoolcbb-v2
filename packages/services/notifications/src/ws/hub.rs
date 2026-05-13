use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

#[derive(Clone)]
pub struct WsHub {
    users: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, user_id: Uuid) -> broadcast::Receiver<String> {
        let mut users = self.users.write().await;
        let tx = users
            .entry(user_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone();
        tx.subscribe()
    }

    pub async fn broadcast_to(&self, user_id: &Uuid, message: &str) {
        let users = self.users.read().await;
        if let Some(tx) = users.get(user_id) {
            let _ = tx.send(message.to_string());
        }
    }

    pub async fn broadcast_to_many(&self, user_ids: &[Uuid], message: &str) {
        let users = self.users.read().await;
        let msg = message.to_string();
        for uid in user_ids {
            if let Some(tx) = users.get(uid) {
                let _ = tx.send(msg.clone());
            }
        }
    }

    pub async fn broadcast_all(&self, message: &str) {
        let users = self.users.read().await;
        let msg = message.to_string();
        for tx in users.values() {
            let _ = tx.send(msg.clone());
        }
    }
}
