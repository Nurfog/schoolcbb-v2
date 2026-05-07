use std::env;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use http_body_util::BodyExt;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    identity_url: String,
    sis_url: String,
    academic_url: String,
    attendance_url: String,
    notifications_url: String,
    frontend_url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    dotenvy::dotenv().ok();

    let state = AppState {
        client: reqwest::Client::new(),
        identity_url: env::var("IDENTITY_URL").unwrap_or_else(|_| "http://localhost:3001".into()),
        sis_url: env::var("SIS_URL").unwrap_or_else(|_| "http://localhost:3002".into()),
        academic_url: env::var("ACADEMIC_URL").unwrap_or_else(|_| "http://localhost:3003".into()),
        attendance_url: env::var("ATTENDANCE_URL").unwrap_or_else(|_| "http://localhost:3004".into()),
        notifications_url: env::var("NOTIFICATIONS_URL").unwrap_or_else(|_| "http://localhost:3005".into()),
        frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
    };

    let cors = CorsLayer::new()
        .allow_origin(state.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/auth/*path", any(proxy_identity))
        .route("/api/students/*path", any(proxy_sis))
        .route("/api/dashboard/*path", any(proxy_sis))
        .route("/api/grades/*path", any(proxy_academic))
        .route("/api/attendance/*path", any(proxy_attendance))
        .route("/ws", any(proxy_notifications))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{host}:{port}");
    tracing::info!("Gateway starting on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

macro_rules! proxy_handler {
    ($name:ident, $service:expr) => {
        async fn $name(state: State<AppState>, req: Request) -> Response {
            proxy_request(&state, $service, req).await
        }
    };
}

proxy_handler!(proxy_identity, "identity");
proxy_handler!(proxy_sis, "sis");
proxy_handler!(proxy_academic, "academic");
proxy_handler!(proxy_attendance, "attendance");
proxy_handler!(proxy_notifications, "notifications");

async fn proxy_request(state: &AppState, service: &str, req: Request) -> Response {
    let base_url = match service {
        "identity" => &state.identity_url,
        "sis" => &state.sis_url,
        "academic" => &state.academic_url,
        "attendance" => &state.attendance_url,
        "notifications" => &state.notifications_url,
        _ => return (StatusCode::BAD_REQUEST, "Unknown service").into_response(),
    };

    let path = req.uri().path();
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    let upstream_url = format!("{base_url}{path}{query}");

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body_bytes = req.into_body().collect().await.unwrap_or_default().to_bytes();

    let upstream_req = state
        .client
        .request(method, &upstream_url)
        .headers(headers)
        .body(body_bytes);

    match upstream_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let body = resp.bytes().await.unwrap_or_default();
            let mut response = Response::new(axum::body::Body::from(body));
            *response.status_mut() = status;
            for (key, value) in headers.iter() {
                if key != "host" && key != "transfer-encoding" {
                    response.headers_mut().insert(key, value.clone());
                }
            }
            response
        }
        Err(e) => {
            tracing::error!("Proxy error to {service}: {e}");
            (StatusCode::BAD_GATEWAY, format!("Service {service} unavailable")).into_response()
        }
    }
}
