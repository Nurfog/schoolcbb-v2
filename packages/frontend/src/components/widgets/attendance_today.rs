use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client::fetch_attendance_today;

#[component]
pub fn AttendanceTodayWidget() -> Element {
    let data = use_resource(|| async move { fetch_attendance_today().await });

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Asistencia Hoy" }
            }
            div { class: "widget-card-body",
                match data() {
                    Some(Ok(val)) => rsx! {
                        AttendanceContent { data: val.clone() }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "empty-state",
                            p { "Error al cargar: {e}" }
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
fn AttendanceContent(data: Value) -> Element {
    let total = data["total_students"].as_i64().unwrap_or(0);
    let present = data["present"].as_i64().unwrap_or(0);
    let absent = data["absent"].as_i64().unwrap_or(0);
    let late = data["late"].as_i64().unwrap_or(0);
    let justified = data["justified"].as_i64().unwrap_or(0);
    let percentage = data["attendance_percentage"].as_f64().unwrap_or(100.0);

    let bar_class = if percentage >= 95.0 {
        "fill good"
    } else if percentage >= 85.0 {
        "fill warn"
    } else {
        "fill bad"
    };

    rsx! {
        div { class: "kpi-grid",
            div { class: "kpi-item",
                div { class: "kpi-value success", "{total}" }
                div { class: "kpi-label", "Total" }
            }
            div { class: "kpi-item",
                div { class: "kpi-value success", "{present}" }
                div { class: "kpi-label", "Presentes" }
            }
            div { class: "kpi-item",
                div { class: "kpi-value danger", "{absent}" }
                div { class: "kpi-label", "Ausentes" }
            }
            div { class: "kpi-item",
                div { class: "kpi-value warning", "{late}" }
                div { class: "kpi-label", "Atrasos" }
            }
        }
        div { class: "percentage-bar",
            div {
                class: "{bar_class}",
                style: "width: {percentage}%",
            }
        }
        div { class: "percentage-text",
            "Asistencia: " strong { "{percentage:.1}%" }
            "  | Justificados: {justified}"
        }
    }
}
