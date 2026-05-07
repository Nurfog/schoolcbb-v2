mod subjects;
mod periods;
mod categories;
mod grades;
mod reports;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(subjects::router())
        .merge(periods::router())
        .merge(categories::router())
        .merge(grades::router())
        .merge(reports::router())
}
