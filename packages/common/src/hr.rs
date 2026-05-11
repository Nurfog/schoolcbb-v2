use chrono::{DateTime, NaiveDate, Utc};
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
