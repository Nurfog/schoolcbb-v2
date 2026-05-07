use dioxus::prelude::*;

#[component]
pub fn Topbar() -> Element {
    rsx! {
        header { class: "topbar",
            div { class: "search-bar",
                span { class: "search-icon", "🔍" }
                input { placeholder: "Buscar alumnos, cursos, asistencias...", "type": "text" }
            }
            div { class: "topbar-actions",
                span { "🔔" }
                div { class: "user-avatar", "AD" }
            }
        }
    }
}
