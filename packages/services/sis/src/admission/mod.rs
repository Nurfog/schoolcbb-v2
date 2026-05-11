mod prospects;
mod stages;
mod activities;
mod documents;
mod classrooms;
mod upload;
mod metrics;
mod custom_fields;

pub use stages::seed_pipeline_stages;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(stages::router())
        .merge(prospects::router())
        .merge(activities::router())
        .merge(documents::router())
        .merge(classrooms::router())
        .merge(upload::router())
        .merge(metrics::router())
        .merge(custom_fields::router())
}
