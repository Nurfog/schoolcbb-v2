use axum::{Json, Router, extract::State, routing::get};
use serde_json::Value;
use uuid::Uuid;

use crate::AppState;
use crate::error::{SisError, SisResult};
use crate::routes::students::Claims;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/dashboard/summary", get(summary))
        .route("/api/dashboard/attendance-today", get(attendance_today))
        .route("/api/dashboard/student-alerts", get(student_alerts))
        .route("/api/dashboard/agenda", get(agenda))
}

fn school_and_corp_id(claims: &Claims) -> (Option<Uuid>, Option<Uuid>) {
    let school_id = claims.school_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let corporation_id = claims.corporation_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    (school_id, corporation_id)
}

async fn summary(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let data = crate::routes::models::get_dashboard_summary(&state.pool, school_id, corporation_id).await?;
    Ok(Json(serde_json::to_value(data).map_err(|e| SisError::Internal(e.to_string()))?))
}

async fn attendance_today(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let today = chrono::Utc::now().date_naive().to_string();
    let records = crate::routes::models::get_attendance_today(&state.pool, &today, school_id, corporation_id).await?;

    let total = records.len() as i64;
    let present = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Presente)
        .count() as i64;
    let absent = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Ausente)
        .count() as i64;
    let late = records
        .iter()
        .filter(|r| r.status == schoolccb_common::attendance::AttendanceStatus::Atraso)
        .count() as i64;
    let justified = records.iter().filter(|r| r.status.es_justificado()).count() as i64;

    Ok(Json(serde_json::json!({
        "date": today,
        "total_students": total,
        "present": present,
        "absent": absent,
        "late": late,
        "justified": justified,
        "attendance_percentage": if total > 0 {
            ((present + justified) as f64 / total as f64) * 100.0
        } else {
            100.0
        }
    })))
}

async fn student_alerts(claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    schoolccb_common::roles::require_licensed_module(
        &state.pool,
        claims.corporation_id.as_deref(),
        "dashboard",
    )
    .await
    .map_err(|e| SisError::Forbidden(e))?;
    let (school_id, corporation_id) = school_and_corp_id(&claims);
    let alerts = crate::routes::models::get_attendance_alerts(&state.pool, school_id, corporation_id).await?;
    Ok(Json(serde_json::json!({ "alerts": alerts })))
}

async fn agenda(_claims: Claims, State(state): State<AppState>) -> SisResult<Json<Value>> {
    let today = chrono::Utc::now().date_naive().to_string();
    let events = crate::routes::models::get_agenda_events(&state.pool, &today).await?;
    Ok(Json(serde_json::json!({ "events": events })))
}
