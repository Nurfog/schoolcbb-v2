mod models;
mod dashboard;
pub mod students;
mod courses;
mod enrollments;
mod import;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(dashboard::router())
        .merge(students::router())
        .merge(courses::router())
        .merge(enrollments::router())
        .merge(import::router())
}
