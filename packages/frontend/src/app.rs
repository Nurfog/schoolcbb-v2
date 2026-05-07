use dioxus::prelude::*;

use crate::components::layout::{sidebar::Sidebar, topbar::Topbar};
use crate::route::Route;

#[component]
pub fn App() -> Element {
    let fav_ver = use_signal(|| 0u32);
    use_context_provider(|| fav_ver);

    rsx! {
        a { class: "skip-link", href: "#main-content", "Saltar al contenido" }
        div { class: "app-layout",
            Sidebar {}
            div { class: "main-area",
                Topbar {}
                div { id: "main-content", class: "dashboard-content",
                    Router::<Route> {}
                }
            }
        }
    }
}
