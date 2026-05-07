mod models;
mod dashboard;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(dashboard::router())
}
