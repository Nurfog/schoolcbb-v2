use dioxus::prelude::*;

use crate::components::layout::{sidebar::Sidebar, topbar::Topbar};
use crate::route::Route;

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "app-layout",
            Sidebar {}
            div { class: "main-area",
                Topbar {}
                div { class: "dashboard-content",
                    Router::<Route> {}
                }
            }
        }
    }
}
