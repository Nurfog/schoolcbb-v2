use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rut::Rut;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CondicionMatricula {
    #[serde(rename = "AL")]
    AlumnoRegular,
    #[serde(rename = "RE")]
    Repitente,
    #[serde(rename = "TR")]
    Trasladado,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Prioritario {
    #[serde(rename = "1")]
    Si,
    #[serde(rename = "2")]
    Preferente,
    #[serde(rename = "0")]
    No,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NEE {
    #[serde(rename = "T")]
    Transitoria,
    #[serde(rename = "P")]
    Permanente,
    #[serde(rename = "N")]
    No,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: Uuid,
    pub rut: Rut,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub grade_level: String,
    pub section: String,
    pub cod_nivel: Option<String>,
    pub condicion: CondicionMatricula,
    pub prioritario: Prioritario,
    pub nee: NEE,
    pub enrolled: bool,
}

impl Student {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn attendance_threshold(&self) -> f64 {
        match self.nee {
            NEE::No => crate::attendance::THRESHOLD_ASISTENCIA_GENERAL,
            _ => crate::attendance::THRESHOLD_ASISTENCIA_NEE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: Uuid,
    pub name: String,
    pub subject: String,
    pub grade_level: String,
    pub section: String,
    pub teacher_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub year: i32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalInfo {
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianRelationship {
    pub id: Uuid,
    pub student_id: Uuid,
    pub guardian_user_id: Uuid,
    pub guardian_name: String,
    pub guardian_rut: String,
    pub relationship: String,
    pub authorized_pickup: bool,
    pub receives_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStudentPayload {
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub grade_level: String,
    pub section: String,
    pub cod_nivel: Option<String>,
    pub condicion: Option<String>,
    pub prioritario: Option<String>,
    pub nee: Option<String>,
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStudentPayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub grade_level: Option<String>,
    pub section: Option<String>,
    pub cod_nivel: Option<String>,
    pub condicion: Option<String>,
    pub prioritario: Option<String>,
    pub nee: Option<String>,
    pub diseases: Option<String>,
    pub allergies: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub emergency_contact_relation: Option<String>,
}
