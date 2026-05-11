mod fees;
mod payment_gateway;
mod payments;
mod scholarships;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(fees::router())
        .merge(payments::router())
        .merge(scholarships::router())
        .merge(payment_gateway::router())
}
