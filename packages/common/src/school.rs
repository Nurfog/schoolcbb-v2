use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Corporation {
    pub id: Uuid,
    pub name: String,
    pub rut: Option<String>,
    pub logo_url: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCorporationPayload {
    pub name: String,
    pub rut: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct School {
    pub id: Uuid,
    pub corporation_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub logo_url: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchoolPayload {
    pub corporation_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
}
