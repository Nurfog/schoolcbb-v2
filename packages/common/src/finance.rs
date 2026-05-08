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

impl StudentFinanceSummary {
    pub fn balance(&self) -> f64 {
        self.total_pending
    }

    pub fn payment_progress(&self) -> f64 {
        if self.total_fees == 0.0 {
            return 100.0;
        }
        (self.total_paid / self.total_fees) * 100.0
    }

    pub fn effective_discount(&self) -> f64 {
        self.discount_percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finance_summary_balance() {
        let s = StudentFinanceSummary {
            student_id: Uuid::nil(),
            total_fees: 500000.0,
            total_paid: 200000.0,
            total_pending: 300000.0,
            discount_percentage: 0.0,
            fees: vec![],
            scholarships: vec![],
        };
        assert!((s.balance() - 300000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_finance_summary_payment_progress() {
        let s = StudentFinanceSummary {
            student_id: Uuid::nil(),
            total_fees: 500000.0,
            total_paid: 200000.0,
            total_pending: 300000.0,
            discount_percentage: 0.0,
            fees: vec![],
            scholarships: vec![],
        };
        assert!((s.payment_progress() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_finance_summary_payment_progress_zero_fees() {
        let s = StudentFinanceSummary {
            student_id: Uuid::nil(),
            total_fees: 0.0,
            total_paid: 0.0,
            total_pending: 0.0,
            discount_percentage: 0.0,
            fees: vec![],
            scholarships: vec![],
        };
        assert!((s.payment_progress() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_finance_summary_full_payment() {
        let s = StudentFinanceSummary {
            student_id: Uuid::nil(),
            total_fees: 300000.0,
            total_paid: 300000.0,
            total_pending: 0.0,
            discount_percentage: 0.0,
            fees: vec![],
            scholarships: vec![],
        };
        assert!((s.balance() - 0.0).abs() < f64::EPSILON);
        assert!((s.payment_progress() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_create_fee_valid_amount() {
        let fee = CreateFeePayload {
            student_id: Uuid::nil(),
            description: "Matrícula 2025".into(),
            amount: 150000.0,
            due_date: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        };
        assert!(fee.amount > 0.0);
        assert!(!fee.description.is_empty());
    }

    #[test]
    fn test_scholarship_discount_range() {
        let s = CreateScholarshipPayload {
            student_id: Uuid::nil(),
            name: "Beca Excelencia".into(),
            discount_percentage: 50.0,
            valid_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            valid_until: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        };
        assert!(s.discount_percentage > 0.0);
        assert!(s.discount_percentage <= 100.0);
    }

    #[test]
    fn test_scholarship_discount_invalid() {
        let s = CreateScholarshipPayload {
            student_id: Uuid::nil(),
            name: "Beca Invalida".into(),
            discount_percentage: 0.0,
            valid_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            valid_until: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        };
        assert!(s.discount_percentage <= 0.0);
    }

    #[test]
    fn test_scholarship_discount_over_100() {
        let s = CreateScholarshipPayload {
            student_id: Uuid::nil(),
            name: "Beca Exceso".into(),
            discount_percentage: 150.0,
            valid_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            valid_until: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        };
        assert!(s.discount_percentage > 100.0);
    }
}
