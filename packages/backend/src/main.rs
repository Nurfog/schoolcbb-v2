mod config;
mod db;
mod error;
mod routes;
mod ws;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use config::Config;
use ws::hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ws_hub: Arc<WsHub>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connected to database");

    db::init::run(&pool).await;

    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_hub = Arc::new(WsHub::new(ws_tx.clone()));

    let state = AppState {
        pool,
        ws_hub,
    };

    let cors = CorsLayer::new()
        .allow_origin(config.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .merge(routes::dashboard::router())
        .merge(routes::attendance::router())
        .merge(routes::grades::router())
        .merge(routes::auth::router())
        .merge(routes::ws::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = config.addr();
    tracing::info!("Backend starting on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
