use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Datos de un empleado o funcionario del establecimiento.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Employee {
    pub id: Uuid,
    pub school_id: Option<Uuid>,
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// Cargo o puesto de trabajo.
    pub position: Option<String>,
    pub hire_date: Option<NaiveDate>,
    /// Categoría laboral (docente, asistente, etc.).
    pub category: Option<String>,
    /// Días de vacaciones disponibles.
    pub vacation_days_available: f32,
    pub active: bool,
    pub supervisor_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para crear un nuevo empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmployeePayload {
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub category: Option<String>,
    pub hire_date: Option<NaiveDate>,
}

/// Payload para actualizar datos de un empleado existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmployeePayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub category: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub vacation_days_available: Option<f32>,
}

/// Contrato laboral de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeeContract {
    pub id: Uuid,
    pub employee_id: Uuid,
    /// Tipo de contrato (planta, honorarios, etc.).
    pub contract_type: String,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub salary_base: f64,
    pub weekly_hours: i32,
    /// Indica si se firmó el anexo de la Ley Karin.
    pub ley_karin_signed: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear un nuevo contrato laboral.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractPayload {
    pub employee_id: Uuid,
    pub contract_type: String,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub salary_base: f64,
    pub weekly_hours: i32,
    pub ley_karin_signed: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

/// Documento asociado a la carpeta de un empleado (contrato, anexo, certificado).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeeDocument {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub doc_type: String,
    pub file_name: String,
    pub file_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Tipo de marcación en el registro de asistencia de empleados.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum EntryType {
    /// Entrada a la jornada laboral.
    Entrada,
    /// Salida a colación (almuerzo).
    SalidaColacion,
    /// Retorno desde colación.
    RetornoColacion,
    /// Salida definitiva de la jornada.
    Salida,
}

impl EntryType {
    /// Retorna el nombre legible del tipo de marcación.
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Entrada => "Entrada",
            EntryType::SalidaColacion => "Salida Colacion",
            EntryType::RetornoColacion => "Retorno Colacion",
            EntryType::Salida => "Salida",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Entrada" => Some(EntryType::Entrada),
            "Salida Colacion" | "SalidaColacion" => Some(EntryType::SalidaColacion),
            "Retorno Colacion" | "RetornoColacion" => Some(EntryType::RetornoColacion),
            "Salida" => Some(EntryType::Salida),
            _ => None,
        }
    }
}

/// Error de cumplimiento normativo en la jornada laboral.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum ComplianceError {
    /// La jornada diaria excede el máximo legal.
    ExcesoJornadaDiaria { max_hours: i32, actual_hours: f64 },
    /// La jornada semanal excede el máximo legal.
    ExcesoSemanal { max_hours: i32, actual_hours: f64 },
    /// El descanso entre jornadas es insuficiente.
    DescansoInsuficiente { min_hours: i32, actual_hours: f64 },
    /// Inconsistencia en los registros (ej: salida sin entrada).
    Inconsistencia { detail: String },
}

/// Validador de cumplimiento de jornada laboral y descansos.
///
/// Revisa los registros de marcación para detectar excesos de jornada
/// diaria/semanal y descansos insuficientes entre jornadas.
#[derive(Debug, Clone)]
pub struct AttendanceValidator;

impl AttendanceValidator {
    /// Valida el cumplimiento normativo de una lista de marcaciones.
    ///
    /// Retorna una lista de errores de cumplimiento encontrados.
    pub fn validate_compliance(
        &self,
        logs: &[AttendanceLog],
        max_daily_hours: i32,
        max_weekly_hours: i32,
        min_rest_hours: i32,
    ) -> Vec<ComplianceError> {
        let mut errors = vec![];

        let mut day_groups: std::collections::HashMap<chrono::NaiveDate, Vec<&AttendanceLog>> =
            std::collections::HashMap::new();
        for log in logs {
            day_groups
                .entry(log.timestamp.date())
                .or_default()
                .push(log);
        }

        for day_logs in day_groups.values() {
            if let (Some(first), Some(last)) = (day_logs.first(), day_logs.last()) {
                let hours = (last.timestamp - first.timestamp).num_minutes() as f64 / 60.0;
                if hours > max_daily_hours as f64 {
                    errors.push(ComplianceError::ExcesoJornadaDiaria {
                        max_hours: max_daily_hours,
                        actual_hours: hours,
                    });
                }
            }

            let has_entry = day_logs.iter().any(|l| l.entry_type.as_str() == "Entrada");
            let has_exit = day_logs.iter().any(|l| l.entry_type.as_str() == "Salida");
            if has_entry && !has_exit {
                errors.push(ComplianceError::Inconsistencia {
                    detail: "Entrada sin salida registrada".into(),
                });
            }
            if !has_entry && has_exit {
                errors.push(ComplianceError::Inconsistencia {
                    detail: "Salida sin entrada registrada".into(),
                });
            }
        }

        let mut same_day_pairs: Vec<(&AttendanceLog, &AttendanceLog)> = Vec::new();
        for logs in day_groups.values() {
            if logs.len() >= 2 {
                for i in 1..logs.len() {
                    same_day_pairs.push((logs[i - 1], logs[i]));
                }
            }
        }
        for (prev, curr) in &same_day_pairs {
            let diff = (curr.timestamp - prev.timestamp).num_minutes() as f64 / 60.0;
            if diff < min_rest_hours as f64 && diff > 0.0 {
                errors.push(ComplianceError::DescansoInsuficiente {
                    min_hours: min_rest_hours,
                    actual_hours: diff,
                });
            }
        }

        let mut sorted_dates: Vec<_> = day_groups.keys().collect();
        sorted_dates.sort();
        for window in sorted_dates.windows(2) {
            let prev_date = window[0];
            let next_date = window[1];
            let prev_logs = &day_groups[prev_date];
            let next_logs = &day_groups[next_date];
            let last_exit = prev_logs
                .iter()
                .filter(|l| l.entry_type.as_str() == "Salida")
                .max_by_key(|l| l.timestamp);
            let first_entry = next_logs
                .iter()
                .filter(|l| l.entry_type.as_str() == "Entrada")
                .min_by_key(|l| l.timestamp);
            if let (Some(exit), Some(entry)) = (last_exit, first_entry) {
                let gap = (entry.timestamp - exit.timestamp).num_minutes() as f64 / 60.0;
                if gap < min_rest_hours as f64 {
                    errors.push(ComplianceError::DescansoInsuficiente {
                        min_hours: min_rest_hours,
                        actual_hours: gap,
                    });
                }
            }
        }

        let weekly_hours: f64 = day_groups
            .values()
            .filter_map(|logs| {
                let first = logs.first()?;
                let last = logs.last()?;
                Some((last.timestamp - first.timestamp).num_minutes() as f64 / 60.0)
            })
            .sum();
        if weekly_hours > max_weekly_hours as f64 {
            errors.push(ComplianceError::ExcesoSemanal {
                max_hours: max_weekly_hours,
                actual_hours: weekly_hours,
            });
        }

        errors
    }
}

/// Registro individual de marcación de asistencia de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AttendanceLog {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub entry_type: String,
    pub device_id: Option<String>,
    pub location_hash: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Payload para registrar una nueva marcación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceLogPayload {
    pub employee_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub entry_type: String,
    pub device_id: Option<String>,
    pub location_hash: Option<String>,
}

/// Registro de modificación a una marcación de asistencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AttendanceModification {
    pub id: Uuid,
    pub attendance_id: Uuid,
    pub original_value: String,
    pub new_value: String,
    pub reason: String,
    pub modified_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Payload para modificar una marcación existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceModificationPayload {
    pub attendance_id: Uuid,
    pub new_timestamp: NaiveDateTime,
    pub new_entry_type: String,
    pub reason: String,
}

/// Resumen diario de asistencia de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub employee_id: Uuid,
    pub first_entry: Option<NaiveDateTime>,
    pub last_exit: Option<NaiveDateTime>,
    pub total_hours: f64,
    pub nightly_rest_compliant: bool,
    pub weekly_hours_limit_exceeded: bool,
}

/// Solicitud de permiso o licencia de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    /// Tipo de permiso (vacaciones, licencia médica, personal, etc.).
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    /// Estado de la solicitud (pendiente, aprobada, rechazada).
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para crear una nueva solicitud de permiso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeavePayload {
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

/// Payload para aprobar o rechazar una solicitud de permiso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveApprovalPayload {
    pub status: String,
    pub approved_by: Uuid,
}

/// Denuncia o reclamo interno (Ley Karin).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Complaint {
    pub id: Uuid,
    pub complainant_name: Option<String>,
    pub complainant_email: Option<String>,
    pub accused_rut: Option<String>,
    /// Tipo de denuncia (acoso laboral, acoso sexual, maltrato, etc.).
    pub complaint_type: String,
    pub description: String,
    /// Estado de la denuncia (recibida, investigando, resuelta, cerrada).
    pub status: String,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload para crear una nueva denuncia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComplaintPayload {
    pub complainant_name: Option<String>,
    pub complainant_email: Option<String>,
    pub accused_rut: Option<String>,
    pub complaint_type: String,
    pub description: String,
}

/// Payload para resolver una denuncia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveComplaintPayload {
    pub status: String,
    pub resolution: String,
}

/// Administradora de Fondos de Pensiones (AFP) del empleado.
///
/// Cada variante incluye la tasa de comisión actual:
///
/// | AFP       | Comisión |
/// |-----------|----------|
/// | Capital   | 11,44%   |
/// | Cuprum    | 11,44%   |
/// | Habitat   | 11,27%   |
/// | Planvital | 11,02%   |
/// | Provida   | 11,45%   |
/// | Modelo    | 10,58%   |
/// | Uno       | 10,87%   |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum PensionFund {
    Capital,
    Cuprum,
    Habitat,
    Planvital,
    Provida,
    Modelo,
    Uno,
}

impl PensionFund {
    /// Retorna la tasa de comisión vigente de la AFP (ej: `0.1144` para Capital).
    pub fn commission_rate(&self) -> f64 {
        match self {
            PensionFund::Capital => 0.1144,
            PensionFund::Cuprum => 0.1144,
            PensionFund::Habitat => 0.1127,
            PensionFund::Planvital => 0.1102,
            PensionFund::Provida => 0.1145,
            PensionFund::Modelo => 0.1058,
            PensionFund::Uno => 0.1087,
        }
    }
}

impl PensionFund {
    /// Parsea una AFP desde su nombre (`"Capital"`, `"Cuprum"`, etc.).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Capital" => Some(PensionFund::Capital),
            "Cuprum" => Some(PensionFund::Cuprum),
            "Habitat" => Some(PensionFund::Habitat),
            "Planvital" => Some(PensionFund::Planvital),
            "Provida" => Some(PensionFund::Provida),
            "Modelo" => Some(PensionFund::Modelo),
            "Uno" => Some(PensionFund::Uno),
            _ => None,
        }
    }
}

impl std::fmt::Display for PensionFund {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Sistema de salud del empleado.
///
/// - `Fonasa`: sistema público, descuento fijo de 7%.
/// - `Isapre`: sistema privado, con nombre del plan y mijo fijo pactado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HealthSystem {
    /// Fondo Nacional de Salud (descuento 7% sobre renta imponible).
    Fonasa,
    /// Institución de Salud Previsional (monto fijo pactado).
    Isapre {
        plan_name: String,
        #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
        fixed_amount: f64,
    },
}

impl HealthSystem {
    #[allow(clippy::should_implement_trait)]
    /// Parse a health system from a stored string.
    /// "Fonasa" returns Some(Fonasa).
    /// Isapre plans require external data (plan_name, fixed_amount) and are
    /// not constructed from a single string — returns None for Isapre strings
    /// to signal that the caller must provide the associated data separately.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Fonasa" => Some(HealthSystem::Fonasa),
            _ => None,
        }
    }
}

impl std::fmt::Display for HealthSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthSystem::Fonasa => write!(f, "Fonasa"),
            HealthSystem::Isapre { plan_name, .. } => write!(f, "Isapre ({plan_name})"),
        }
    }
}

/// Asociación entre un empleado y su AFP / sistema de salud.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeePensionFund {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub pension_fund: String,
    pub health_system: String,
    pub health_plan_name: Option<String>,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub health_fixed_amount: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// Registro de liquidación de sueldo de un empleado para un mes y año.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Payroll {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub salary_base: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub gratificacion: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub non_taxable_earnings: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub taxable_income: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub afp_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub health_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub unemployment_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub income_tax: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub other_deductions: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub net_salary: f64,
    /// Indica si fue exportado al libro de remuneraciones electrónico (LRE).
    pub lre_exported: bool,
    /// Indica si fue exportado a Previred.
    pub previred_exported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Línea individual del detalle de una liquidación de sueldo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollLineItem {
    pub concept: String,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub amount: f64,
    /// Categoría del ítem: `"Imponible"`, `"No Imponible"`, `"Descuento Legal"`, `"Descuento"`.
    pub category: String,
}

/// Resultado del cálculo de una liquidación de sueldo, con desglose completo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollCalculation {
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub salary_base: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub gratificacion: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub non_taxable_earnings: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub taxable_income: f64,
    pub afp_rate: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub afp_discount: f64,
    pub health_rate: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub health_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub unemployment_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub income_tax: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub other_deductions: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub net_salary: f64,
    pub breakdown: Vec<PayrollLineItem>,
}

/// Payload para solicitar el cálculo de una liquidación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollPayload {
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub non_taxable_earnings: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub other_deductions: f64,
}

/// Registro de nómina para exportación a Previred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviredRecord {
    pub rut: String,
    pub name: String,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub gross_salary: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub afp_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub health_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub unemployment_discount: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub net_salary: f64,
}

/// Cerca geográfica (geofence) asociada a un empleado para control de asistencia
/// por ubicación.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeeGeofence {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub lat: f64,
    pub lng: f64,
    pub radius_meters: f64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una nueva cerca geográfica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofencePayload {
    pub employee_id: Uuid,
    pub lat: f64,
    pub lng: f64,
    pub radius_meters: f64,
    pub name: String,
}

/// Licencia médica de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct MedicalLicense {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub license_type: String,
    pub folio: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: i32,
    pub diagnosis: Option<String>,
    pub status: String,
    pub file_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una nueva licencia médica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMedicalLicensePayload {
    pub employee_id: Uuid,
    pub license_type: String,
    pub folio: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub diagnosis: Option<String>,
}

/// Evaluación docente o de desempeño de un empleado.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct TeacherEvaluation {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub evaluator_id: Option<Uuid>,
    pub evaluation_type: String,
    pub score: Option<f64>,
    pub observations: Option<String>,
    pub period: Option<String>,
    pub year: i32,
    pub created_at: DateTime<Utc>,
}

/// Payload para crear una nueva evaluación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvaluationPayload {
    pub employee_id: Uuid,
    pub evaluation_type: String,
    pub score: Option<f64>,
    pub observations: Option<String>,
    pub period: Option<String>,
    pub year: i32,
}

/// Calculadora de liquidación de sueldo.
///
/// Permite configurar sueldo base, gratificación, tasa AFP y monto fijo
/// de Isapre para obtener el sueldo líquido.
#[derive(Debug, Clone)]
pub struct PayrollCalculator {
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub base_salary: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub gratificacion: f64,
    pub afp_rate: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub isapre_fixed_amount: Option<f64>,
}

impl PayrollCalculator {
    /// Crea una nueva calculadora con el sueldo base y valores por defecto
    /// (gratificación = 25% del base con tope de $500.000, AFP 10%).
    pub fn new(base_salary: f64) -> Self {
        Self {
            base_salary,
            gratificacion: (base_salary * 0.25).min(500000.0),
            afp_rate: 0.10,
            isapre_fixed_amount: None,
        }
    }

    /// Establece el monto de gratificación.
    pub fn with_gratificacion(mut self, grat: f64) -> Self {
        self.gratificacion = grat;
        self
    }

    /// Establece la tasa de cotización AFP.
    pub fn with_afp_rate(mut self, rate: f64) -> Self {
        self.afp_rate = rate;
        self
    }

    /// Establece el mijo fijo de Isapre.
    pub fn with_isapre(mut self, amount: f64) -> Self {
        self.isapre_fixed_amount = Some(amount);
        self
    }

    /// Calcula el sueldo líquido: imponible - AFP - salud - cesantía.
    pub fn calculate_liquid(&self) -> f64 {
        let taxable = self.base_salary + self.gratificacion;
        let afp_discount = taxable * self.afp_rate;
        let health = self.isapre_fixed_amount.unwrap_or(taxable * 0.07);
        let unemployment = taxable * 0.006;
        taxable - afp_discount - health - unemployment
    }
}

/// Calcula una liquidación de sueldo completa a partir del contrato, los
/// datos del empleado (AFP, salud) y el payload de entrada.
///
/// Retorna un [`PayrollCalculation`] con todos los haberes, descuentos legales
/// y el desglose por ítem.
pub fn calculate_payroll(
    contract: &EmployeeContract,
    payload: &PayrollPayload,
    pension_fund: &PensionFund,
    health_system: &HealthSystem,
    health_fixed_amount: Option<f64>,
) -> PayrollCalculation {
    let salary_base = contract.salary_base;
    let monthly_gratificacion = (salary_base * 0.25).min(500000.0);
    let non_taxable = payload.non_taxable_earnings;
    let taxable_income = salary_base + monthly_gratificacion;

    let afp_rate = 0.10;
    let afp_commission = match pension_fund {
        PensionFund::Capital | PensionFund::Cuprum => 0.1144,
        PensionFund::Habitat => 0.1127,
        PensionFund::Planvital => 0.1102,
        PensionFund::Provida => 0.1145,
        PensionFund::Modelo => 0.1058,
        PensionFund::Uno => 0.1087,
    };
    let total_afp_rate = afp_rate + afp_commission;
    let afp_discount = taxable_income * total_afp_rate;

    let health_rate = 0.07;
    let health_discount = match health_system {
        HealthSystem::Fonasa => taxable_income * health_rate,
        HealthSystem::Isapre { .. } => {
            health_fixed_amount.unwrap_or(taxable_income * health_rate)
        }
    };

    let unemployment_discount = taxable_income * 0.006;

    let income_tax = calculate_income_tax(taxable_income);

    let other_deductions = payload.other_deductions;
    let total_deductions =
        afp_discount + health_discount + unemployment_discount + income_tax + other_deductions;
    let net_salary = taxable_income + non_taxable - total_deductions;

    let mut breakdown = vec![
        PayrollLineItem {
            concept: "Sueldo Base".into(),
            amount: salary_base,
            category: "Imponible".into(),
        },
        PayrollLineItem {
            concept: "Gratificacion".into(),
            amount: monthly_gratificacion,
            category: "Imponible".into(),
        },
        PayrollLineItem {
            concept: "Movilizacion/Colacion".into(),
            amount: non_taxable,
            category: "No Imponible".into(),
        },
        PayrollLineItem {
            concept: "AFP (10% + Comision)".into(),
            amount: -afp_discount,
            category: "Descuento Legal".into(),
        },
        PayrollLineItem {
            concept: format!("Salud ({health_system})"),
            amount: -health_discount,
            category: "Descuento Legal".into(),
        },
        PayrollLineItem {
            concept: "Seguro Cesantia".into(),
            amount: -unemployment_discount,
            category: "Descuento Legal".into(),
        },
    ];
    if income_tax > 0.0 {
        breakdown.push(PayrollLineItem {
            concept: "Impuesto 2da Categoria".into(),
            amount: -income_tax,
            category: "Descuento Legal".into(),
        });
    }
    if other_deductions > 0.0 {
        breakdown.push(PayrollLineItem {
            concept: "Otros Descuentos".into(),
            amount: -other_deductions,
            category: "Descuento".into(),
        });
    }

    PayrollCalculation {
        employee_id: payload.employee_id,
        month: payload.month,
        year: payload.year,
        salary_base,
        gratificacion: monthly_gratificacion,
        non_taxable_earnings: non_taxable,
        taxable_income,
        afp_rate: total_afp_rate,
        afp_discount,
        health_rate,
        health_discount,
        unemployment_discount,
        income_tax,
        other_deductions,
        net_salary,
        breakdown,
    }
}

/// Calcula el impuesto a la renta de segunda categoría (global complementario)
/// mensual según las tablas de impuesto progresivo chilenas.
///
/// El cálculo se realiza anualizando la renta imponible mensual, aplicando
/// los tramos y tasas vigentes, y luego dividiendo por 12.
fn calculate_income_tax(monthly_taxable_income: f64) -> f64 {
    let annual_taxable = monthly_taxable_income * 12.0;
    let tax = if annual_taxable <= 937_440.0 {
        0.0
    } else if annual_taxable <= 1_874_880.0 {
        (annual_taxable - 937_440.0) * 0.04
    } else if annual_taxable <= 3_124_800.0 {
        37_497.0 + (annual_taxable - 1_874_880.0) * 0.08
    } else if annual_taxable <= 4_374_720.0 {
        137_497.0 + (annual_taxable - 3_124_800.0) * 0.135
    } else if annual_taxable <= 6_249_600.0 {
        306_247.0 + (annual_taxable - 4_374_720.0) * 0.23
    } else if annual_taxable <= 8_124_480.0 {
        737_247.0 + (annual_taxable - 6_249_600.0) * 0.304
    } else {
        1_307_247.0 + (annual_taxable - 8_124_480.0) * 0.35
    };
    (tax / 12.0).max(0.0)
}

/// Calcula los días progresivos de vacaciones según los años de servicio.
///
/// | Años de servicio | Días de vacaciones |
/// |------------------|--------------------|
/// | 0–10             | 15                 |
/// | 11–15            | 18                 |
/// | 16–20            | 21                 |
/// | 21–25            | 24                 |
/// | 26–30            | 27                 |
/// | 31+              | 30                 |
pub fn calculate_progressive_vacation_days(years_of_service: i32) -> f64 {
    match years_of_service {
        0..=10 => 15.0,
        11..=15 => 18.0,
        16..=20 => 21.0,
        21..=25 => 24.0,
        26..=30 => 27.0,
        _ => 30.0,
    }
}

/// Calcula la diferencia en años completos entre dos fechas.
/// Retorna 0 si la fecha de contratación es posterior a `from_date`.
pub fn years_between(hire_date: NaiveDate, from_date: NaiveDate) -> i32 {
    let mut years = from_date.year() - hire_date.year();
    if from_date.ordinal() < hire_date.ordinal() {
        years -= 1;
    }
    years.max(0)
}
