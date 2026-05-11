use axum::{
    extract::{FromRequestParts, Path, State},
    http::request::Parts,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{FinanceError, FinanceResult};
use crate::AppState;

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
    type Rejection = FinanceError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(FinanceError::Unauthorized)?;

        let secret = &_state.config.jwt_secret;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| FinanceError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), FinanceError> {
    if !roles.contains(&claims.role.as_str()) {
        return Err(FinanceError::Forbidden(format!(
            "Se requiere uno de los roles {:?}, tiene '{}'",
            roles, claims.role
        )));
    }
    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/finance/fees", get(list_fees).post(create_fee))
        .route("/api/finance/fees/{id}", get(get_fee).put(update_fee).delete(delete_fee))
        .route("/api/finance/fees/student/{student_id}", get(fees_by_student))
}

async fn list_fees(
    claims: Claims,
    State(state): State<AppState>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let school_condition = claims.school_id.as_ref().map(|sid| format!(" WHERE school_id = '{}'::uuid", sid)).unwrap_or_default();

    let fees = sqlx::query_as::<_, schoolcbb_common::finance::Fee>(
        &format!("SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees{} ORDER BY due_date DESC LIMIT 100", school_condition),
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "fees": fees, "total": fees.len() })))
}

async fn get_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let fee = sqlx::query_as::<_, schoolcbb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(FinanceError::NotFound("Cuota no encontrada".into()))?;

    Ok(Json(json!({ "fee": fee })))
}

async fn create_fee(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::finance::CreateFeePayload>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    if payload.description.trim().is_empty() || payload.amount <= 0.0 {
        return Err(FinanceError::Validation("Descripción y monto válido son obligatorios".into()));
    }

    let school_id = claims.school_id.and_then(|s| Uuid::parse_str(&s).ok());

    let id = Uuid::new_v4();
    let result = sqlx::query_as::<_, schoolcbb_common::finance::Fee>(
        r#"
        INSERT INTO fees (id, student_id, description, amount, due_date, school_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.due_date)
    .bind(school_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "fee": result })))
}

async fn update_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP"])?;

    let paid = payload.get("paid").and_then(|v| v.as_bool());
    let paid_amount = payload.get("paid_amount").and_then(|v| v.as_f64());

    if let Some(true) = paid {
        let paid_date = chrono::Utc::now().date_naive();
        sqlx::query(
            "UPDATE fees SET paid = true, paid_date = $1, paid_amount = COALESCE($2, amount) WHERE id = $3",
        )
        .bind(paid_date)
        .bind(paid_amount)
        .bind(id)
        .execute(&state.pool)
        .await?;
    }

    let fee = sqlx::query_as::<_, schoolcbb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(FinanceError::NotFound("Cuota no encontrada".into()))?;

    Ok(Json(json!({ "fee": fee })))
}

async fn delete_fee(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;

    sqlx::query("DELETE FROM fees WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "message": "Cuota eliminada correctamente" })))
}

async fn fees_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> FinanceResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor", "Director", "UTP", "Apoderado"])?;

    let fees = sqlx::query_as::<_, schoolcbb_common::finance::Fee>(
        "SELECT id, student_id, description, amount, due_date, paid, paid_date, paid_amount, created_at FROM fees WHERE student_id = $1 ORDER BY due_date",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "fees": fees })))
}
