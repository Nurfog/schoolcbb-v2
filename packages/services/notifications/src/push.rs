use axum::{extract::State, Json, Router, routing::post};
use serde_json::{json, Value};

use crate::error::NotifResult;
use crate::ws::routes::Claims;

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/api/communications/push/subscribe", post(subscribe_push))
        .route("/api/communications/push/unsubscribe", post(unsubscribe_push))
}

async fn subscribe_push(
    _claims: Claims,
    State(_state): State<crate::AppState>,
    _body: Json<Value>,
) -> NotifResult<Json<Value>> {
    Ok(Json(json!({ "message": "Suscripcion push registrada" })))
}

async fn unsubscribe_push(
    _claims: Claims,
    State(_state): State<crate::AppState>,
    _body: Json<Value>,
) -> NotifResult<Json<Value>> {
    Ok(Json(json!({ "message": "Suscripcion push eliminada" })))
}
