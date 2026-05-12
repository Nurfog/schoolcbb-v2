mod admin;
mod client;
mod config;
mod error;
mod models;
mod routes;
mod scheduler;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();
    let config = Arc::new(Config::from_env());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Identity Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::warn!("SQLx migrations skipped: {e}");
    }

    models::seed_admin(&pool)
        .await
        .expect("Failed to seed admin user");
    tracing::info!("Admin user seeded (check .env for credentials)");

    models::seed_roles(&pool).await;
    models::seed_permission_definitions(&pool).await;
    tracing::info!("Roles and permissions seeded");

    models::seed_root_admin(&pool).await;
    models::seed_license_plans(&pool).await;

    models::seed_default_school(&pool).await;

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    scheduler::start(pool.clone()).await;

    let app = Router::new()
        .merge(routes::router())
        .merge(admin::admin_router())
        .merge(client::client_router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("Identity Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
