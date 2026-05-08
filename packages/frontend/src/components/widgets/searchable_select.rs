use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn SearchableSelect(
    fetch_url: String,
    results_key: String,
    label_key: String,
    value_key: String,
    placeholder: String,
    on_select: EventHandler<String>,
    initial_label: Option<String>,
) -> Element {
    let mut query = use_signal(|| String::new());
    let mut is_open = use_signal(|| false);
    let init_val = initial_label.clone().unwrap_or_default();
    let mut selected_label = use_signal(|| init_val);
    let init_some = initial_label.is_some();
    let mut has_selected = use_signal(|| init_some);

    let results = use_resource(move || {
        let q = query();
        let url = fetch_url.clone();
        let rk = results_key.clone();
        async move {
            if q.len() < 1 {
                return Vec::<Value>::new();
            }
            let separator = if url.contains('?') { "&" } else { "?" };
            match client::fetch_json(&format!("{}{}search={}", url, separator, q)).await {
                Ok(data) => data[&rk].as_array().cloned().unwrap_or_default(),
                Err(_) => Vec::new(),
            }
        }
    });

    let items = results().unwrap_or_default();
    let display_val = if has_selected() { selected_label() } else { query() };

    let f_label_key = label_key.clone();
    let f_value_key = value_key.clone();

    let rendered_items: Vec<_> = items.iter().map(|item| {
        let label = item.get(&f_label_key).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let value = item.get(&f_value_key).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let label_c = label.clone();
        let value_c = value.clone();
        let key = value.clone();
        rsx! {
            div {
                class: "searchable-select-item",
                key: "{key}",
                onmousedown: move |_| {
                    selected_label.set(label_c.clone());
                    has_selected.set(true);
                    is_open.set(false);
                    query.set("".to_string());
                    on_select.call(value_c.clone());
                },
                "{label}"
            }
        }
    }).collect();

    rsx! {
        div { class: "searchable-select",
            input {
                class: "login-input",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{display_val}",
                oninput: move |evt: FormEvent| {
                    query.set(evt.value());
                    is_open.set(true);
                    has_selected.set(false);
                },
                onfocus: move |_| if !has_selected() { is_open.set(true); },
                onblur: move |_| is_open.set(false),
                autocomplete: "off",
            }
            if has_selected() {
                span {
                    class: "searchable-select-clear",
                    onclick: move |_| {
                        selected_label.set("".to_string());
                        has_selected.set(false);
                        on_select.call("".to_string());
                    },
                    svg { role: "presentation", view_box: "0 0 24 24", width: "14", height: "14",
                        line { x1: "18", y1: "6", x2: "6", y2: "18" }
                        line { x1: "6", y1: "6", x2: "18", y2: "18" }
                    }
                }
            }
            if is_open() && query().len() >= 1 {
                div { class: "searchable-select-dropdown",
                    if items.is_empty() {
                        div { class: "searchable-select-empty", "Sin resultados" }
                    } else {
                        {rendered_items.into_iter()}
                    }
                }
            }
        }
    }
}
