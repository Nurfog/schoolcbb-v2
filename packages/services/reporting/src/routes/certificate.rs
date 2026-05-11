use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, State},
    http::request::Parts,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{ReportError, ReportResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub school_id: Option<String>,
    pub corporation_id: Option<String>,
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = ReportError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ReportError::Unauthorized)?;

        let secret = &_state.config.jwt_secret;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| ReportError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), ReportError> {
    if !roles.contains(&claims.role.as_str()) {
        return Err(ReportError::Forbidden(format!(
            "Se requiere uno de los roles {:?}, tiene '{}'",
            roles, claims.role
        )));
    }
    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/reports/certificate/student/{student_id}",
        get(certificate_student),
    )
}

async fn certificate_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> ReportResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let student = sqlx::query_as::<_, StudentRow>(
        r#"
        SELECT s.id, CONCAT(s.first_name, ' ', s.last_name) as student_name, s.rut,
               s.grade_level, s.section, s.enrolled
        FROM students s WHERE s.id = $1
        "#,
    )
    .bind(student_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ReportError::NotFound("Estudiante no encontrado".into()))?;

    if !student.enrolled {
        return Err(ReportError::Validation(
            "El estudiante no se encuentra matriculado actualmente".into(),
        ));
    }

    let enrollment_year: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(year)::int FROM enrollments WHERE student_id = $1 AND active = true",
    )
    .bind(student_id)
    .fetch_one(&state.pool)
    .await?;

    let year = enrollment_year.unwrap_or_else(|| {
        let now = chrono::Utc::now();
        let s = now.format("%Y").to_string();
        s.parse::<i32>().unwrap_or(2025)
    });

    let issuer_name = claims.name.clone();
    let issued_at = chrono::Utc::now().format("%d/%m/%Y %H:%M").to_string();

    let cert = schoolcbb_common::reporting::CertificateRegular {
        student_id: student.id,
        student_name: student.student_name,
        rut: student.rut,
        grade_level: student.grade_level,
        section: student.section,
        year,
        enrollment_status: "Matriculado".to_string(),
        issued_at,
        issuer_name,
    };

    Ok(Json(json!({ "certificate": cert })))
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
struct StudentRow {
    id: Uuid,
    student_name: String,
    rut: String,
    grade_level: String,
    section: String,
    enrolled: bool,
}
