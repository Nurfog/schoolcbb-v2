mod courses;
mod dashboard;
mod enrollments;
mod import;
mod models;
pub mod students;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(dashboard::router())
        .merge(students::router())
        .merge(courses::router())
        .merge(enrollments::router())
        .merge(import::router())
}
