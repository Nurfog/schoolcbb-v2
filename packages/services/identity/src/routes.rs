use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query, State},
    http::request::Parts,
    routing::{get, post, put},
    Json, Router,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::{AuthError, AuthResult};
use crate::models::{self, Claims};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/forgot-password", post(forgot_password))
        .route("/api/auth/reset-password", post(reset_password))
        .route("/api/auth/me", get(me))
        .route("/api/auth/register", post(register))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/revoke-all", post(revoke_all))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/users", get(list_users))
        .route("/api/auth/users/:id/role", put(update_role))
        .route("/api/auth/users/:id/toggle", post(toggle_active))
        .route("/api/user/modules", get(list_modules))
        .route("/api/user/favorites/:module_id", post(toggle_favorite))
        .route("/api/user/profile", put(update_profile))
        .route("/api/user/password", put(change_password))
        .route("/api/user/preferences", get(get_user_preferences))
        .route("/api/user/preferences", put(update_user_preferences))
        .route("/api/config/branding", get(get_branding))
        .route("/api/config/branding", put(update_branding))
}

fn require_role(claims: &Claims, required: &str) -> Result<(), AuthError> {
    if claims.role != required {
        return Err(AuthError::Forbidden(format!(
            "Se requiere rol '{}', tiene '{}'",
            required, claims.role
        )));
    }
    Ok(())
}

fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), AuthError> {
    if !roles.contains(&claims.role.as_str()) {
        return Err(AuthError::Forbidden(format!(
            "Se requiere uno de los roles {:?}, tiene '{}'",
            roles, claims.role
        )));
    }
    Ok(())
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

fn generate_token_pair(
    config: &crate::config::Config,
    user_id: Uuid,
    role: &str,
    name: &str,
    email: &str,
) -> Result<(String, Claims), AuthError> {
    let now = chrono::Utc::now();

    let access_claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        name: name.to_string(),
        email: email.to_string(),
        exp: (now + chrono::Duration::hours(12)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::Internal(format!("JWT encoding failed: {e}")))?;

    Ok((token, access_claims))
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

    let id = user.id;
    let (token, _claims) = generate_token_pair(
        &state.config,
        id,
        &user.role,
        &user.name,
        &user.email,
    )?;

    let (refresh_token, _) = models::create_refresh_token(&state.pool, id, 7).await?;

    Ok(Json(json!({
        "token": token,
        "refresh_token": refresh_token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut
        }
    })))
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    if email.is_empty() {
        return Err(AuthError::Internal("Email es obligatorio".into()));
    }
    let user = models::find_by_email(&state.pool, email)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    use rand::Rng;
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(48)
        .map(char::from)
        .collect();
    let token_hash = models::hash_token(&token);

    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    tracing::info!("Password reset token for {}: {}", email, token);

    Ok(Json(json!({
        "message": "Si el email está registrado, recibirás un enlace de recuperación",
        "reset_token": token,
    })))
}

async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let token = payload.get("token").and_then(|v| v.as_str()).unwrap_or("");
    let new_password = payload.get("new_password").and_then(|v| v.as_str()).unwrap_or("");
    if token.is_empty() || new_password.len() < 6 {
        return Err(AuthError::Internal("Token inválido o contraseña muy corta (mín. 6 caracteres)".into()));
    }

    let token_hash = models::hash_token(token);
    let reset = sqlx::query_as::<_, (Uuid, Uuid, bool)>(
        "SELECT id, user_id, used FROM password_reset_tokens
         WHERE token_hash = $1 AND expires_at > NOW() AND used = false",
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::Internal("Token inválido o expirado".into()))?;

    sqlx::query("UPDATE password_reset_tokens SET used = true WHERE id = $1")
        .bind(reset.0)
        .execute(&state.pool)
        .await?;

    models::change_password(&state.pool, reset.1, new_password).await?;

    Ok(Json(json!({ "message": "Contraseña actualizada correctamente" })))
}

async fn me(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
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

async fn register(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::user::RegisterPayload>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    if payload.role == "Administrador" || payload.role == "Sostenedor" {
        require_role(&claims, "Sostenedor")?;
    }

    if payload.rut.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AuthError::Internal("RUT y nombre son obligatorios".into()));
    }

    if !["Sostenedor", "Director", "UTP", "Administrador", "Profesor", "Apoderado", "Alumno"]
        .contains(&payload.role.as_str())
    {
        return Err(AuthError::Internal(format!("Rol inválido: {}", payload.role)));
    }

    let user = models::insert_user(
        &state.pool,
        &payload.rut,
        &payload.name,
        &payload.email,
        &payload.password,
        &payload.role,
    )
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_email_key") {
                return AuthError::Internal("El email ya está registrado".into());
            }
            if db_err.constraint() == Some("users_rut_key") {
                return AuthError::Internal("El RUT ya está registrado".into());
            }
        }
        AuthError::Database(e)
    })?;

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

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::user::RefreshPayload>,
) -> AuthResult<Json<Value>> {
    let stored = models::find_refresh_token(&state.pool, &payload.refresh_token)
        .await?
        .ok_or(AuthError::TokenInvalid("Refresh token inválido o expirado".into()))?;

    models::revoke_refresh_token(&state.pool, stored.id).await?;

    let user = models::find_by_id(&state.pool, stored.user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    if !user.active {
        return Err(AuthError::Unauthorized);
    }

    let (token, _claims) = generate_token_pair(
        &state.config,
        user.id,
        &user.role,
        &user.name,
        &user.email,
    )?;

    let (new_refresh_token, _) = models::create_refresh_token(&state.pool, user.id, 7).await?;

    Ok(Json(json!({
        "token": token,
        "refresh_token": new_refresh_token,
        "user": {
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "role": user.role,
            "rut": user.rut
        }
    })))
}

async fn revoke_all(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_role(&claims, "Administrador")?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    models::revoke_all_user_tokens(&state.pool, user_id).await?;
    Ok(Json(json!({ "message": "All sessions revoked" })))
}

async fn logout(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    models::revoke_all_user_tokens(&state.pool, user_id).await?;
    Ok(Json(json!({ "message": "Sesión cerrada correctamente" })))
}

#[derive(Serialize, sqlx::FromRow)]
struct UserListItem {
    id: Uuid,
    rut: String,
    name: String,
    email: String,
    role: String,
    active: bool,
}

#[derive(Deserialize)]
struct UserQuery {
    search: Option<String>,
}

async fn list_users(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<UserQuery>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let search = q.search.as_deref().unwrap_or("");
    let users = if search.is_empty() {
        sqlx::query_as::<_, UserListItem>(
            "SELECT id, rut, name, email, role, active FROM users ORDER BY name",
        )
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, UserListItem>(
            "SELECT id, rut, name, email, role, active FROM users
             WHERE name ILIKE $1 OR email ILIKE $1 ORDER BY name",
        )
        .bind(format!("%{}%", search))
        .fetch_all(&state.pool)
        .await?
    };
    Ok(Json(json!({ "users": users })))
}

async fn update_role(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let new_role = payload
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::Internal("role es requerido".into()))?;
    if !["Sostenedor", "Director", "UTP", "Administrador", "Profesor", "Apoderado", "Alumno"]
        .contains(&new_role)
    {
        return Err(AuthError::Internal("Rol inválido".into()));
    }
    let user = sqlx::query_as::<_, UserListItem>(
        "UPDATE users SET role = $1 WHERE id = $2 RETURNING id, rut, name, email, role, active",
    )
    .bind(new_role)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::UserNotFound)?;
    Ok(Json(json!({ "user": user })))
}

async fn toggle_active(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Sostenedor"])?;
    let user = sqlx::query_as::<_, UserListItem>(
        "UPDATE users SET active = NOT active WHERE id = $1 RETURNING id, rut, name, email, role, active",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::UserNotFound)?;
    Ok(Json(json!({ "user": user })))
}

async fn update_profile(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    if name.trim().is_empty() || email.trim().is_empty() {
        return Err(AuthError::Internal("Nombre y email son obligatorios".into()));
    }
    let user = models::update_user_profile(&state.pool, id, name, email)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("users_email_key") {
                    return AuthError::Internal("El email ya está registrado".into());
                }
            }
            AuthError::Database(e)
        })?;
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

async fn change_password(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let current_password = payload.get("current_password").and_then(|v| v.as_str()).unwrap_or("");
    let new_password = payload.get("new_password").and_then(|v| v.as_str()).unwrap_or("");
    if new_password.len() < 6 {
        return Err(AuthError::Internal("La nueva contraseña debe tener al menos 6 caracteres".into()));
    }
    let user = models::find_by_id(&state.pool, id)
        .await?
        .ok_or(AuthError::UserNotFound)?;
    if !models::verify_password(current_password, &user.password_hash) {
        return Err(AuthError::Internal("La contraseña actual no es correcta".into()));
    }
    models::change_password(&state.pool, id, new_password).await?;
    Ok(Json(json!({ "message": "Contraseña actualizada correctamente" })))
}

async fn get_user_preferences(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let prefs = models::get_preferences(&state.pool, id).await?;
    Ok(Json(json!({
        "show_module_manager": prefs.show_module_manager
    })))
}

async fn update_user_preferences(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;
    let show = payload.get("show_module_manager").and_then(|v| v.as_bool()).unwrap_or(true);
    let prefs = models::update_preferences(&state.pool, id, show).await?;
    Ok(Json(json!({
        "show_module_manager": prefs.show_module_manager
    })))
}

async fn get_branding(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_any_role(&claims, &["Sostenedor", "Administrador"])?;
    let config = models::get_branding(&state.pool).await?;
    if let Some(c) = config {
        Ok(Json(json!({
            "school_name": c.school_name,
            "school_logo_url": c.school_logo_url,
            "primary_color": c.primary_color,
            "secondary_color": c.secondary_color
        })))
    } else {
        Ok(Json(json!({
            "school_name": "",
            "school_logo_url": "",
            "primary_color": "#1A2B3C",
            "secondary_color": "#243B4F"
        })))
    }
}

async fn update_branding(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_role(&claims, "Sostenedor")?;
    let school_name = payload.get("school_name").and_then(|v| v.as_str()).unwrap_or("");
    let school_logo_url = payload.get("school_logo_url").and_then(|v| v.as_str()).unwrap_or("");
    let primary_color = payload.get("primary_color").and_then(|v| v.as_str()).unwrap_or("#1A2B3C");
    let secondary_color = payload.get("secondary_color").and_then(|v| v.as_str()).unwrap_or("#243B4F");
    let config = models::upsert_branding(&state.pool, school_name, school_logo_url, primary_color, secondary_color).await?;
    Ok(Json(json!({
        "school_name": config.school_name,
        "school_logo_url": config.school_logo_url,
        "primary_color": config.primary_color,
        "secondary_color": config.secondary_color
    })))
}

fn builtin_modules() -> Vec<schoolcbb_common::modules::Module> {
    vec![
        schoolcbb_common::modules::Module { id: "dashboard".into(), name: "Panel de Control".into(), icon: "dashboard".into(), category: "Administración".into(), route: "/".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "students".into(), name: "Gestión de Alumnos".into(), icon: "students".into(), category: "Académico".into(), route: "/students".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "attendance".into(), name: "Asistencia".into(), icon: "attendance".into(), category: "Académico".into(), route: "/attendance".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "grades".into(), name: "Calificaciones".into(), icon: "grades".into(), category: "Académico".into(), route: "/grades".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "agenda".into(), name: "Agenda Escolar".into(), icon: "agenda".into(), category: "Comunicaciones".into(), route: "/agenda".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "notifications".into(), name: "Centro de Mensajería".into(), icon: "notifications".into(), category: "Comunicaciones".into(), route: "/notifications".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "reports".into(), name: "Reportes".into(), icon: "reports".into(), category: "Administración".into(), route: "/reports".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "finance".into(), name: "Finanzas".into(), icon: "config".into(), category: "Administración".into(), route: "/finance".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "users".into(), name: "Usuarios y Perfiles".into(), icon: "users".into(), category: "Sistema".into(), route: "/users".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "courses".into(), name: "Cursos".into(), icon: "book".into(), category: "Académico".into(), route: "/courses".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "enrollments".into(), name: "Matrículas".into(), icon: "clipboard".into(), category: "Académico".into(), route: "/enrollments".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "subjects".into(), name: "Asignaturas".into(), icon: "book".into(), category: "Académico".into(), route: "/subjects".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "academic-years".into(), name: "Años Académicos".into(), icon: "calendar".into(), category: "Administración".into(), route: "/academic-years".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "admission".into(), name: "Admisiones".into(), icon: "users".into(), category: "Administración".into(), route: "/admission".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "grade-levels".into(), name: "Niveles".into(), icon: "book".into(), category: "Académico".into(), route: "/grade-levels".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "classrooms".into(), name: "Salas".into(), icon: "home".into(), category: "Administración".into(), route: "/classrooms".into(), is_favorite: false },
        schoolcbb_common::modules::Module { id: "audit".into(), name: "Auditoría".into(), icon: "file-text".into(), category: "Sistema".into(), route: "/audit".into(), is_favorite: false },
    ]
}

async fn list_modules(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid("Invalid user".into()))?;
    let favs: Vec<(String,)> = sqlx::query_as("SELECT module_id FROM user_favorites WHERE user_id = $1")
        .bind(user_id).fetch_all(&state.pool).await?;
    let fav_set: std::collections::HashSet<String> = favs.into_iter().map(|r| r.0).collect();
    let modules: Vec<schoolcbb_common::modules::Module> = builtin_modules().into_iter()
        .map(|m| schoolcbb_common::modules::Module { is_favorite: fav_set.contains(&m.id), ..m }).collect();
    Ok(Json(json!({ "modules": modules })))
}

async fn toggle_favorite(
    claims: Claims, State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(payload): Json<schoolcbb_common::modules::FavoriteToggle>,
) -> AuthResult<Json<Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::TokenInvalid("Invalid user".into()))?;
    if payload.favorite {
        sqlx::query("INSERT INTO user_favorites (user_id, module_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id).bind(&module_id).execute(&state.pool).await?;
    } else {
        sqlx::query("DELETE FROM user_favorites WHERE user_id = $1 AND module_id = $2")
            .bind(user_id).bind(&module_id).execute(&state.pool).await?;
    }
    Ok(Json(json!({ "module_id": module_id, "favorite": payload.favorite })))
}
