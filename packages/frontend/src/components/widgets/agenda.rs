use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client::fetch_agenda;

#[component]
pub fn AgendaWidget() -> Element {
    let data = use_resource(|| async move { fetch_agenda().await });

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Agenda / Próximos Eventos" }
            }
            div { class: "widget-card-body",
                match data() {
                    Some(Ok(val)) => rsx! {
                        AgendaContent { data: val.clone() }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "empty-state",
                            p { "Error: {e}" }
                        }
                    },
                    None => rsx! {
                        div { class: "loading-spinner", "Cargando..." }
                    },
                }
            }
        }
    }
}

#[component]
fn AgendaContent(data: Value) -> Element {
    let events = data["events"].as_array().map(|a| a.clone()).unwrap_or_default();

    if events.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "icon", "📅" }
                p { "No hay eventos programados" }
            }
        };
    }

    rsx! {
        div {
            for event in events {
                EventItem {
                    title: event["title"].as_str().unwrap_or("--").to_string(),
                    date_str: event["date"].as_str().unwrap_or("").to_string(),
                    event_type: event["event_type"].as_str().unwrap_or("Evento").to_string(),
                }
            }
        }
    }
}

#[component]
fn EventItem(title: String, date_str: String, event_type: String) -> Element {
    let (day, month) = parse_date(&date_str);
    let type_lower = event_type.to_lowercase();

    rsx! {
        div { class: "event-item",
            div { class: "event-date-badge {type_lower}",
                span { class: "day", "{day}" }
                span { class: "month", "{month}" }
            }
            div { class: "event-details",
                div { class: "event-title", "{title}" }
                div { class: "event-type", "{event_type}" }
            }
        }
    }
}

#[allow(dead_code)]
fn parse_date(date_str: &str) -> (String, String) {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3 {
        let day = parts[2].to_string();
        let month_num: u32 = parts[1].parse().unwrap_or(1);
        let months = [
            "Ene", "Feb", "Mar", "Abr", "May", "Jun",
            "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
        ];
        let month = months.get(month_num as usize - 1).unwrap_or(&"---").to_string();
        (day, month)
    } else {
        ("--".into(), "---".into())
    }
}
