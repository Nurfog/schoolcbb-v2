use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query, State},
    http::request::Parts,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AcademicError, AcademicResult};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AcademicError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AcademicError::Unauthorized)?;

        let secret = &_state.config.jwt_secret;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| AcademicError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), AcademicError> {
    if !roles.contains(&claims.role.as_str()) {
        return Err(AcademicError::Forbidden(format!(
            "Se requiere uno de los roles {:?}, tiene '{}'",
            roles, claims.role
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct SubjectFilter {
    pub search: Option<String>,
    pub level: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grades/subjects", get(list_subjects).post(create_subject))
        .route("/api/grades/subjects/{id}", get(get_subject).put(update_subject).delete(deactivate_subject))
}

async fn list_subjects(
    claims: Claims,
    State(state): State<AppState>,
    Query(filter): Query<SubjectFilter>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let search_pattern = filter.search.as_ref().map(|q| format!("%{}%", q));

    let subjects = if let (Some(ref pat), Some(ref lvl)) = (&search_pattern, &filter.level) {
        sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE (name ILIKE $1 OR code ILIKE $1) AND level = $2 ORDER BY name",
        )
        .bind(pat)
        .bind(lvl)
        .fetch_all(&state.pool)
        .await?
    } else if let Some(ref pat) = search_pattern {
        sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE name ILIKE $1 OR code ILIKE $1 ORDER BY name",
        )
        .bind(pat)
        .fetch_all(&state.pool)
        .await?
    } else if let Some(ref lvl) = filter.level {
        sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE level = $1 ORDER BY name",
        )
        .bind(lvl)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
            "SELECT id, code, name, level, hours_per_week, active FROM subjects ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(json!({ "subjects": subjects, "total": subjects.len() })))
}

async fn get_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"])?;

    let subject = sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
        "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Asignatura no encontrada".into()))?;

    Ok(Json(json!({ "subject": subject })))
}

async fn create_subject(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::academic::CreateSubjectPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    if payload.code.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AcademicError::Validation("Código y nombre son obligatorios".into()));
    }

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
        r#"
        INSERT INTO subjects (id, code, name, level, hours_per_week)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, code, name, level, hours_per_week, active
        "#,
    )
    .bind(id)
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(&payload.level)
    .bind(payload.hours_per_week.unwrap_or(0))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("subjects_code_key") {
                return AcademicError::Conflict("El código de asignatura ya existe".into());
            }
        }
        AcademicError::Database(e)
    })?;

    Ok(Json(json!({ "subject": result })))
}

async fn update_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolcbb_common::academic::UpdateSubjectPayload>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let existing = sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
        "SELECT id, code, name, level, hours_per_week, active FROM subjects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AcademicError::NotFound("Asignatura no encontrada".into()))?;

    let code = payload.code.unwrap_or(existing.code);
    let name = payload.name.unwrap_or(existing.name);
    let level = payload.level.or(existing.level);
    let hours_per_week = payload.hours_per_week.unwrap_or(existing.hours_per_week);

    let result = sqlx::query_as::<_, schoolcbb_common::academic::Subject>(
        r#"
        UPDATE subjects SET code = $1, name = $2, level = $3, hours_per_week = $4
        WHERE id = $5
        RETURNING id, code, name, level, hours_per_week, active
        "#,
    )
    .bind(&code)
    .bind(&name)
    .bind(&level)
    .bind(hours_per_week)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "subject": result })))
}

async fn deactivate_subject(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AcademicResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    let result = sqlx::query("UPDATE subjects SET active = false WHERE id = $1 AND active = true")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AcademicError::NotFound("Asignatura no encontrada o ya desactivada".into()));
    }

    Ok(Json(json!({ "message": "Asignatura desactivada correctamente" })))
}
