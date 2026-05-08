use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct PipelineStage {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub is_final: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStagePayload {
    pub name: String,
    pub sort_order: Option<i32>,
    pub is_final: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStagePayload {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
    pub is_final: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Prospect {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub current_stage_id: Option<Uuid>,
    pub assigned_user_id: Option<Uuid>,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProspectPayload {
    pub first_name: String,
    pub last_name: String,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProspectPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub rut: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct ProspectActivity {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<NaiveDateTime>,
    pub is_completed: bool,
    pub created_by: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivityPayload {
    pub prospect_id: Uuid,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct ProspectDocument {
    pub id: Uuid,
    pub prospect_id: Uuid,
    pub file_name: String,
    pub s3_url: Option<String>,
    pub doc_type: String,
    pub is_verified: bool,
    pub uploaded_by: Option<Uuid>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentPayload {
    pub prospect_id: Uuid,
    pub file_name: String,
    pub doc_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Classroom {
    pub id: Uuid,
    pub name: String,
    pub capacity: i32,
    pub location: Option<String>,
    pub active: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClassroomPayload {
    pub name: String,
    pub capacity: i32,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClassroomPayload {
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub location: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacancyCheckResult {
    pub grade_level: String,
    pub total_capacity: i32,
    pub enrolled_count: i32,
    pub available: i32,
}
