use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub position: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub category: Option<String>,
    pub vacation_days_available: f64,
    pub active: bool,
    pub supervisor_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmployeePayload {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub category: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub vacation_days_available: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeeContract {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub contract_type: String,
    pub salary_base: f64,
    pub weekly_hours: i32,
    pub ley_karin_signed: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContractPayload {
    pub employee_id: Uuid,
    pub contract_type: String,
    pub salary_base: f64,
    pub weekly_hours: i32,
    pub ley_karin_signed: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Entrada,
    SalidaColacion,
    RetornoColacion,
    Salida,
}

impl EntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Entrada => "Entrada",
            EntryType::SalidaColacion => "Salida Colacion",
            EntryType::RetornoColacion => "Retorno Colacion",
            EntryType::Salida => "Salida",
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceError {
    ExcesoJornadaDiaria { max_hours: i32, actual_hours: f64 },
    ExcesoSemanal { max_hours: i32, actual_hours: f64 },
    DescansoInsuficiente { min_hours: i32, actual_hours: f64 },
    Inconsistencia { detail: String },
}

#[derive(Debug, Clone)]
pub struct AttendanceValidator;

impl AttendanceValidator {
    pub fn validate_compliance(
        &self,
        logs: &[AttendanceLog],
        max_daily_hours: i32,
        _max_weekly_hours: i32,
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

            let entries: Vec<&str> = day_logs.iter().map(|l| l.entry_type.as_str()).collect();
            let has_entry = entries.contains(&"Entrada");
            let has_exit = entries.contains(&"Salida");
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

        if logs.len() >= 2 {
            for i in 1..logs.len() {
                let diff = (logs[i].timestamp - logs[i - 1].timestamp).num_minutes() as f64 / 60.0;
                if diff < min_rest_hours as f64 && diff > 0.0 {
                    errors.push(ComplianceError::DescansoInsuficiente {
                        min_hours: min_rest_hours,
                        actual_hours: diff,
                    });
                }
            }
        }

        errors
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceLogPayload {
    pub employee_id: Uuid,
    pub timestamp: NaiveDateTime,
    pub entry_type: String,
    pub device_id: Option<String>,
    pub location_hash: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceModificationPayload {
    pub attendance_id: Uuid,
    pub new_timestamp: NaiveDateTime,
    pub new_entry_type: String,
    pub reason: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeavePayload {
    pub employee_id: Uuid,
    pub leave_type: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveApprovalPayload {
    pub status: String,
    pub approved_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Complaint {
    pub id: Uuid,
    pub complainant_name: Option<String>,
    pub complainant_email: Option<String>,
    pub accused_rut: Option<String>,
    pub complaint_type: String,
    pub description: String,
    pub status: String,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComplaintPayload {
    pub complainant_name: Option<String>,
    pub complainant_email: Option<String>,
    pub accused_rut: Option<String>,
    pub complaint_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveComplaintPayload {
    pub status: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl std::fmt::Display for PensionFund {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSystem {
    Fonasa,
    Isapre {
        plan_name: String,
        fixed_amount: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct EmployeePensionFund {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub pension_fund: String,
    pub health_system: String,
    pub health_plan_name: Option<String>,
    pub health_fixed_amount: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Payroll {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    pub salary_base: f64,
    pub gratificacion: f64,
    pub non_taxable_earnings: f64,
    pub taxable_income: f64,
    pub afp_discount: f64,
    pub health_discount: f64,
    pub unemployment_discount: f64,
    pub income_tax: f64,
    pub other_deductions: f64,
    pub net_salary: f64,
    pub lre_exported: bool,
    pub previred_exported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollLineItem {
    pub concept: String,
    pub amount: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollCalculation {
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    pub salary_base: f64,
    pub gratificacion: f64,
    pub non_taxable_earnings: f64,
    pub taxable_income: f64,
    pub afp_rate: f64,
    pub afp_discount: f64,
    pub health_rate: f64,
    pub health_discount: f64,
    pub unemployment_discount: f64,
    pub income_tax: f64,
    pub other_deductions: f64,
    pub net_salary: f64,
    pub breakdown: Vec<PayrollLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollPayload {
    pub employee_id: Uuid,
    pub month: i32,
    pub year: i32,
    pub non_taxable_earnings: f64,
    pub other_deductions: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviredRecord {
    pub rut: String,
    pub name: String,
    pub gross_salary: f64,
    pub afp_discount: f64,
    pub health_discount: f64,
    pub unemployment_discount: f64,
    pub net_salary: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofencePayload {
    pub employee_id: Uuid,
    pub lat: f64,
    pub lng: f64,
    pub radius_meters: f64,
    pub name: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMedicalLicensePayload {
    pub employee_id: Uuid,
    pub license_type: String,
    pub folio: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub diagnosis: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvaluationPayload {
    pub employee_id: Uuid,
    pub evaluation_type: String,
    pub score: Option<f64>,
    pub observations: Option<String>,
    pub period: Option<String>,
    pub year: i32,
}

#[derive(Debug, Clone)]
pub struct PayrollCalculator {
    pub base_salary: f64,
    pub gratificacion: f64,
    pub afp_rate: f64,
    pub isapre_fixed_amount: Option<f64>,
}

impl PayrollCalculator {
    pub fn new(base_salary: f64) -> Self {
        Self {
            base_salary,
            gratificacion: (base_salary * 0.25).min(500000.0),
            afp_rate: 0.10,
            isapre_fixed_amount: None,
        }
    }

    pub fn with_gratificacion(mut self, grat: f64) -> Self {
        self.gratificacion = grat;
        self
    }

    pub fn with_afp_rate(mut self, rate: f64) -> Self {
        self.afp_rate = rate;
        self
    }

    pub fn with_isapre(mut self, amount: f64) -> Self {
        self.isapre_fixed_amount = Some(amount);
        self
    }

    pub fn calculate_liquid(&self) -> f64 {
        let taxable = self.base_salary + self.gratificacion;
        let afp_discount = taxable * self.afp_rate;
        let health = self.isapre_fixed_amount.unwrap_or(taxable * 0.07);
        let unemployment = taxable * 0.006;
        taxable - afp_discount - health - unemployment
    }
}

pub fn calculate_payroll(
    _employee: &Employee,
    contract: &EmployeeContract,
    payload: &PayrollPayload,
    pension_fund: &str,
    health_system: &str,
    health_fixed_amount: Option<f64>,
) -> PayrollCalculation {
    let salary_base = contract.salary_base;
    let monthly_gratificacion = (salary_base * 0.25 * payload.month as f64 / 12.0).min(500000.0);
    let non_taxable = payload.non_taxable_earnings;
    let taxable_income = salary_base + monthly_gratificacion;

    let afp_rate = 0.10;
    let afp_commission = match pension_fund {
        "Capital" | "Cuprum" => 0.1144,
        "Habitat" => 0.1127,
        "Planvital" => 0.1102,
        "Provida" => 0.1145,
        "Modelo" => 0.1058,
        "Uno" => 0.1087,
        _ => 0.10,
    };
    let total_afp_rate = afp_rate + afp_commission;
    let afp_discount = taxable_income * total_afp_rate;

    let health_rate = if health_system == "Fonasa" {
        0.07
    } else {
        0.07
    };
    let health_discount = if health_system == "Fonasa" || health_fixed_amount.is_none() {
        taxable_income * health_rate
    } else {
        health_fixed_amount.unwrap_or(0.0)
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
            concept: format!("Salud ({})", health_system),
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

fn calculate_income_tax(monthly_taxable_income: f64) -> f64 {
    let annual_taxable = monthly_taxable_income * 12.0;
    let tax = if annual_taxable <= 0.0 {
        0.0
    } else if annual_taxable <= 937_440.0 {
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

pub fn years_between(hire_date: NaiveDate, from_date: NaiveDate) -> i32 {
    let mut years = from_date.year() - hire_date.year();
    if from_date.ordinal() < hire_date.ordinal() {
        years -= 1;
    }
    years.max(0)
}
