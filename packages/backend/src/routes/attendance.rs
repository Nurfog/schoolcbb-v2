use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;

use crate::db::models;
use crate::error::AppResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/attendance/today", get(today))
        .route("/api/attendance/monthly/{year}/{month}", get(monthly))
        .route("/api/attendance/alerts", get(alerts))
}

async fn today(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let today = chrono::Utc::now().date_naive().to_string();
    let records = models::get_attendance_today(&state.pool, &today).await?;
    Ok(Json(serde_json::to_value(records).unwrap()))
}

async fn monthly(
    State(state): State<AppState>,
    axum::extract::Path((year, month)): axum::extract::Path<(i32, u32)>,
) -> AppResult<Json<Value>> {
    let data = models::get_monthly_summary(&state.pool, year, month).await?;
    Ok(Json(serde_json::to_value(data).unwrap()))
}

async fn alerts(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let alerts = models::get_attendance_alerts(&state.pool).await?;
    Ok(Json(serde_json::to_value(alerts).unwrap()))
}
