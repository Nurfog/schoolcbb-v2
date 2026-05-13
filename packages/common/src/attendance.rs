use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Estado de asistencia diaria de un estudiante.
///
/// Los códigos entre paréntesis corresponden al formato de exportación SIGE:
///
/// | Variante      | SIGE | Descripción                         |
/// |---------------|------|-------------------------------------|
/// | `Presente`    | A    | Asistió a clases                    |
/// | `Ausente`     | F    | No asistió (falta injustificada)    |
/// | `Atraso`      | ATR  | Llegó tarde                         |
/// | `Justificado` | J    | Falta justificada por el apoderado  |
/// | `Licencia`    | L    | Licencia médica                     |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum AttendanceStatus {
    /// Asistió a clases (SIGE: A).
    Presente,
    /// No asistió (SIGE: F).
    Ausente,
    /// Llegó tarde (SIGE: ATR).
    Atraso,
    /// Falta justificada por apoderado (SIGE: J).
    Justificado,
    /// Licencia médica (SIGE: L).
    Licencia,
}

impl AttendanceStatus {
    /// Retorna `true` si el estudiante estuvo presente.
    pub fn es_asistencia(&self) -> bool {
        matches!(self, AttendanceStatus::Presente)
    }

    /// Retorna `true` si el estudiante estuvo ausente sin justificación.
    pub fn es_ausencia(&self) -> bool {
        matches!(self, AttendanceStatus::Ausente)
    }

    /// Retorna `true` si la ausencia está justificada o corresponde a licencia médica.
    pub fn es_justificado(&self) -> bool {
        matches!(
            self,
            AttendanceStatus::Justificado | AttendanceStatus::Licencia
        )
    }

    /// Parsea un estado desde su nombre en español (`"Presente"`, `"Ausente"`, etc.).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Presente" => Some(AttendanceStatus::Presente),
            "Ausente" => Some(AttendanceStatus::Ausente),
            "Atraso" => Some(AttendanceStatus::Atraso),
            "Justificado" => Some(AttendanceStatus::Justificado),
            "Licencia" => Some(AttendanceStatus::Licencia),
            _ => None,
        }
    }
}

impl std::str::FromStr for AttendanceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AttendanceStatus::from_str(s).ok_or_else(|| format!("Estado de asistencia inválido: {s}"))
    }
}

impl AttendanceStatus {

    /// Retorna el nombre del estado en español (`"Presente"`, `"Ausente"`, etc.).
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

/// Registro de asistencia diaria de un estudiante.
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

/// Payload para crear un registro de asistencia.
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

/// Payload para actualizar un registro de asistencia existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAttendancePayload {
    pub status: Option<String>,
    pub time: Option<NaiveTime>,
    pub observation: Option<String>,
}

/// Entrada de carga masiva de asistencia para un curso completo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAttendanceEntry {
    pub course_id: Uuid,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub subject: String,
    pub teacher_id: Uuid,
    /// Lista de registros individuales por estudiante.
    pub records: Vec<StudentAttendanceRecord>,
}

/// Registro de asistencia individual dentro de una carga masiva.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAttendanceRecord {
    pub student_id: Uuid,
    pub status: String,
    pub observation: Option<String>,
}

/// Resumen de asistencia mensual de un estudiante.
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
    /// Porcentaje de asistencia del mes: `present / total_days * 100`.
    /// Si `total_days` es 0, retorna 100.
    pub fn attendance_percentage(&self) -> f64 {
        if self.total_days == 0 {
            return 100.0;
        }
        (self.present as f64 / self.total_days as f64) * 100.0
    }

    /// Retorna `true` si el porcentaje de asistencia es menor al umbral dado.
    pub fn is_below_threshold(&self, threshold: f64) -> bool {
        self.attendance_percentage() < threshold
    }
}

/// Umbral mínimo de asistencia para estudiantes sin NEE (85%).
pub const THRESHOLD_ASISTENCIA_GENERAL: f64 = 85.0;

/// Umbral mínimo de asistencia para estudiantes con NEE (75%).
pub const THRESHOLD_ASISTENCIA_NEE: f64 = 75.0;

/// Alerta generada cuando la asistencia de un estudiante cae bajo cierto umbral.
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

/// Nivel de severidad de una alerta de asistencia.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum AlertSeverity {
    /// Riesgo bajo (primeras faltas).
    Bajo,
    /// Riesgo medio.
    Medio,
    /// Riesgo alto (posible repitencia).
    Alto,
}

/// Resumen de asistencia anual completo de un estudiante.
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

/// Reporte de asistencia de un curso en una fecha y asignatura determinada.
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

/// Fila de exportación de asistencia para plataforma Supereduc/SIGE.
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
    fn test_from_str_returns_none_for_unknown() {
        assert_eq!(AttendanceStatus::from_str("Desconocido"), None);
    }

    #[test]
    fn test_from_str_parses_all() {
        assert_eq!(AttendanceStatus::from_str("Ausente"), Some(AttendanceStatus::Ausente));
        assert_eq!(AttendanceStatus::from_str("Atraso"), Some(AttendanceStatus::Atraso));
        assert_eq!(AttendanceStatus::from_str("Justificado"), Some(AttendanceStatus::Justificado));
        assert_eq!(AttendanceStatus::from_str("Licencia"), Some(AttendanceStatus::Licencia));
        assert_eq!(AttendanceStatus::from_str("Presente"), Some(AttendanceStatus::Presente));
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

    #[test]
    fn test_from_str_empty_returns_none() {
        assert_eq!(AttendanceStatus::from_str(""), None);
        assert_eq!(AttendanceStatus::from_str(" "), None);
    }

    #[test]
    fn test_alert_severity_eq() {
        assert_eq!(AlertSeverity::Bajo, AlertSeverity::Bajo);
        assert_eq!(AlertSeverity::Medio, AlertSeverity::Medio);
        assert_eq!(AlertSeverity::Alto, AlertSeverity::Alto);
        assert_ne!(AlertSeverity::Bajo, AlertSeverity::Alto);
    }

    #[test]
    fn test_attendance_alert_construction() {
        let alert = AttendanceAlert {
            student_id: Uuid::nil(),
            student_name: "Juan Pérez".into(),
            rut: "1-9".into(),
            month: 3,
            year: 2025,
            attendance_percentage: 70.0,
            total_absences: 6,
            severity: AlertSeverity::Alto,
        };
        assert_eq!(alert.student_name, "Juan Pérez");
        assert!(alert.severity == AlertSeverity::Alto);
    }

    #[test]
    fn test_yearly_summary_construction() {
        let monthly = MonthlyAttendanceSummary {
            student_id: Uuid::nil(),
            student_name: "María".into(),
            rut: "1-9".into(),
            year: 2025,
            month: 3,
            total_days: 20,
            present: 18,
            absent: 1,
            late: 1,
            justified: 0,
        };
        let summary = YearlyAttendanceSummary {
            student_id: Uuid::nil(),
            student_name: "María".into(),
            rut: "1-9".into(),
            year: 2025,
            months: vec![monthly],
            total_days: 20,
            present: 18,
            absent: 1,
            late: 1,
            justified: 0,
            attendance_percentage: 90.0,
            is_below_general_threshold: false,
            is_below_nee_threshold: false,
        };
        assert!((summary.attendance_percentage - 90.0).abs() < f64::EPSILON);
        assert!(!summary.is_below_general_threshold);
    }

    #[test]
    fn test_from_str_trait_impl() {
        let parsed: AttendanceStatus = "Presente".parse().unwrap();
        assert_eq!(parsed, AttendanceStatus::Presente);
        let parsed: Result<AttendanceStatus, _> = "Desconocido".parse();
        assert!(parsed.is_err());
    }
}
