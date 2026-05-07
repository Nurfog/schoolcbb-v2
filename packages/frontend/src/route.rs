use dioxus::prelude::*;

use crate::components::layout::dashboard_grid::DashboardGrid;

#[derive(Routable, Clone)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Dashboard {},
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        DashboardGrid {}
    }
}
