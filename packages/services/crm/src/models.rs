use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub school_id: Option<String>,
    pub corporation_id: Option<String>,
    pub admin_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesStage {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub is_final: bool,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesProspect {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProspectPayload {
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProspectPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub source: Option<String>,
    pub requirements: Option<serde_json::Value>,
    pub current_stage_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub estimated_value: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesActivity {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub is_completed: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivityPayload {
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesContract {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub tax_id: Option<String>,
    pub plan_id: Option<Uuid>,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: f64,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub activated_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractPayload {
    pub prospect_id: Uuid,
    pub plan_id: Uuid,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SalesDocument {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub file_name: String,
    pub file_url: Option<String>,
    pub doc_type: String,
    pub is_verified: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentPayload {
    pub contract_id: Uuid,
    pub file_name: String,
    pub doc_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SalesProposal {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub modules: Option<serde_json::Value>,
    pub total_value: f64,
    pub discount: f64,
    pub version: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
