use dioxus::prelude::*;

use crate::api::client;

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        let day = &date_str[8..10];
        let month = &date_str[5..7];
        let year = &date_str[0..4];
        format!("{}/{}/{}", day, month, year)
    } else {
        date_str.to_string()
    }
}

#[component]
pub fn NotificationsPage() -> Element {
    let mut messages = use_resource(|| client::fetch_json("/api/communications/messages"));
    let mut show_compose = use_signal(|| false);

    rsx! {
        div { class: "page-header",
            h1 { "Mensajes" }
            p { "Bandeja de mensajes y comunicaciones" }
        }
        div { class: "page-toolbar",
            button {
                class: "btn-primary",
                onclick: move |_| show_compose.set(true),
                "Nuevo Mensaje"
            }
        }
        if show_compose() {
            ComposeModal { is_open: show_compose, on_sent: move || messages.restart() }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Asunto" }
                        th { "Estado" }
                        th { "Fecha" }
                    }
                }
                tbody {
                    match messages() {
                        Some(Ok(data)) => {
                            let list = data["messages"].as_array().cloned().unwrap_or_default();
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "3", class: "empty-state", "No hay mensajes" } } }
                            } else {
                                rsx! {
                                    for msg in list {
                                        MessageRow { msg: msg }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "3", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "3", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

#[component]
fn MessageRow(msg: serde_json::Value) -> Element {
    let subject = msg["subject"].as_str().unwrap_or("Sin asunto").to_string();
    let is_read = msg["read"].as_bool().unwrap_or(true);
    let created = msg["created_at"].as_str().unwrap_or("").to_string();
    let row_class = if !is_read { "unread" } else { "" };

    rsx! {
        tr { class: "{row_class}",
            td { class: "cell-name",
                if !is_read {
                    span { class: "unread-dot" }
                }
                span { "{subject}" }
            }
            td {
                if is_read {
                    span { class: "status-inactive", "Leído" }
                } else {
                    span { class: "status-active", "Nuevo" }
                }
            }
            td { "{format_date(&created)}" }
        }
    }
}

#[component]
fn ComposeModal(is_open: Signal<bool>, on_sent: EventHandler) -> Element {
    let mut receiver = use_signal(|| "".to_string());
    let mut subject = use_signal(|| "".to_string());
    let mut body = use_signal(|| "".to_string());
    let mut sending = use_signal(|| false);

    let do_send = move |_| {
        if subject().is_empty() || body().is_empty() {
            return;
        }
        sending.set(true);
        let payload = serde_json::json!({
            "receiver_id": receiver(),
            "subject": subject(),
            "body": body(),
        });
        spawn(async move {
            let _ = client::post_json("/api/communications/messages", &payload).await;
            sending.set(false);
            is_open.set(false);
            on_sent.call(());
        });
    };

    rsx! {
        div { class: "quick-search-overlay",
            div { class: "quick-search-modal compose-modal",
                div { class: "quick-search-header",
                    h3 { "Nuevo Mensaje" }
                    button { class: "close-btn", onclick: move |_| is_open.set(false),
                        svg { role: "presentation", view_box: "0 0 24 24", width: "20", height: "20",
                            line { x1: "18", y1: "6", x2: "6", y2: "18" }
                            line { x1: "6", y1: "6", x2: "18", y2: "18" }
                        }
                    }
                }
                div { class: "compose-body",
                    div { class: "field",
                        label { "Destinatario ID (UUID)" }
                        input { class: "login-input", value: "{receiver}", placeholder: "UUID del destinatario",
                            oninput: move |evt| receiver.set(evt.value()),
                        }
                    }
                    div { class: "field",
                        label { "Asunto" }
                        input { class: "login-input", value: "{subject}", placeholder: "Asunto del mensaje",
                            oninput: move |evt| subject.set(evt.value()),
                        }
                    }
                    div { class: "field",
                        label { "Mensaje" }
                        textarea { class: "login-input compose-textarea", value: "{body}", placeholder: "Escriba su mensaje...",
                            rows: "5",
                            oninput: move |evt| body.set(evt.value()),
                        }
                    }
                    button { class: "login-btn", onclick: do_send, disabled: sending(),
                        if sending() { "Enviando..." } else { "Enviar Mensaje" }
                    }
                }
            }
        }
    }
}
