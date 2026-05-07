use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum FinanceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl IntoResponse for FinanceError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            FinanceError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Error interno del servidor".into())
            }
            FinanceError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            FinanceError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            FinanceError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autorizado".into()),
            FinanceError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

pub type FinanceResult<T> = Result<T, FinanceError>;
