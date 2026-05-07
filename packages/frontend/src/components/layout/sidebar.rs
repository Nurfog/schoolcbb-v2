use dioxus::prelude::*;

#[component]
pub fn Sidebar() -> Element {
    let mut collapsed = use_signal(|| false);

    let toggle = move |_| {
        collapsed.set(!collapsed());
    };

    let sidebar_class = if collapsed() {
        "sidebar collapsed"
    } else {
        "sidebar"
    };

    rsx! {
        nav { class: "{sidebar_class}",
            div { class: "sidebar-header",
                div { class: "logo", "SC" }
                span { class: "brand", "SchoolCBB" }
            }
            div { class: "sidebar-nav",
                NavItem { icon: "📊", label: "Dashboard", active: true }
                NavItem { icon: "📋", label: "Asistencia", active: false }
                NavItem { icon: "📝", label: "Calificaciones", active: false }
                NavItem { icon: "👥", label: "Alumnos", active: false }
                NavItem { icon: "📅", label: "Agenda", active: false }
                NavItem { icon: "⚙️", label: "Configuración", active: false }
            }
            div { class: "sidebar-footer",
                button { class: "toggle-btn", onclick: toggle,
                    if collapsed() { "➡️" } else { "⬅️" }
                }
            }
        }
    }
}

#[component]
fn NavItem(icon: String, label: String, active: bool) -> Element {
    let class = if active {
        "nav-item active"
    } else {
        "nav-item"
    };

    rsx! {
        a { class: "{class}", href: "#",
            span { class: "icon", "{icon}" }
            span { class: "label", "{label}" }
        }
    }
}
