mod config;
mod error;
mod routes;

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

    tracing::info!("Reporting Service connected to database");
    schoolccb_common::db_schema::run(&pool).await;

    let state = AppState {
        pool,
        config: config.clone(),
    };

    let app = Router::new()
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("Reporting Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
