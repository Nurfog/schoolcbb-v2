mod fees;
mod payments;
mod scholarships;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(fees::router())
        .merge(payments::router())
        .merge(scholarships::router())
}
