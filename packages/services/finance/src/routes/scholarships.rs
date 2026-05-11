use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::FinanceResult;
use crate::routes::fees::{require_any_role, Claims};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/finance/scholarships", get(list_scholarships).post(create_scholarship))
        .route("/api/finance/scholarships/{id}", get(get_scholarship).put(approve_scholarship).delete(delete_scholarship))
        .route("/api/finance/scholarships/student/{student_id}", get(scholarships_by_student))
}

async fn list_scholarships(
    claims: Claims,
    State(state): State<AppState>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let scholarships = sqlx::query_as::<_, schoolcbb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM scholarships ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "scholarships": scholarships, "total": scholarships.len() })))
}

async fn get_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let scholarship = sqlx::query_as::<_, schoolcbb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM scholarships WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Beca no encontrada".into()))?;

    Ok(Json(json!({ "scholarship": scholarship })))
}

async fn create_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::finance::CreateScholarshipPayload>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    if payload.name.trim().is_empty() || payload.discount_percentage <= 0.0 || payload.discount_percentage > 100.0 {
        return Err(crate::error::FinanceError::Validation("Nombre y porcentaje válido (1-100) son obligatorios".into()));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolcbb_common::finance::Scholarship>(
        r#"
        INSERT INTO scholarships (id, student_id, name, discount_percentage, valid_from, valid_until)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&payload.name)
    .bind(payload.discount_percentage)
    .bind(payload.valid_from)
    .bind(payload.valid_until)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "scholarship": result })))
}

async fn approve_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director"])?;

    let approver_id: Uuid = claims.sub.parse().map_err(|_| crate::error::FinanceError::Unauthorized)?;

    let result = sqlx::query_as::<_, schoolcbb_common::finance::Scholarship>(
        r#"
        UPDATE scholarships SET approved = true, approved_by = $1 WHERE id = $2
        RETURNING id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at
        "#,
    )
    .bind(approver_id)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(crate::error::FinanceError::NotFound("Beca no encontrada".into()))?;

    Ok(Json(json!({ "scholarship": result })))
}

async fn delete_scholarship(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM scholarships WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Beca eliminada correctamente" })))
}

async fn scholarships_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Apoderado"])?;

    let scholarships = sqlx::query_as::<_, schoolcbb_common::finance::Scholarship>(
        "SELECT id, student_id, name, discount_percentage, approved, approved_by, valid_from, valid_until, created_at FROM scholarships WHERE student_id = $1 ORDER BY valid_from DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "scholarships": scholarships })))
}
