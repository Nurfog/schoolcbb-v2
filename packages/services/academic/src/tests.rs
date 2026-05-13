use axum::response::IntoResponse;

use crate::error::{AcademicError, AcademicResult};

fn validate_grade(value: f64) -> AcademicResult<f64> {
    if value < 1.0 || value > 7.0 {
        return Err(AcademicError::Validation(format!(
            "Nota {} fuera de rango (1.0 - 7.0)",
            value
        )));
    }
    if (value * 10.0).fract() != 0.0 {
        return Err(AcademicError::Validation(
            "Nota debe tener máximo 1 decimal".into(),
        ));
    }
    Ok(value)
}

#[test]
fn test_validate_grade_accepts_valid() {
    assert!(validate_grade(5.5).is_ok());
    assert!(validate_grade(7.0).is_ok());
    assert!(validate_grade(1.0).is_ok());
}

#[test]
fn test_validate_grade_rejects_out_of_range() {
    let result = validate_grade(0.5);
    assert!(result.is_err());
    match result {
        Err(AcademicError::Validation(msg)) => assert!(msg.contains("fuera de rango")),
        _ => panic!("expected Validation error"),
    }
    assert!(validate_grade(7.5).is_err());
}

#[test]
fn test_validate_grade_rejects_too_many_decimals() {
    let result = validate_grade(5.55);
    assert!(result.is_err());
    match result {
        Err(AcademicError::Validation(msg)) => assert!(msg.contains("1 decimal")),
        _ => panic!("expected Validation error"),
    }
}

#[test]
fn test_academic_error_into_response_validation() {
    let err = AcademicError::Validation("campo requerido".into());
    let resp = err.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
}
