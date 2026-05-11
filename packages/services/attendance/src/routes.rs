mod alerts;
mod attendance;
mod reports;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(attendance::router())
        .merge(reports::router())
        .merge(alerts::router())
}
