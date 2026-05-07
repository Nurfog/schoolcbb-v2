use dioxus::prelude::*;

use super::quick_search::QuickSearch;
use crate::api::client;
use crate::route::has_token;

#[component]
pub fn Topbar() -> Element {
    let mut show_search = use_signal(|| false);
    let unread = use_resource(|| async {
        if !has_token() { return Ok(serde_json::json!({"unread": 0})); }
        client::fetch_json("/api/communications/messages/unread-count").await
    });

    let open_search = move |_| {
        show_search.set(true);
    };

    let unread_count: i64 = match unread() {
        Some(Ok(data)) => data["unread"].as_i64().unwrap_or(0),
        _ => 0,
    };

    rsx! {
        header { class: "topbar",
            div { class: "search-bar", onclick: open_search,
                span { class: "search-icon",
                    svg { width: "16", height: "16", view_box: "0 0 24 24",
                        circle { cx: "11", cy: "11", r: "8" }
                        line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
                    }
                }
                input { placeholder: "Buscar alumnos por nombre o RUT...", "type": "text", disabled: "true" }
                div { class: "search-shortcut",
                    kbd { "Ctrl" }
                    kbd { "K" }
                }
            }
            div { class: "topbar-actions",
                a { href: "/notifications",
                    button { class: "notif-btn",
                        svg { role: "presentation", view_box: "0 0 24 24",
                            path { d: "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" }
                            path { d: "M13.73 21a2 2 0 0 1-3.46 0" }
                        }
                        if unread_count > 0 {
                            div { class: "notif-badge", "{unread_count}" }
                        }
                    }
                }
                div { class: "user-avatar", "AD" }
            }
        }
        if show_search() {
            QuickSearch { is_open: show_search }
        }
    }
}
