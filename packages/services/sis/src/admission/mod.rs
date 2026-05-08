mod prospects;
mod stages;
mod activities;
mod documents;
mod classrooms;
mod upload;

pub use stages::seed_pipeline_stages;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(stages::router())
        .merge(prospects::router())
        .merge(activities::router())
        .merge(documents::router())
        .merge(classrooms::router())
        .merge(upload::router())
}
