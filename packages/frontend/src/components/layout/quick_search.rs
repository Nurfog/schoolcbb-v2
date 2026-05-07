use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client::search_students;

#[component]
pub fn QuickSearch(is_open: Signal<bool>) -> Element {
    let mut query = use_signal(|| String::new());
    let mut results = use_signal(|| Vec::<Value>::new());
    let mut loading = use_signal(|| false);
    let mut selected_idx = use_signal(|| 0i32);

    let _ = use_resource(move || {
        let q = query();
        async move {
            if q.len() < 2 {
                results.set(Vec::new());
                loading.set(false);
                return;
            }
            loading.set(true);
            match search_students(&q).await {
                Ok(data) => {
                    let items = data["students"].as_array().cloned().unwrap_or_default();
                    results.set(items);
                }
                Err(_) => {
                    results.set(Vec::new());
                }
            }
            loading.set(false);
        }
    });

    let close = move |_| is_open.set(false);

    let on_input = move |evt: FormEvent| {
        query.set(evt.value());
        selected_idx.set(0);
    };

    let on_key_down = move |evt: KeyboardEvent| {
        let k = evt.key();
        match k {
            Key::Escape => is_open.set(false),
            Key::ArrowDown => {
                let len = results.read().len() as i32;
                if selected_idx() < len - 1 {
                    selected_idx.set(selected_idx() + 1);
                }
            }
            Key::ArrowUp => {
                if selected_idx() > 0 {
                    selected_idx.set(selected_idx() - 1);
                }
            }
            Key::Enter => {
                if results.read().get(selected_idx() as usize).is_some() {
                    is_open.set(false);
                }
            }
            _ => {}
        }
    };

    let loading_state = loading();
    let query_text = query();
    let student_list = results.read().clone();
    rsx! {
        div { class: "quick-search-overlay", onclick: close,
            div { class: "quick-search-modal", onclick: |e| e.stop_propagation(),
                div { class: "quick-search-input-wrap",
                    svg { role: "presentation", view_box: "0 0 24 24",
                        circle { cx: "11", cy: "11", r: "8" }
                        line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
                    }
                    input {
                        placeholder: "Buscar alumnos por nombre o RUT...",
                        value: "{query_text}",
                        autofocus: "true",
                        oninput: on_input,
                        onkeydown: on_key_down,
                    }
                    span { class: "esc-hint", "ESC" }
                }
                div { class: "quick-search-results",
                    if loading_state {
                        div { class: "loading-spinner", "Buscando..." }
                    } else if query_text.len() < 2 {
                        div { class: "quick-search-empty", "Escribe al menos 2 caracteres" }
                    } else {
                        SearchResultsList {
                            students: student_list,
                            selected_idx: selected_idx(),
                            on_select: move |_| is_open.set(false),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SearchResultsList(students: Vec<Value>, selected_idx: i32, on_select: EventHandler<()>) -> Element {
    if students.is_empty() {
        return rsx! {
            div { class: "quick-search-empty", "Sin resultados" }
        };
    }

    rsx! {
        for (i, s) in students.iter().enumerate() {
            SearchResultRow {
                key: "{i}",
                student: s.clone(),
                highlighted: i == selected_idx as usize,
                on_click: on_select.clone(),
            }
        }
    }
}

#[component]
fn SearchResultRow(student: Value, highlighted: bool, on_click: EventHandler<()>) -> Element {
    let full_name = format!(
        "{} {}",
        student["first_name"].as_str().unwrap_or(""),
        student["last_name"].as_str().unwrap_or("")
    );
    let detail = format!(
        "{} {}",
        student["grade_level"].as_str().unwrap_or(""),
        student["section"].as_str().unwrap_or("")
    );
    let initial = student["first_name"]
        .as_str()
        .and_then(|n| n.chars().next())
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".into());

    rsx! {
        a {
            class: "quick-search-result-item",
            "data-selected": "{highlighted}",
            href: "#",
            onclick: move |_| on_click.call(()),
            div { class: "avatar", "{initial}" }
            div { class: "info",
                div { class: "name", "{full_name}" }
                div { class: "detail", "{detail}" }
            }
        }
    }
}
