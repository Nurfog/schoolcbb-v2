mod ws;

use axum::Router;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub ws_hub: Arc<ws::hub::WsHub>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let (ws_tx, _) = broadcast::channel::<String>(100);
    let ws_hub = Arc::new(ws::hub::WsHub::new(ws_tx.clone()));

    let state = AppState { ws_hub };

    let app = Router::new()
        .merge(ws::routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3005".into());
    let addr = format!("{host}:{port}");

    tracing::info!("Notifications Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
