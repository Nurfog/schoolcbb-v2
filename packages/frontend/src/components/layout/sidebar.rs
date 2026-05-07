use dioxus::prelude::*;

fn current_path() -> String {
    web_sys::window()
        .and_then(|w| w.location().pathname().ok())
        .unwrap_or_else(|| "/".to_string())
}

#[component]
pub fn Sidebar() -> Element {
    let mut collapsed = use_signal(|| false);

    let toggle = move |_| {
        collapsed.set(!collapsed());
    };

    let sc = if collapsed() { "sidebar collapsed" } else { "sidebar" };
    let path = current_path();

    rsx! {
        nav { class: "{sc}", role: "navigation", aria_label: "Navegación principal",
            div { class: "sidebar-header",
                div { class: "logo", "SC" }
                span { class: "brand", "SchoolCBB" }
            }
            div { class: "sidebar-nav",
                div { class: "nav-section-label", "Principal" }
                NavItem { icon_svg: "dashboard", label: "Dashboard", href: "/", path: path.clone() }
                NavItem { icon_svg: "students", label: "Alumnos", href: "/students", path: path.clone() }
                NavItem { icon_svg: "attendance", label: "Asistencia", href: "/attendance", path: path.clone() }
                NavItem { icon_svg: "grades", label: "Calificaciones", href: "/grades", path: path.clone() }
                div { class: "nav-section-label", "Gestión" }
                NavItem { icon_svg: "agenda", label: "Agenda", href: "/agenda", path: path.clone() }
                NavItem { icon_svg: "notifications", label: "Notificaciones", href: "/notifications", path: path.clone() }
                NavItem { icon_svg: "reports", label: "Reportes", href: "/reports", path: path.clone() }
                NavItem { icon_svg: "config", label: "Finanzas", href: "/finance", path: path.clone() }
                div { class: "nav-section-label", "Sistema" }
                NavItem { icon_svg: "users", label: "Usuarios", href: "/users", path: path.clone() }
                NavItem { icon_svg: "config", label: "Configuración", href: "#", path: path.clone() }
            }
            div { class: "sidebar-footer",
                button { class: "toggle-btn", onclick: toggle,
                    if collapsed() {
                        svg { width: "18", height: "18", view_box: "0 0 24 24",
                            polyline { points: "9 18 15 12 9 6" }
                        }
                    } else {
                        svg { width: "18", height: "18", view_box: "0 0 24 24",
                            polyline { points: "15 18 9 12 15 6" }
                        }
                    }
                }
                if !collapsed() {
                    button { class: "nav-item logout", onclick: move |_| {
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.remove_item("jwt_token");
                                }
                            }
                            let nav = navigator();
                            nav.push("/login");
                        },
                        span { class: "icon", {render_icon("logout")} }
                        span { class: "label", "Cerrar Sesión" }
                    }
                }
            }
        }
    }
}

#[component]
fn NavItem(icon_svg: String, label: String, href: String, path: String) -> Element {
    let is_active = if href == "#" { false } else { path == href || (href != "/" && path.starts_with(&href)) };
    let class = if is_active { "nav-item active" } else { "nav-item" };

    rsx! {
        a { class: "{class}", href: "{href}",
            span { class: "icon", {render_icon(&icon_svg)} }
            span { class: "label", "{label}" }
        }
    }
}

fn render_icon(name: &str) -> Element {
    match name {
        "dashboard" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
            }
        },
        "students" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
                circle { cx: "9", cy: "7", r: "4" }
                path { d: "M23 21v-2a4 4 0 0 0-3-3.87" }
                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
            }
        },
        "attendance" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M12 20h9" }
                path { d: "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" }
            }
        },
        "grades" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" }
                path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" }
                path { d: "M8 7h8" }
                path { d: "M8 11h6" }
            }
        },
        "agenda" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" }
                line { x1: "16", y1: "2", x2: "16", y2: "6" }
                line { x1: "8", y1: "2", x2: "8", y2: "6" }
                line { x1: "3", y1: "10", x2: "21", y2: "10" }
            }
        },
        "notifications" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" }
                path { d: "M13.73 21a2 2 0 0 1-3.46 0" }
            }
        },
        "reports" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                polyline { points: "14 2 14 8 20 8" }
                line { x1: "16", y1: "13", x2: "8", y2: "13" }
                line { x1: "16", y1: "17", x2: "8", y2: "17" }
                polyline { points: "10 9 9 9 8 9" }
            }
        },
        "config" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                circle { cx: "12", cy: "12", r: "3" }
                path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
            }
        },
        "users" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" }
                circle { cx: "9", cy: "7", r: "4" }
                path { d: "M23 21v-2a4 4 0 0 0-3-3.87" }
                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
            }
        },
        "logout" => rsx! {
            svg { role: "presentation", view_box: "0 0 24 24",
                path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                polyline { points: "16 17 21 12 16 7" }
                line { x1: "21", y1: "12", x2: "9", y2: "12" }
            }
        },
        _ => rsx! { svg { role: "presentation", view_box: "0 0 24 24",
            circle { cx: "12", cy: "12", r: "10" }
        }}
    }
}
