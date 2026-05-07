use axum::{
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", post(me))
}

async fn login(
    Json(payload): Json<schoolcbb_core::user::AuthPayload>,
) -> AppResult<Json<Value>> {
    tracing::info!("Login attempt for: {}", payload.email);

    Ok(Json(json!({
        "token": "placeholder-jwt-token",
        "user": {
            "id": "00000000-0000-0000-0000-000000000001",
            "name": "Admin",
            "email": payload.email,
            "role": "Administrador"
        }
    })))
}

async fn me() -> Json<Value> {
    Json(json!({"user": null}))
}
