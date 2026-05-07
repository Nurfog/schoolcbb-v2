mod attendance;
mod reports;
mod alerts;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(attendance::router())
        .merge(reports::router())
        .merge(alerts::router())
}
