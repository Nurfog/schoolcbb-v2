use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client::fetch_student_alerts;

#[component]
pub fn StudentAlertsWidget() -> Element {
    let data = use_resource(|| async move { fetch_student_alerts().await });

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Alertas de Alumnos" }
                span { "Últimos 30 días" }
            }
            div { class: "widget-card-body",
                match data() {
                    Some(Ok(val)) => rsx! {
                        AlertsContent { data: val.clone() }
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
fn AlertsContent(data: Value) -> Element {
    let alerts = data["alerts"]
        .as_array()
        .map(|a| a.clone())
        .unwrap_or_default();

    if alerts.is_empty() {
        return rsx! {
            div { class: "empty-state",
                div { class: "icon", "✅" }
                p { "Sin alertas de asistencia" }
            }
        };
    }

    rsx! {
        div {
            for alert in alerts {
                AlertItem {
                    name: alert["student_name"].as_str().unwrap_or("--").to_string(),
                    percentage: alert["attendance_percentage"].as_f64().unwrap_or(100.0),
                    absences: alert["total_absences"].as_i64().unwrap_or(0),
                    severity: alert["severity"].as_str().unwrap_or("Bajo").to_string(),
                }
            }
        }
    }
}

#[component]
fn AlertItem(name: String, percentage: f64, absences: i64, severity: String) -> Element {
    rsx! {
        div { class: "alert-item",
            div { class: "alert-badge {severity}" }
            div { class: "alert-info",
                div { class: "alert-name", "{name}" }
                div { class: "alert-detail",
                    "Ausencias: {absences} | Asistencia: {percentage:.1}%"
                }
            }
            span { class: "alert-severity {severity}", "{severity}" }
        }
    }
}
