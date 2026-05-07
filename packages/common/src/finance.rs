use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Fee {
    pub id: Uuid,
    pub student_id: Uuid,
    pub description: String,
    pub amount: f64,
    pub due_date: NaiveDate,
    pub paid: bool,
    pub paid_date: Option<NaiveDate>,
    pub paid_amount: Option<f64>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFeePayload {
    pub student_id: Uuid,
    pub description: String,
    pub amount: f64,
    pub due_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Payment {
    pub id: Uuid,
    pub fee_id: Uuid,
    pub student_id: Uuid,
    pub amount: f64,
    pub payment_date: NaiveDate,
    pub payment_method: String,
    pub reference: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentPayload {
    pub fee_id: Uuid,
    pub student_id: Uuid,
    pub amount: f64,
    pub payment_date: Option<NaiveDate>,
    pub payment_method: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Scholarship {
    pub id: Uuid,
    pub student_id: Uuid,
    pub name: String,
    pub discount_percentage: f64,
    pub approved: bool,
    pub approved_by: Option<Uuid>,
    pub valid_from: NaiveDate,
    pub valid_until: NaiveDate,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScholarshipPayload {
    pub student_id: Uuid,
    pub name: String,
    pub discount_percentage: f64,
    pub valid_from: NaiveDate,
    pub valid_until: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentFinanceSummary {
    pub student_id: Uuid,
    pub total_fees: f64,
    pub total_paid: f64,
    pub total_pending: f64,
    pub discount_percentage: f64,
    pub fees: Vec<Fee>,
    pub scholarships: Vec<Scholarship>,
}
