mod config;
mod error;
mod grpc;
mod routes;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tonic::transport::Server;
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

    tracing::info!("Academic Service connected to database");
    schoolcbb_common::db_schema::run(&pool).await;

    let state = AppState {
        pool,
        config: config.clone(),
    };

    let app = Router::new()
        .merge(routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let rest_addr = config.addr();
    let grpc_addr = format!("{}:{}", config.host, config.port + 100);
    let pool_clone = state.pool.clone();

    tokio::spawn(async move {
        tracing::info!("Academic gRPC starting on {grpc_addr}");
        let grpc = grpc::AcademicGrpc { pool: pool_clone };
        Server::builder()
            .add_service(schoolcbb_proto::academic_service_server::AcademicServiceServer::new(grpc))
            .serve(grpc_addr.parse().unwrap())
            .await
            .unwrap();
    });

    tracing::info!("Academic REST starting on {rest_addr}");
    let listener = tokio::net::TcpListener::bind(&rest_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
