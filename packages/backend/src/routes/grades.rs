use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;
use uuid::Uuid;

use crate::db::models;
use crate::error::AppResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grades/student/{student_id}/{semester}/{year}", get(student_grades))
}

async fn student_grades(
    State(state): State<AppState>,
    axum::extract::Path((student_id, semester, year)): axum::extract::Path<(Uuid, i32, i32)>,
) -> AppResult<Json<Value>> {
    let report = models::get_student_grades(&state.pool, student_id, semester, year).await?;
    Ok(Json(serde_json::to_value(report).unwrap()))
}
