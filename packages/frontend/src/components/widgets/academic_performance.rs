use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn AcademicPerformanceWidget() -> Element {
    let data = use_resource(|| async {
        client::fetch_dashboard_summary().await
    });

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Rendimiento Académico" }
                span { "Resumen general" }
            }
            div { class: "widget-card-body",
                match data() {
                    Some(Ok(val)) => rsx! {
                        AcademicContent { data: val.clone() }
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
fn AcademicContent(data: Value) -> Element {
    let subjects = data.get("subjects_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let students = data.get("total_students").and_then(|v| v.as_i64()).unwrap_or(0);
    let avg_grade = data.get("average_grade").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let passing = data.get("passing_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let passing_str = format!("{:.0}%", passing);
    let avg_str = format!("{:.1}", avg_grade);

    let passing_class = if passing >= 90.0 { "kpi-value success" } else if passing >= 75.0 { "kpi-value warning" } else { "kpi-value danger" };
    let bar_class = if passing >= 90.0 { "bar-fill good" } else if passing >= 75.0 { "bar-fill warn" } else { "bar-fill bad" };
    let bar_width = format!("{:.0}%", passing);

    let summary = if passing >= 90.0 {
        "Rendimiento general sobresaliente"
    } else if passing >= 75.0 {
        "Rendimiento general adecuado"
    } else {
        "Se requiere mejorar el rendimiento general"
    };

    rsx! {
        div { class: "kpi-grid",
            div { class: "kpi-item",
                span { class: "kpi-value info", "{students}" }
                span { class: "kpi-label", "Estudiantes" }
            }
            div { class: "kpi-item",
                span { class: "kpi-value primary", "{subjects}" }
                span { class: "kpi-label", "Asignaturas" }
            }
            div { class: "kpi-item",
                span { class: "kpi-value success", "{avg_str}" }
                span { class: "kpi-label", "Promedio Gral" }
            }
            div { class: "kpi-item",
                span { class: "{passing_class}", "{passing_str}" }
                span { class: "kpi-label", "Aprobación" }
            }
        }
        div { class: "percentage-bar",
            div { class: "{bar_class}", style: "width: {bar_width}" }
        }
        p { class: "summary-text", "{summary}" }
    }
}
