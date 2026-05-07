mod config;
mod error;
mod ws;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub ws_hub: Arc<ws::hub::WsHub>,
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

    tracing::info!("Notifications Service connected to database");
    schoolcbb_common::db_schema::run(&pool).await;

    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_hub = Arc::new(ws::hub::WsHub::new(ws_tx.clone()));

    let state = AppState {
        pool,
        config: config.clone(),
        ws_hub,
    };

    let addr = config.addr();

    let app = Router::new()
        .merge(ws::routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    tracing::info!("Notifications Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
