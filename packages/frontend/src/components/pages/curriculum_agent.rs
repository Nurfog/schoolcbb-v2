use dioxus::prelude::*;

use crate::api::client;

#[derive(Clone)]
struct SearchResultItem {
    text: String,
    fuente: String,
    nivel: String,
    asignatura: String,
    #[allow(dead_code)]
    score: usize,
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[component]
pub fn CurriculumAgent() -> Element {
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResultItem>::new);
    let mut loading = use_signal(|| false);
    let mut searched = use_signal(|| false);

    let do_search = move |_| {
        let q = query().trim().to_string();
        if q.is_empty() || loading() {
            return;
        }
        query.set(String::new());
        loading.set(true);
        searched.set(true);

        spawn(async move {
            let resp = client::post_json(
                "/api/curriculum/search",
                &serde_json::json!({ "q": q, "limit": 10 }),
            )
            .await;

            loading.set(false);

            match resp {
                Ok(data) => {
                    let items: Vec<SearchResultItem> = data["results"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .map(|v| SearchResultItem {
                                    text: v["text"].as_str().unwrap_or("").to_string(),
                                    fuente: v["fuente"].as_str().unwrap_or("").to_string(),
                                    nivel: v["nivel"].as_str().unwrap_or("").to_string(),
                                    asignatura: v["asignatura"].as_str().unwrap_or("").to_string(),
                                    score: v["score"].as_u64().unwrap_or(0) as usize,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    results.set(items);
                }
                Err(e) => {
                    results.set(vec![SearchResultItem {
                        text: format!("Error al buscar: {e}"),
                        fuente: String::new(),
                        nivel: String::new(),
                        asignatura: String::new(),
                        score: 0,
                    }]);
                }
            }
        });
    };

    let on_key_down = move |e: Event<KeyboardData>| {
        if e.key() == Key::Enter {
            let q = query().trim().to_string();
            if q.is_empty() || loading() {
                return;
            }
            query.set(String::new());
            loading.set(true);
            searched.set(true);

            spawn(async move {
                let resp = client::post_json(
                    "/api/curriculum/search",
                    &serde_json::json!({ "q": q, "limit": 10 }),
                )
                .await;

                loading.set(false);

                match resp {
                    Ok(data) => {
                        let items: Vec<SearchResultItem> = data["results"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .map(|v| SearchResultItem {
                                        text: v["text"].as_str().unwrap_or("").to_string(),
                                        fuente: v["fuente"].as_str().unwrap_or("").to_string(),
                                        nivel: v["nivel"].as_str().unwrap_or("").to_string(),
                                        asignatura: v["asignatura"].as_str().unwrap_or("").to_string(),
                                        score: v["score"].as_u64().unwrap_or(0) as usize,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        results.set(items);
                    }
                    Err(e) => {
                        results.set(vec![SearchResultItem {
                            text: format!("Error al buscar: {e}"),
                            fuente: String::new(),
                            nivel: String::new(),
                            asignatura: String::new(),
                            score: 0,
                        }]);
                    }
                }
            });
        }
    };

    let content = if loading() {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "Buscando en la base de conocimientos del currículum nacional..."
                }
            }
        }
    } else if !results().is_empty() {
        let items = results();
        rsx! {
            p { class: "result-count", "Se encontraron {items.len()} resultados" }
            {items.iter().map(|r| {
                let has_meta = !r.nivel.is_empty() || !r.asignatura.is_empty();
                rsx! {
                    div { class: "search-result-card",
                        p { class: "result-text", "{truncate(&r.text, 300)}" }
                        if has_meta {
                            div { class: "result-meta",
                                if !r.nivel.is_empty() {
                                    span { class: "result-tag nivel", "{r.nivel}" }
                                }
                                if !r.asignatura.is_empty() {
                                    span { class: "result-tag asignatura", "{r.asignatura}" }
                                }
                            }
                        }
                        if !r.fuente.is_empty() {
                            div { class: "result-fuente", "Fuente: {r.fuente}" }
                        }
                    }
                }
            })}
        }
    } else if searched() {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "No se encontraron resultados para tu búsqueda."
                }
            }
        }
    } else {
        rsx! {
            div { class: "chat-message bot-message",
                div { class: "message-bubble bot-bubble",
                    "Ingresa un término de búsqueda para consultar el Currículum Nacional chileno."
                }
            }
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Currículum Nacional" }
            p { "Bases Curriculares, normativa y programas de estudio del Ministerio de Educación" }
        }
        div { class: "curriculum-search-container",
            div { class: "chat-input-area",
                input {
                    class: "chat-input",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    onkeydown: on_key_down,
                    placeholder: "Ej: Decreto 67, OA Matemática 1° Básico..."
                }
                button {
                    class: "btn btn-primary",
                    disabled: loading() || query().trim().is_empty(),
                    onclick: do_search,
                    { if loading() { "Buscando..." } else { "Buscar" } }
                }
            }
            div { class: "chat-messages",
                {content}
            }
        }
    }
}
