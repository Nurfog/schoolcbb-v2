use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

fn jwt_claims() -> Option<Value> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok().flatten()?;
    let token = storage.get_item("jwt_token").ok().flatten()?;
    let parts: Vec<&str> = token.split('.').collect();
    let payload_b64 = parts.get(1)?;
    let decoded = window.atob(payload_b64).ok()?;
    serde_json::from_str(&decoded).ok()
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn role_label(role: &str) -> &'static str {
    match role {
        "Sostenedor" => "Sostenedor",
        "Administrador" => "Admin",
        "Director" => "Director",
        "UTP" => "UTP",
        "Profesor" => "Profesor",
        "Apoderado" => "Apoderado",
        "Alumno" => "Alumno",
        _ => "Usuario",
    }
}

#[component]
pub fn Sidebar() -> Element {
    let claims = use_signal(jwt_claims);
    let user_name = claims()
        .as_ref()
        .and_then(|c| c["name"].as_str())
        .unwrap_or("Usuario")
        .to_string();
    let user_role = claims()
        .as_ref()
        .and_then(|c| c["role"].as_str())
        .unwrap_or("")
        .to_string();
    let user_initials = initials(&user_name);

    let fav_ver = use_context::<Signal<u32>>();
    let modules = use_resource(move || async move {
        let _ = fav_ver();
        client::fetch_json("/api/user/modules").await
    });

    rsx! {
        nav { class: "sidebar", role: "navigation", aria_label: "Navegación principal",
            div { class: "sidebar-header",
                div { class: "logo", "SC" }
                span { class: "brand", "SchoolCBB" }
            }

            div { class: "sidebar-nav",
                div { class: "sidebar-user",
                    div { class: "user-avatar-sidebar", "{user_initials}" }
                    div { class: "user-info",
                        span { class: "user-name", "{user_name}" }
                        span { class: "user-role", "{role_label(&user_role)}" }
                    }
                }

                a { class: "nav-item", href: "/dashboard",
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
                        }
                    }
                    span { class: "label", "Dashboard" }
                }

                div { class: "nav-section-label", "Acceso Rápido"}
                div { class: "sidebar-favs",
                    match modules() {
                        Some(Ok(data)) => {
                            let favs: Vec<Value> = data["modules"].as_array().cloned().unwrap_or_default()
                                .into_iter().filter(|m| m["is_favorite"].as_bool().unwrap_or(false)).collect();
                            if favs.is_empty() {
                                rsx! { p { class: "empty-hint", "Sin favoritos" } }
                            } else {
                                let items: Vec<_> = favs.into_iter().map(|m| {
                                    let route = m["route"].as_str().unwrap_or("/").to_string();
                                    let name = m["name"].as_str().unwrap_or("--").to_string();
                                    rsx! {
                                        a { key: "{route}", class: "fav-link", href: "{route}",
                                            span { class: "fav-dot" }
                                            span { "{name}" }
                                        }
                                    }
                                }).collect();
                                rsx! { {items.into_iter()} }
                            }
                        }
                        _ => rsx! { p { class: "empty-hint", "Cargando..." } },
                    }
                }
            }

            div { class: "sidebar-footer",
                div { class: "nav-section-label", "Sistema"}

                a { class: "nav-item config-item", href: "/",
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
                            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
                            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
                        }
                    }
                    span { class: "label", "Module Manager" }
                }

                a { class: "nav-item config-item", href: "/config",
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            circle { cx: "12", cy: "12", r: "3" }
                            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
                        }
                    }
                    span { class: "label", "Configuración" }
                }



                button { class: "nav-item logout", onclick: move |_| {
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.remove_item("jwt_token");
                            }
                            let _ = window.location().set_href("/login");
                        }
                    },
                    span { class: "icon",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                            polyline { points: "16 17 21 12 16 7" }
                            line { x1: "21", y1: "12", x2: "9", y2: "12" }
                        }
                    }
                    span { class: "label", "Cerrar Sesión" }
                }
            }
        }
    }
}
