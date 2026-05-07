use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttendanceStatus {
    Presente,    // SIGE: A
    Ausente,     // SIGE: F
    Atraso,      // SIGE: ATR
    Justificado, // SIGE: J
    Licencia,    // SIGE: L
}

impl AttendanceStatus {
    pub fn es_asistencia(&self) -> bool {
        matches!(self, AttendanceStatus::Presente)
    }

    pub fn es_ausencia(&self) -> bool {
        matches!(self, AttendanceStatus::Ausente)
    }

    pub fn es_justificado(&self) -> bool {
        matches!(self, AttendanceStatus::Justificado | AttendanceStatus::Licencia)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyAttendance {
    pub id: Uuid,
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub status: AttendanceStatus,
    pub subject: String,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAttendanceSummary {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub year: i32,
    pub month: u32,
    pub total_days: i32,
    pub present: i32,
    pub absent: i32,
    pub late: i32,
    pub justified: i32,
}

impl MonthlyAttendanceSummary {
    pub fn attendance_percentage(&self) -> f64 {
        if self.total_days == 0 {
            return 100.0;
        }
        (self.present as f64 / self.total_days as f64) * 100.0
    }

    pub fn is_below_threshold(&self, threshold: f64) -> bool {
        self.attendance_percentage() < threshold
    }
}

pub const THRESHOLD_ASISTENCIA_GENERAL: f64 = 85.0;
pub const THRESHOLD_ASISTENCIA_NEE: f64 = 75.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceAlert {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub month: u32,
    pub year: i32,
    pub attendance_percentage: f64,
    pub total_absences: i32,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Bajo,
    Medio,
    Alto,
}
