use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mensaje interno entre usuarios del sistema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub subject: String,
    pub body: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Destinatario de un mensaje masivo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "id")]
pub enum AudienceTarget {
    /// Usuario específico por su ID.
    User(Uuid),
    /// Todos los integrantes de un curso.
    Course(Uuid),
    /// Todos los estudiantes.
    AllStudents,
    /// Todos los profesores.
    AllTeachers,
    /// Todo el personal del establecimiento.
    AllStaff,
}

/// Payload para crear y enviar un mensaje.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessagePayload {
    pub audience: AudienceTarget,
    pub subject: String,
    pub body: String,
}

/// Conteo de mensajes de un usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCount {
    pub total: i64,
    pub unread: i64,
}

/// Registro de entrevista o reunión entre un profesor y un estudiante/apoderado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct InterviewLog {
    pub id: Uuid,
    pub student_id: Uuid,
    pub teacher_id: Uuid,
    pub date: NaiveDate,
    pub reason: String,
    pub notes: String,
    pub follow_up: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una nueva entrevista.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInterviewPayload {
    pub student_id: Uuid,
    pub reason: String,
    pub notes: String,
    pub follow_up: Option<String>,
    pub date: Option<NaiveDate>,
}

/// Payload para actualizar una entrevista existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInterviewPayload {
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub follow_up: Option<String>,
}
