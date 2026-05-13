use serde_json::Value;
use uuid::Uuid;

/// Entrada de auditoría que registra un cambio sobre una entidad del sistema.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    /// Tipo de entidad afectada (ej: `"student"`, `"course"`).
    pub entity_type: String,
    /// ID de la entidad afectada.
    pub entity_id: Uuid,
    /// Acción realizada (ej: `"create"`, `"update"`, `"delete"`).
    pub action: String,
    /// Usuario que realizó la acción (si aplica).
    pub user_id: Option<Uuid>,
    /// Cambios en formato JSON (valores anterior y nuevo).
    pub changes: Option<Value>,
}

/// Escribe una entrada de auditoría en la base de datos.
#[cfg(feature = "db")]
pub async fn log(pool: &sqlx::PgPool, entry: &AuditEntry) {
    let id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
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
    .await
    {
        tracing::error!("Failed to write audit log: {e}");
    }
}
