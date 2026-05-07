use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rut::Rut;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Administrador,
    Profesor,
    Apoderado,
    Alumno,
}

impl UserRole {
    pub fn es_admin(&self) -> bool {
        matches!(self, UserRole::Administrador)
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
pub struct AuthResponse {
    pub token: String,
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
