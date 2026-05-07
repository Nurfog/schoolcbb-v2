use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AcademicError, AcademicResult};
use crate::routes::subjects::{require_any_role, Claims};
use crate::AppState;

#[derive(Deserialize)]
pub struct GradeFilter {
    pub student_id: Option<Uuid>,
    pub course_subject_id: Option<Uuid>,
    pub semester: Option<i32>,
    pub year: Option<i32>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grades", get(list_grades).post(create_grade))
        .route("/api/grades/{id}", get(get_grade).put(update_grade).delete(delete_grade))
        .route("/api/grades/bulk", post(bulk_create_grades))
        .route("/api/grades/course-subject/{course_subject_id}", get(grades_by_course_subject))
        .route("/api/grades/student/{student_id}/{semester}/{year}", get(student_grades))
}

async fn list_grades(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<GradeFilter>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let grades = if let Some(sid) = filter.student_id {
        if let Some(csid) = filter.course_subject_id {
            if let Some(sem) = filter.semester {
                if let Some(y) = filter.year {
                    sqlx::query_as::<_, RawGrade>(
                        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
                         FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 AND year = $4 ORDER BY date DESC"
                    ).bind(sid).bind(csid).bind(sem).bind(y).fetch_all(&state.pool).await?
                } else {
                    sqlx::query_as::<_, RawGrade>(
                        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
                         FROM grades WHERE student_id = $1 AND course_subject_id = $2 AND semester = $3 ORDER BY date DESC"
                    ).bind(sid).bind(csid).bind(sem).fetch_all(&state.pool).await?
                }
            } else {
                sqlx::query_as::<_, RawGrade>(
                    "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
                     FROM grades WHERE student_id = $1 AND course_subject_id = $2 ORDER BY date DESC"
                ).bind(sid).bind(csid).fetch_all(&state.pool).await?
            }
        } else {
            sqlx::query_as::<_, RawGrade>(
                "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
                 FROM grades WHERE student_id = $1 ORDER BY date DESC"
            ).bind(sid).fetch_all(&state.pool).await?
        }
    } else {
        sqlx::query_as::<_, RawGrade>(
            "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
             FROM grades ORDER BY date DESC"
        ).fetch_all(&state.pool).await?
    };

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn get_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let grade = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Calificación no encontrada".into()))?;

    Ok(Json(json!({ "grade": grade })))
}

async fn create_grade(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::academic::CreateGradePayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if !(1.0..=7.0).contains(&payload.grade) {
        return Err(AcademicError::Validation("La nota debe estar entre 1.0 y 7.0".into()));
    }

    let subject_name = resolve_subject_name(&state.pool, payload.course_subject_id).await?;

    let id = Uuid::new_v4();
    let grade_val = (payload.grade * 10.0).round() / 10.0;

    let result = sqlx::query_as::<_, RawGrade>(
        r#"
        INSERT INTO grades (id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, course_subject_id, category_id, observation)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&subject_name)
    .bind(grade_val)
    .bind(&payload.grade_type)
    .bind(payload.semester)
    .bind(payload.year)
    .bind(payload.date)
    .bind(payload.teacher_id)
    .bind(payload.course_subject_id)
    .bind(payload.category_id)
    .bind(&payload.observation)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "grade": result })))
}

async fn update_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolcbb_common::academic::UpdateGradePayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Calificación no encontrada".into()))?;

    let grade_val = match payload.grade {
        Some(g) => {
            if !(1.0..=7.0).contains(&g) {
                return Err(AcademicError::Validation("La nota debe estar entre 1.0 y 7.0".into()));
            }
            (g * 10.0).round() / 10.0
        }
        None => existing.grade,
    };
    let grade_type = payload.grade_type.unwrap_or(existing.grade_type);
    let category_id = payload.category_id.or(existing.category_id);
    let observation = payload.observation.or(existing.observation);

    let result = sqlx::query_as::<_, RawGrade>(
        r#"
        UPDATE grades SET grade = $1, grade_type = $2, category_id = $3, observation = $4
        WHERE id = $5
        RETURNING id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id
        "#,
    )
    .bind(grade_val)
    .bind(&grade_type)
    .bind(category_id)
    .bind(&observation)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "grade": result })))
}

async fn delete_grade(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM grades WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AcademicError::NotFound("Calificación no encontrada".into()));
    }

    Ok(Json(json!({ "message": "Calificación eliminada correctamente" })))
}

async fn grades_by_course_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(course_subject_id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let grades = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE course_subject_id = $1 ORDER BY date DESC",
    )
    .bind(course_subject_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn bulk_create_grades(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::academic::BulkGradeEntry>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    if payload.grades.is_empty() {
        return Err(AcademicError::Validation("Debe incluir al menos una calificación".into()));
    }

    let subject_name = resolve_subject_name(&state.pool, payload.course_subject_id).await?;

    let mut imported = 0;
    let mut errors: Vec<Value> = vec![];

    for entry in &payload.grades {
        if !(1.0..=7.0).contains(&entry.grade) {
            errors.push(json!({
                "student_id": entry.student_id,
                "error": "Nota fuera de rango (1.0 - 7.0)"
            }));
            continue;
        }

        let grade_val = (entry.grade * 10.0).round() / 10.0;
        let id = Uuid::new_v4();

        let result = sqlx::query(
            r#"
            INSERT INTO grades (id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, course_subject_id, category_id, observation)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(id)
        .bind(entry.student_id)
        .bind(&subject_name)
        .bind(grade_val)
        .bind(&payload.grade_type)
        .bind(payload.semester)
        .bind(payload.year)
        .bind(payload.date)
        .bind(payload.teacher_id)
        .bind(payload.course_subject_id)
        .bind(payload.category_id)
        .bind(&entry.observation)
        .execute(&state.pool)
        .await;

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                errors.push(json!({
                    "student_id": entry.student_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(Json(json!({
        "imported": imported,
        "errors": errors,
        "total": payload.grades.len()
    })))
}

async fn student_grades(
    claims: Claims,
    State(state): State<AppState>,
    Path((student_id, semester, year)): Path<(Uuid, i32, i32)>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor", "Apoderado", "Alumno"])?;

    let grades = sqlx::query_as::<_, RawGrade>(
        "SELECT id, student_id, subject, grade, grade_type, semester, year, date, teacher_id, observation, category_id FROM grades WHERE student_id = $1 AND semester = $2 AND year = $3 ORDER BY subject, date",
    )
    .bind(student_id)
    .bind(semester)
    .bind(year)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "grades": grades, "total": grades.len() })))
}

async fn resolve_subject_name(pool: &sqlx::PgPool, course_subject_id: Uuid) -> Result<String, AcademicError> {
    let row: (String,) = sqlx::query_as(
        r#"
        SELECT s.name FROM subjects s
        JOIN course_subjects cs ON cs.subject_id = s.id
        WHERE cs.id = $1
        "#,
    )
    .bind(course_subject_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AcademicError::Validation("La asignatura del curso no existe".into()))?;

    Ok(row.0)
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct RawGrade {
    pub id: Uuid,
    pub student_id: Uuid,
    pub subject: String,
    pub grade: f64,
    pub grade_type: String,
    pub semester: i32,
    pub year: i32,
    pub date: chrono::NaiveDate,
    pub teacher_id: Uuid,
    pub observation: Option<String>,
    pub category_id: Option<Uuid>,
}
