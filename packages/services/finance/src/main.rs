mod config;
mod error;
mod grpc;
mod payment_gateway;
mod routes;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use payment_gateway::{MockGateway, PaymentGateway, PaymentGatewayConfig, WebpayGateway};

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub gateway: Option<&'static dyn PaymentGateway>,
}

fn init_gateway() -> Option<&'static dyn PaymentGateway> {
    let cfg = PaymentGatewayConfig::from_env()?;
    let gateway: &'static dyn PaymentGateway = match cfg.provider.as_str() {
        "webpay" => Box::leak(Box::new(WebpayGateway { config: cfg })),
        _ => Box::leak(Box::new(MockGateway)),
    };
    tracing::info!("Payment gateway initialized: {}", gateway.provider_name());
    Some(gateway)
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

    tracing::info!("Finance Service connected to database");
    schoolcbb_common::db_schema::run(&pool).await;

    let gateway = init_gateway();

    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        gateway,
    };

    let grpc_pool = pool.clone();
    let grpc_addr = format!("{}:{}", config.host, config.port + 1000);
    tokio::spawn(async move {
        grpc::start_grpc_server(grpc_pool, grpc_addr).await;
    });

    let app = Router::new()
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = config.addr();
    tracing::info!("Finance Service starting on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
