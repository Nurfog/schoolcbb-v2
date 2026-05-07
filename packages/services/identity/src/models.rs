use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub rut: String,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked: bool,
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, rut, name, email, password_hash, role, active FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, rut, name, email, password_hash, role, active FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert_user(
    pool: &PgPool,
    rut: &str,
    name: &str,
    email: &str,
    password: &str,
    role: &str,
) -> Result<UserRow, sqlx::Error> {
    let hash = hash_password(password);

    let id = Uuid::new_v4();
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, rut, name, email, password_hash, role, active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        RETURNING id, rut, name, email, password_hash, role, active
        "#,
    )
    .bind(id)
    .bind(rut)
    .bind(name)
    .bind(email)
    .bind(&hash)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    duration_days: i64,
) -> Result<(String, RefreshTokenRow), sqlx::Error> {
    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let token_hash = hash_token(&token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(duration_days);
    let id = Uuid::new_v4();

    let row = sqlx::query_as::<_, RefreshTokenRow>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked)
        VALUES ($1, $2, $3, $4, false)
        RETURNING id, user_id, token_hash, expires_at, revoked
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok((token, row))
}

pub async fn find_refresh_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
    let token_hash = hash_token(token);
    sqlx::query_as::<_, RefreshTokenRow>(
        r#"
        SELECT id, user_id, token_hash, expires_at, revoked
        FROM refresh_tokens
        WHERE token_hash = $1 AND revoked = false AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
}

pub async fn revoke_refresh_token(pool: &PgPool, token_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revoke_all_user_tokens(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

pub async fn seed_admin(pool: &PgPool) -> Result<(), sqlx::Error> {
    let exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM users WHERE email = $1",
    )
    .bind("admin@colegio.cl")
    .fetch_one(pool)
    .await?;

    if exists.0 > 0 {
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"admin123", &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, rut, name, email, password_hash, role, active)
         VALUES ($1, '11.111.111-1', 'Administrador', 'admin@colegio.cl', $2, 'Administrador', true)",
    )
    .bind(Uuid::new_v4())
    .bind(&hash)
    .execute(pool)
    .await?;

    tracing::info!("Admin user created: admin@colegio.cl");
    Ok(())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}
