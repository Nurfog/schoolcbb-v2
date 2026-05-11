mod fees;
mod payments;
mod scholarships;
mod payment_gateway;

pub fn router() -> axum::Router<crate::AppState> {
    axum::Router::new()
        .merge(fees::router())
        .merge(payments::router())
        .merge(scholarships::router())
        .merge(payment_gateway::router())
}
