use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
    routing::post,
    Json, Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};

use crate::error::{AuthError, AuthResult};
use crate::models::{self, Claims};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", post(me))
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AuthError::Unauthorized)?;

        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "cambio-en-produccion".into());

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid("Token inválido".into()),
        })?;

        Ok(token_data.claims)
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::user::AuthPayload>,
) -> AuthResult<Json<Value>> {
    let user = models::find_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !models::verify_password(&payload.password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }

    if !user.active {
        return Err(AuthError::Unauthorized);
    }

    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::hours(12)).timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        role: user.role.clone(),
        name: user.name.clone(),
        email: user.email.clone(),
        exp,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::Internal(format!("JWT encoding failed: {e}")))?;

    Ok(Json(json!({
        "token": token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role
        }
    })))
}

async fn me(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    let id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID in token".into()))?;

    let user = models::find_by_id(&state.pool, id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut
        }
    })))
}
