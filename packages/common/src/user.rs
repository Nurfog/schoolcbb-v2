use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rut::Rut;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Sostenedor,
    Director,
    UTP,
    Administrador,
    Profesor,
    Apoderado,
    Alumno,
}

impl UserRole {
    pub fn es_admin(&self) -> bool {
        matches!(self, UserRole::Administrador | UserRole::Sostenedor | UserRole::Director)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Sostenedor => "Sostenedor",
            UserRole::Director => "Director",
            UserRole::UTP => "UTP",
            UserRole::Administrador => "Administrador",
            UserRole::Profesor => "Profesor",
            UserRole::Apoderado => "Apoderado",
            UserRole::Alumno => "Alumno",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Sostenedor" => Some(UserRole::Sostenedor),
            "Director" => Some(UserRole::Director),
            "UTP" => Some(UserRole::UTP),
            "Administrador" => Some(UserRole::Administrador),
            "Profesor" => Some(UserRole::Profesor),
            "Apoderado" => Some(UserRole::Apoderado),
            "Alumno" => Some(UserRole::Alumno),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub rut: Rut,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub rut: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_students: i64,
    pub total_teachers: i64,
    pub attendance_today_percentage: f64,
    pub pending_alerts: i64,
    pub today_events: Vec<AgendaEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub date: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Clase,
    Reunion,
    Evaluacion,
    Evento,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceTodayWidget {
    pub date: String,
    pub total_students: i64,
    pub present: i64,
    pub absent: i64,
    pub late: i64,
    pub justified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertWidget {
    pub alerts: Vec<AttendanceAlert>,
}

use crate::attendance::AttendanceAlert;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaWidget {
    pub events: Vec<AgendaEvent>,
}
