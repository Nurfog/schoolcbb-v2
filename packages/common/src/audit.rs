use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub user_id: Option<Uuid>,
    pub changes: Option<Value>,
}

#[cfg(feature = "db")]
pub async fn log(pool: &sqlx::PgPool, entry: &AuditEntry) {
    let id = Uuid::new_v4();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, entity_type, entity_id, action, user_id, changes)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(id)
    .bind(&entry.entity_type)
    .bind(entry.entity_id)
    .bind(&entry.action)
    .bind(entry.user_id)
    .bind(&entry.changes)
    .execute(pool)
    .await;
}
