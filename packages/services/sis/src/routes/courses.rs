use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::students::{require_any_role, Claims};
use crate::error::{SisError, SisResult};
use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
struct RawCourse {
    id: Uuid,
    name: String,
    subject: String,
    grade_level: String,
    section: String,
    teacher_id: Uuid,
    plan: Option<String>,
    classroom_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct CourseQuery {
    grade_level: Option<String>,
    teacher_id: Option<String>,
    search: Option<String>,
    plan: Option<String>,
}

#[derive(Deserialize)]
struct CreateCoursePayload {
    name: String,
    subject: String,
    grade_level: String,
    section: String,
    teacher_id: Uuid,
    plan: Option<String>,
    classroom_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct UpdateCoursePayload {
    name: Option<String>,
    subject: Option<String>,
    grade_level: Option<String>,
    section: Option<String>,
    teacher_id: Option<Uuid>,
    plan: Option<String>,
    classroom_id: Option<Uuid>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/courses", get(list_courses).post(create_course))
        .route("/api/courses/:id", get(get_course).put(update_course).delete(delete_course))
}

async fn list_courses(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<CourseQuery>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP", "Profesor"])?;

    let (where_clause, _param_idx) = build_filters(&q);
    let sql = format!(
        "SELECT id, name, subject, grade_level, section, teacher_id, plan, classroom_id FROM courses {} ORDER BY grade_level, plan, name",
        where_clause
    );

    let mut query = sqlx::query_as::<_, RawCourse>(&sql);
    if let Some(ref gl) = q.grade_level {
        query = query.bind(gl);
    }
    if let Some(ref p) = q.plan {
        query = query.bind(p);
    }
    if let Some(ref tid) = q.teacher_id {
        let uid = Uuid::parse_str(tid).map_err(|_| SisError::Validation("teacher_id inválido".into()))?;
        query = query.bind(uid);
    }
    if let Some(ref s) = q.search {
        let pat = format!("%{}%", s);
        query = query.bind(pat);
    }

    let courses = query.fetch_all(&state.pool).await?;
    Ok(Json(json!({ "courses": courses })))
}

fn build_filters(q: &CourseQuery) -> (String, u32) {
    let mut clauses = Vec::new();
    let mut idx = 1u32;
    if q.grade_level.is_some() { clauses.push(format!("grade_level = ${}", idx)); idx += 1; }
    if q.plan.is_some() { clauses.push(format!("plan = ${}", idx)); idx += 1; }
    if q.teacher_id.is_some() { clauses.push(format!("teacher_id = ${}", idx)); idx += 1; }
    if q.search.is_some() { clauses.push(format!("(name ILIKE ${} OR subject ILIKE ${})", idx, idx)); idx += 1; }
    let where_clause = if clauses.is_empty() { String::new() } else { format!("WHERE {}", clauses.join(" AND ")) };
    (where_clause, idx)
}

async fn create_course(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateCoursePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP"])?;

    if payload.name.trim().is_empty() || payload.subject.trim().is_empty() {
        return Err(SisError::Validation("Nombre y asignatura son obligatorios".into()));
    }

    let course = sqlx::query_as::<_, RawCourse>(
        "INSERT INTO courses (name, subject, grade_level, section, teacher_id, plan, classroom_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, name, subject, grade_level, section, teacher_id, plan, classroom_id",
    )
    .bind(&payload.name)
    .bind(&payload.subject)
    .bind(&payload.grade_level)
    .bind(&payload.section)
    .bind(payload.teacher_id)
    .bind(&payload.plan)
    .bind(&payload.classroom_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "course": course })))
}

async fn get_course(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP", "Profesor"])?;

    let course = sqlx::query_as::<_, RawCourse>(
        "SELECT id, name, subject, grade_level, section, teacher_id, plan, classroom_id FROM courses WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Curso no encontrado".into()))?;

    Ok(Json(json!({ "course": course })))
}

async fn update_course(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCoursePayload>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador", "Director", "UTP"])?;

    let current = sqlx::query_as::<_, RawCourse>(
        "SELECT id, name, subject, grade_level, section, teacher_id, plan, classroom_id FROM courses WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(SisError::NotFound("Curso no encontrado".into()))?;

    let name = payload.name.unwrap_or(current.name);
    let subject = payload.subject.unwrap_or(current.subject);
    let grade_level = payload.grade_level.unwrap_or(current.grade_level);
    let section = payload.section.unwrap_or(current.section);
    let teacher_id = payload.teacher_id.unwrap_or(current.teacher_id);
    let plan = payload.plan.or(current.plan);
    let classroom_id = payload.classroom_id.or(current.classroom_id);

    let course = sqlx::query_as::<_, RawCourse>(
        "UPDATE courses SET name = $1, subject = $2, grade_level = $3, section = $4, teacher_id = $5, plan = $6, classroom_id = $7 WHERE id = $8
         RETURNING id, name, subject, grade_level, section, teacher_id, plan, classroom_id",
    )
    .bind(&name)
    .bind(&subject)
    .bind(&grade_level)
    .bind(&section)
    .bind(teacher_id)
    .bind(&plan)
    .bind(&classroom_id)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "course": course })))
}

async fn delete_course(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> SisResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;

    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM enrollments WHERE course_id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    if exists > 0 {
        return Err(SisError::Conflict("No se puede eliminar un curso con alumnos matriculados".into()));
    }

    sqlx::query("DELETE FROM course_subjects WHERE course_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM courses WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Curso eliminado correctamente" })))
}
