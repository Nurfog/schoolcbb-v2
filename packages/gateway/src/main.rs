mod graphql;

use std::env;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Json, Request, State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use futures_util::{SinkExt, StreamExt};
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
    finance_url: String,
    reporting_url: String,
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
        finance_url: env::var("FINANCE_URL").unwrap_or_else(|_| "http://localhost:3006".into()),
        reporting_url: env::var("REPORTING_URL").unwrap_or_else(|_| "http://localhost:3007".into()),
    };
    let cors = CorsLayer::new()
        .allow_origin(state.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let schema = graphql::build_schema(&state.sis_url, &state.academic_url, state.client.clone());

    let app = Router::new()
        .route("/health", get(|| async { (StatusCode::OK, axum::Json(serde_json::json!({"status": "ok", "service": "gateway"}))) }))
        .route("/api/auth/*path", any(proxy_identity))
        .route("/api/user/*path", any(proxy_identity))
        .route("/api/students/*path", any(proxy_sis))
        .route("/api/courses/*path", any(proxy_sis))
        .route("/api/enrollments/*path", any(proxy_sis))
        .route("/api/dashboard/*path", any(proxy_sis))
        .route("/api/admission/*path", any(proxy_sis))
        .route("/api/grades/*path", any(proxy_academic))
        .route("/api/academic-years/*path", any(proxy_academic))
        .route("/api/academic/grade-levels/*path", any(proxy_academic))
        .route("/api/academic/audit-log", any(proxy_academic))
        .route("/api/attendance/*path", any(proxy_attendance))
        .route("/api/communications/*path", any(proxy_notifications))
        .route("/api/finance/*path", any(proxy_finance))
        .route("/api/reports/*path", any(proxy_reporting))
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
        .route("/ws", any(ws_proxy))
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

async fn ws_proxy(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let upstream = state.notifications_url.clone();
    ws.on_upgrade(move |socket| handle_ws_proxy(socket, upstream))
}

async fn graphql_playground() -> impl IntoResponse {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html><head><title>SchoolCBB GraphQL</title>
<script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css"/>
</head><body><div id="root"></div>
<script>window.addEventListener('load',function(){GraphQLPlayground.init(document.getElementById('root'),{endpoint:'/graphql'})})</script>
</body></html>"#,
    )
}

async fn graphql_handler(
    Extension(schema): Extension<graphql::AppSchema>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let query = body.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let vars = body.get("variables");

    let mut req = async_graphql::Request::new(query);
    if let Some(v) = vars {
        if let Some(obj) = v.as_object() {
            let vars = async_graphql::Variables::from_json(serde_json::Value::Object(obj.clone()));
            req = req.variables(vars);
        }
    }
    if let Some(auth) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        req = req.data(auth.to_string());
    }

    let resp = schema.execute(req).await;
    Json(serde_json::json!(resp.data))
}

async fn handle_ws_proxy(client_ws: WebSocket, upstream_url: String) {
    let ws_url = upstream_url.replace("http://", "ws://").replace("https://", "wss://");
    let ws_url = format!("{}/ws", ws_url.trim_end_matches('/'));

    match tokio_tungstenite::connect_async(&ws_url).await {
        Ok((upstream_ws, _)) => {
            let (mut client_sender, mut client_receiver) = client_ws.split();
            let (mut upstream_sender, mut upstream_receiver) = upstream_ws.split();

            let c2u = tokio::spawn(async move {
                while let Some(Ok(msg)) = client_receiver.next().await {
                    let data = msg.into_data();
                    if upstream_sender.send(tungstenite::Message::Binary(data)).await.is_err() {
                        break;
                    }
                }
            });

            let u2c = tokio::spawn(async move {
                while let Some(Ok(msg)) = upstream_receiver.next().await {
                    let data = msg.into_data();
                    if client_sender.send(Message::Binary(data)).await.is_err() {
                        break;
                    }
                }
            });

            tokio::select! {
                _ = c2u => {},
                _ = u2c => {},
            }
        }
        Err(e) => {
            tracing::error!("WebSocket proxy: failed to connect to upstream {ws_url}: {e}");
        }
    }
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
proxy_handler!(proxy_finance, "finance");
proxy_handler!(proxy_reporting, "reporting");

async fn proxy_request(state: &AppState, service: &str, req: Request) -> Response {
    let base_url = match service {
        "identity" => &state.identity_url,
        "sis" => &state.sis_url,
        "academic" => &state.academic_url,
        "attendance" => &state.attendance_url,
        "notifications" => &state.notifications_url,
        "finance" => &state.finance_url,
        "reporting" => &state.reporting_url,
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
