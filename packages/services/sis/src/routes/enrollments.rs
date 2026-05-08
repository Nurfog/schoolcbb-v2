use axum::{
    extract::{Path, Query, State},
    routing::{get, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::students::{require_any_role, Claims};
use crate::error::{SisError, SisResult};
use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct RawEnrollment {
    id: Uuid,
    student_id: Uuid,
    course_id: Uuid,
    year: i32,
    active: bool,
}

#[derive(Deserialize)]
struct EnrollmentQuery {
    student_id: Option<Uuid>,
    course_id: Option<Uuid>,
    year: Option<i32>,
    active: Option<bool>,
}

#[derive(Deserialize)]
struct CreateEnrollmentPayload {
    student_id: Uuid,
    course_id: Uuid,
    year: i32,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/enrollments", get(list_enrollments).post(create_enrollment))
        .route("/api/enrollments/{id}", delete(delete_enrollment))
}

async fn list_enrollments(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<EnrollmentQuery>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP", "Profesor"])?;

    let mut sql = "SELECT id, student_id, course_id, year, active FROM enrollments WHERE 1=1".to_string();
    if q.student_id.is_some() { sql.push_str(" AND student_id = $1"); }
    if q.course_id.is_some() { sql.push_str(" AND course_id = $2"); }
    if q.year.is_some() { sql.push_str(" AND year = $3"); }
    if q.active.is_some() { sql.push_str(" AND active = $4"); }
    sql.push_str(" ORDER BY year DESC, student_id");

    let mut query = sqlx::query_as::<_, RawEnrollment>(&sql);
    if let Some(sid) = q.student_id { query = query.bind(sid); }
    if let Some(cid) = q.course_id { query = query.bind(cid); }
    if let Some(y) = q.year { query = query.bind(y); }
    if let Some(a) = q.active { query = query.bind(a); }

    let enrollments = query.fetch_all(&state.pool).await?;
    Ok(Json(json!({ "enrollments": enrollments })))
}

async fn create_enrollment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateEnrollmentPayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP"])?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM enrollments WHERE student_id = $1 AND course_id = $2 AND year = $3",
    )
    .bind(payload.student_id)
    .bind(payload.course_id)
    .bind(payload.year)
    .fetch_one(&state.pool)
    .await?;

    if exists > 0 {
        return Err(SisError::Conflict("El alumno ya está matriculado en este curso y año".into()));
    }

    let enrollment = sqlx::query_as::<_, RawEnrollment>(
        "INSERT INTO enrollments (student_id, course_id, year)
         VALUES ($1, $2, $3)
         RETURNING id, student_id, course_id, year, active",
    )
    .bind(payload.student_id)
    .bind(payload.course_id)
    .bind(payload.year)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "enrollment": enrollment })))
}

async fn delete_enrollment(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let result = sqlx::query("DELETE FROM enrollments WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(SisError::NotFound("Matrícula no encontrada".into()));
    }

    Ok(Json(json!({ "message": "Matrícula eliminada correctamente" })))
}
