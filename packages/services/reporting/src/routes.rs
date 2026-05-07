mod certificate;
mod concentration;
mod final_record;
mod sige;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(certificate::router())
        .merge(concentration::router())
        .merge(final_record::router())
        .merge(sige::router())
}
