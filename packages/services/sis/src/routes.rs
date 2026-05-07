mod models;
mod dashboard;
mod students;
mod import;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(dashboard::router())
        .merge(students::router())
        .merge(import::router())
}
