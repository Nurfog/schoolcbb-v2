use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn ModuleManager() -> Element {
    let mut modules = use_resource(|| client::fetch_json("/api/user/modules"));
    let mut search = use_signal(String::new);

    rsx! {
        div { class: "module-manager",
            div { class: "module-search",
                input {
                    class: "module-search-input",
                    placeholder: "Buscar módulos...",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
            }
            div { class: "module-grid",
                match modules() {
                    Some(Ok(data)) => {
                        let list: Vec<Value> = data["modules"].as_array().cloned().unwrap_or_default()
                            .into_iter().filter(|m| {
                                let q = search().to_lowercase();
                                q.is_empty() || m["name"].as_str().unwrap_or("").to_lowercase().contains(&q)
                            }).collect();
                        rsx! {
                            for m in list {
                                ModuleTile { module: m, on_toggle: move || modules.restart() }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { p { class: "empty-state", "Error: {e}" } },
                    None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }
    }
}

#[component]
fn ModuleTile(module: Value, on_toggle: EventHandler) -> Element {
    let id = module["id"].as_str().unwrap_or("").to_string();
    let name = module["name"].as_str().unwrap_or("").to_string();
    let icon = module["icon"].as_str().unwrap_or("dashboard").to_string();
    let route = module["route"].as_str().unwrap_or("/").to_string();
    let is_fav = module["is_favorite"].as_bool().unwrap_or(false);

    let star_cls = if is_fav { "active" } else { "" };

    let do_toggle = move |evt: Event<MouseData>| {
        evt.prevent_default();
        let mid = id.clone();
        let new_fav = !is_fav;
        spawn(async move {
            let _ = client::post_json(&format!("/api/user/favorites/{}", mid), &serde_json::json!({ "module_id": mid, "favorite": new_fav })).await;
            on_toggle.call(());
        });
    };

    rsx! {
        a { class: "module-tile", href: "{route}",
            div { class: "tile-star {star_cls}", onclick: do_toggle,
                svg { role: "presentation", view_box: "0 0 24 24",
                    polygon { points: "12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" }
                }
            }
            div { class: "tile-icon {icon}",
                svg { role: "presentation", view_box: "0 0 24 24",
                    { module_icon(icon.clone()) }
                }
            }
            span { class: "tile-name", "{name}" }
        }
    }
}

fn module_icon(icon: String) -> Element {
    match icon.as_str() {
        "dashboard" => rsx! { rect { x: "3", y: "3", width: "7", height: "7", rx: "1" } rect { x: "14", y: "3", width: "7", height: "7", rx: "1" } rect { x: "3", y: "14", width: "7", height: "7", rx: "1" } rect { x: "14", y: "14", width: "7", height: "7", rx: "1" } },
        "students" | "users" => rsx! { path { d: "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" } circle { cx: "9", cy: "7", r: "4" } path { d: "M23 21v-2a4 4 0 0 0-3-3.87" } path { d: "M16 3.13a4 4 0 0 1 0 7.75" } },
        "attendance" => rsx! { path { d: "M12 20h9" } path { d: "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" } },
        "grades" => rsx! { path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" } path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" } path { d: "M8 7h8" } path { d: "M8 11h6" } },
        "agenda" => rsx! { rect { x: "3", y: "4", width: "18", height: "18", rx: "2", ry: "2" } line { x1: "16", y1: "2", x2: "16", y2: "6" } line { x1: "8", y1: "2", x2: "8", y2: "6" } line { x1: "3", y1: "10", x2: "21", y2: "10" } },
        "notifications" | "megaphone" => rsx! { path { d: "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" } path { d: "M13.73 21a2 2 0 0 1-3.46 0" } },
        "reports" | "chart" => rsx! { path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" } polyline { points: "14 2 14 8 20 8" } line { x1: "16", y1: "13", x2: "8", y2: "13" } line { x1: "16", y1: "17", x2: "8", y2: "17" } },
        "book" => rsx! { path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" } path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" } path { d: "M8 7h8" } path { d: "M8 11h6" } },
        "clipboard" => rsx! { path { d: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" } path { d: "M9 2h6a2 2 0 0 1 2 2v6H7V4a2 2 0 0 1 2-2z" } path { d: "M8 7h8" } path { d: "M8 11h6" } path { d: "M8 15h6" } },
        "money" => rsx! { path { d: "M12 1v22" } path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" } },
        "config" | "gear" => rsx! { circle { cx: "12", cy: "12", r: "3" } path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" } },
        _ => rsx! { circle { cx: "12", cy: "12", r: "10" } },
    }
}
