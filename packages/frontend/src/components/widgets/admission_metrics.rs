use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn AdmissionMetricsWidget() -> Element {
    let data = use_resource(|| async move { client::fetch_admission_metrics().await });

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Métricas de Admisión" }
                span { "Pipeline overview" }
            }
            div { class: "widget-card-body",
                match data() {
                    Some(Ok(val)) => rsx! { MetricsContent { data: val.clone() } },
                    Some(Err(e)) => rsx! { div { class: "empty-state", p { "Error: {e}" } } },
                    None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }
    }
}

#[component]
fn MetricsContent(data: Value) -> Element {
    let total = data["total_prospects"].as_i64().unwrap_or(0);
    let finalized = data["total_finalized"].as_i64().unwrap_or(0);
    let conversion = data["conversion_rate"].as_str().unwrap_or("0.0");
    let recent = data["recent_activities_7d"].as_i64().unwrap_or(0);
    let stages = data["stages"].as_array().cloned().unwrap_or_default();
    let sources = data["sources"].as_array().cloned().unwrap_or_default();

    let stage_bars: Vec<Element> = stages.iter().map(|s| {
        let name = s["stage_name"].as_str().unwrap_or("").to_string();
        let count = s["count"].as_i64().unwrap_or(0);
        let max_count = stages.iter().map(|x| x["count"].as_i64().unwrap_or(0)).max().unwrap_or(1).max(1);
        let pct = (count as f64 / max_count as f64) * 100.0;
        rsx! {
            div { class: "metric-bar-row",
                span { class: "metric-bar-label", "{name}" }
                div { class: "metric-bar-track",
                    div { class: "metric-bar-fill", style: "width: {pct}%" }
                }
                span { class: "metric-bar-value", "{count}" }
            }
        }
    }).collect();

    let source_items: Vec<Element> = sources.iter().map(|s| {
        let name = s["source"].as_str().unwrap_or("sin origen").to_string();
        let count = s["count"].as_i64().unwrap_or(0);
        rsx! {
            div { class: "source-item",
                span { "{name}" }
                span { class: "source-count", "{count}" }
            }
        }
    }).collect();

    rsx! {
        div { class: "kpi-grid",
            div { class: "kpi-item",
                span { class: "kpi-value primary", "{total}" }
                span { class: "kpi-label", "Total Postulantes" }
            }
            div { class: "kpi-item",
                span { class: "kpi-value success", "{finalized}" }
                span { class: "kpi-label", "Matriculados" }
            }
            div { class: "kpi-item",
                span { class: "kpi-value info", "{conversion}%" }
                span { class: "kpi-label", "Tasa Conversión" }
            }
            div { class: "kpi-item",
                span { class: "kpi-value warning", "{recent}" }
                span { class: "kpi-label", "Actividades (7d)" }
            }
        }
        h4 { style: "margin: 12px 0 8px; font-size: 13px; color: #555;", "Postulantes por Etapa" }
        div { class: "metric-bar-list", { stage_bars.into_iter() } }
        SourcesSection { source_items: source_items }
    }
}

#[component]
fn SourcesSection(source_items: Vec<Element>) -> Element {
    if source_items.is_empty() {
        return rsx! {};
    }
    rsx! {
        h4 { style: "margin: 12px 0 8px; font-size: 13px; color: #555;", "Por Origen" }
        div { class: "source-list", { source_items.into_iter() } }
    }
}
