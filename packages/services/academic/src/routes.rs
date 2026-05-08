mod subjects;
mod periods;
mod categories;
mod grades;
mod reports;
mod academic_years;
pub mod grade_levels;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(subjects::router())
        .merge(periods::router())
        .merge(categories::router())
        .merge(grades::router())
        .merge(reports::router())
        .merge(academic_years::router())
        .merge(grade_levels::router())
}
