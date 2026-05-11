use dioxus::prelude::*;

use crate::api::client;

fn format_date(date_str: &str) -> String {
    if date_str.len() >= 10 {
        format!(
            "{}/{}/{}",
            &date_str[8..10],
            &date_str[5..7],
            &date_str[0..4]
        )
    } else {
        date_str.to_string()
    }
}

fn get_events(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data["events"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn AgendaPage() -> Element {
    let events = use_resource(|| client::fetch_json("/api/dashboard/agenda"));

    rsx! {
        div { class: "page-header",
            h1 { "Agenda" }
            p { "Calendario escolar y próximos eventos" }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Fecha" }
                        th { "Evento" }
                        th { "Tipo" }
                        th { "Descripción" }
                    }
                }
                tbody {
                    match events() {
                        Some(Ok(data)) => {
                            let list = get_events(&data);
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "4", class: "empty-state", "No hay eventos programados" } } }
                            } else {
                                rsx! {
                                    for evt in list {
                                        AgendaEventRow { evt: evt }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "4", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "4", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

#[component]
fn AgendaEventRow(evt: serde_json::Value) -> Element {
    let title = evt["title"].as_str().unwrap_or("").to_string();
    let date = evt["date"].as_str().unwrap_or("").to_string();
    let etype = evt["event_type"].as_str().unwrap_or("Evento").to_string();
    let desc = evt["description"].as_str().unwrap_or("").to_string();
    let type_class = format!("event-type-{}", etype.to_lowercase());

    rsx! {
        tr {
            td { "{format_date(&date)}" }
            td { class: "cell-name", "{title}" }
            td { span { class: "{type_class}", "{etype}" } }
            td { "{desc}" }
        }
    }
}
