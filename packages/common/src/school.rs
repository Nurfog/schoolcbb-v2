use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Representante legal (sostenedor) de una corporación o colegio.
/// Una corporación puede tener múltiples representantes; un colegio también.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct LegalRepresentative {
    pub id: Uuid,
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para crear un representante legal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLegalRepPayload {
    pub corporation_id: Option<Uuid>,
    pub school_id: Option<Uuid>,
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

/// Payload para actualizar un representante legal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLegalRepPayload {
    pub rut: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub active: Option<bool>,
}

/// Corporación o sostenedor educativo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Corporation {
    pub id: Uuid,
    pub name: String,
    pub rut: Option<String>,
    pub logo_url: Option<String>,
    pub legal_representative_name: Option<String>,
    pub legal_representative_rut: Option<String>,
    pub legal_representative_email: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una nueva corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCorporationPayload {
    pub name: String,
    pub rut: Option<String>,
    pub logo_url: Option<String>,
    pub legal_representative_name: Option<String>,
    pub legal_representative_rut: Option<String>,
    pub legal_representative_email: Option<String>,
}

/// Establecimiento educacional perteneciente a una corporación.
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

/// Payload para actualizar una corporación existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorporationPayload {
    pub name: Option<String>,
    pub rut: Option<String>,
    pub logo_url: Option<String>,
    pub legal_representative_name: Option<String>,
    pub legal_representative_rut: Option<String>,
    pub legal_representative_email: Option<String>,
}

/// Payload para actualizar un establecimiento existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSchoolPayload {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub logo_url: Option<String>,
}

/// Payload para crear un nuevo establecimiento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchoolPayload {
    pub corporation_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
}
