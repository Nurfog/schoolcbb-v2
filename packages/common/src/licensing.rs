use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Plan de licenciamiento con precios mensual y anual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub price_monthly: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub price_yearly: f64,
    pub featured: bool,
    pub sort_order: i32,
    pub active: bool,
    pub is_custom: bool,
    pub show_in_portal: bool,
    pub created_at: DateTime<Utc>,
}

/// Módulo incluido o excluido en un plan de licencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModule {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub module_key: String,
    pub module_name: String,
    pub included: bool,
    pub sub_modules: Option<Vec<String>>,
}

/// Licencia asignada a una corporación para un plan específico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporationLicense {
    pub id: Uuid,
    pub corporation_id: Uuid,
    pub plan_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub auto_renew: bool,
    pub grace_period_days: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pago asociado a una licencia de corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayment {
    pub id: Uuid,
    pub corporation_license_id: Uuid,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub amount: f64,
    pub currency: String,
    pub payment_method: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Extensión de días adicionales para una licencia (periodo de gracia).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseExtension {
    pub id: Uuid,
    pub corporation_license_id: Uuid,
    pub days_extended: i32,
    pub reason: String,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Resumen del estado de licencia de una corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSummary {
    pub corporation_name: String,
    pub corporation_id: Uuid,
    pub plan_name: String,
    pub plan_id: Uuid,
    pub status: String,
    pub days_remaining: i64,
    pub total_schools: i64,
    pub total_students: i64,
    pub total_employees: i64,
}

/// Payload para crear un nuevo plan de licencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLicensePlanPayload {
    pub name: String,
    pub description: Option<String>,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub price_monthly: f64,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub price_yearly: f64,
    pub featured: bool,
    pub sort_order: i32,
    pub is_custom: bool,
    pub show_in_portal: bool,
    pub modules: Vec<PlanModuleInput>,
}

/// Módulo de entrada al crear un plan de licencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModuleInput {
    pub module_key: String,
    pub module_name: String,
    pub included: bool,
    pub sub_modules: Option<Vec<String>>,
}

/// Payload para asignar una licencia a una corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignLicensePayload {
    pub corporation_id: Uuid,
    pub plan_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub auto_renew: bool,
    pub grace_period_days: Option<i32>,
}

/// Payload para extender una licencia existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendLicensePayload {
    pub days: i32,
    pub reason: String,
}

/// Payload para registrar un pago de licencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPaymentPayload {
    pub corporation_license_id: Uuid,
    #[doc = "Moneda en CLP - usar con precaución: f64 puede causar errores de redondeo"]
    pub amount: f64,
    pub currency: Option<String>,
    pub payment_method: String,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Sobrescritura manual de habilitación de un módulo para una corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporationModuleOverride {
    pub id: Uuid,
    pub corporation_id: Uuid,
    pub module_key: String,
    pub enabled: bool,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input para sobrescribir el estado de un módulo en una corporación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporationModuleOverrideInput {
    pub module_key: String,
    pub enabled: bool,
    pub reason: Option<String>,
}
