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
        matches!(
            self,
            AttendanceStatus::Justificado | AttendanceStatus::Licencia
        )
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Ausente" => AttendanceStatus::Ausente,
            "Atraso" => AttendanceStatus::Atraso,
            "Justificado" => AttendanceStatus::Justificado,
            "Licencia" => AttendanceStatus::Licencia,
            _ => AttendanceStatus::Presente,
        }
    }
}

impl std::str::FromStr for AttendanceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AttendanceStatus::from_str(s))
    }
}

impl AttendanceStatus {

    pub fn as_str(&self) -> &'static str {
        match self {
            AttendanceStatus::Presente => "Presente",
            AttendanceStatus::Ausente => "Ausente",
            AttendanceStatus::Atraso => "Atraso",
            AttendanceStatus::Justificado => "Justificado",
            AttendanceStatus::Licencia => "Licencia",
        }
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
pub struct CreateAttendancePayload {
    pub student_id: Uuid,
    pub course_id: Uuid,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub status: String,
    pub subject: String,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAttendancePayload {
    pub status: Option<String>,
    pub time: Option<NaiveTime>,
    pub observation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAttendanceEntry {
    pub course_id: Uuid,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub subject: String,
    pub teacher_id: Uuid,
    pub records: Vec<StudentAttendanceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAttendanceRecord {
    pub student_id: Uuid,
    pub status: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyAttendanceSummary {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub year: i32,
    pub months: Vec<MonthlyAttendanceSummary>,
    pub total_days: i32,
    pub present: i32,
    pub absent: i32,
    pub late: i32,
    pub justified: i32,
    pub attendance_percentage: f64,
    pub is_below_general_threshold: bool,
    pub is_below_nee_threshold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseAttendanceReport {
    pub course_id: Uuid,
    pub course_name: String,
    pub date: NaiveDate,
    pub subject: String,
    pub total_students: i32,
    pub present_count: i32,
    pub absent_count: i32,
    pub late_count: i32,
    pub justified_count: i32,
    pub records: Vec<StudentAttendanceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupereducExportRow {
    pub rut: String,
    pub student_name: String,
    pub grade_level: String,
    pub section: String,
    pub total_days: i32,
    pub present: i32,
    pub absent: i32,
    pub late: i32,
    pub justified: i32,
    pub attendance_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attendance_status_presente() {
        let s = AttendanceStatus::Presente;
        assert!(s.es_asistencia());
        assert!(!s.es_ausencia());
        assert!(!s.es_justificado());
        assert_eq!(s.as_str(), "Presente");
    }

    #[test]
    fn test_attendance_status_ausente() {
        let s = AttendanceStatus::Ausente;
        assert!(!s.es_asistencia());
        assert!(s.es_ausencia());
        assert!(!s.es_justificado());
        assert_eq!(s.as_str(), "Ausente");
    }

    #[test]
    fn test_attendance_status_justificado() {
        let s = AttendanceStatus::Justificado;
        assert!(s.es_justificado());
        assert_eq!(s.as_str(), "Justificado");
    }

    #[test]
    fn test_attendance_status_licencia_es_justificado() {
        let s = AttendanceStatus::Licencia;
        assert!(s.es_justificado());
        assert_eq!(s.as_str(), "Licencia");
    }

    #[test]
    fn test_from_str_defaults_to_presente() {
        assert_eq!(
            AttendanceStatus::from_str("Desconocido"),
            AttendanceStatus::Presente
        );
    }

    #[test]
    fn test_from_str_parses_all() {
        assert_eq!(
            AttendanceStatus::from_str("Ausente"),
            AttendanceStatus::Ausente
        );
        assert_eq!(
            AttendanceStatus::from_str("Atraso"),
            AttendanceStatus::Atraso
        );
        assert_eq!(
            AttendanceStatus::from_str("Justificado"),
            AttendanceStatus::Justificado
        );
        assert_eq!(
            AttendanceStatus::from_str("Licencia"),
            AttendanceStatus::Licencia
        );
    }

    #[test]
    fn test_monthly_summary_percentage() {
        let s = MonthlyAttendanceSummary {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            rut: "1-9".into(),
            year: 2025,
            month: 3,
            total_days: 20,
            present: 18,
            absent: 1,
            late: 1,
            justified: 0,
        };
        assert!((s.attendance_percentage() - 90.0).abs() < f64::EPSILON);
        assert!(!s.is_below_threshold(85.0));
        assert!(!s.is_below_threshold(90.0));
        assert!(s.is_below_threshold(95.0));
    }

    #[test]
    fn test_monthly_summary_zero_days() {
        let s = MonthlyAttendanceSummary {
            student_id: Uuid::nil(),
            student_name: "Test".into(),
            rut: "1-9".into(),
            year: 2025,
            month: 3,
            total_days: 0,
            present: 0,
            absent: 0,
            late: 0,
            justified: 0,
        };
        assert!((s.attendance_percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_thresholds_constants() {
        assert!((THRESHOLD_ASISTENCIA_GENERAL - 85.0).abs() < f64::EPSILON);
        assert!((THRESHOLD_ASISTENCIA_NEE - 75.0).abs() < f64::EPSILON);
    }
}
