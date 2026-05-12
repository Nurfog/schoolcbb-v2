use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::kpi_card::KpiCard;
use crate::components::widgets::simple_chart::{BarChart, DoughnutChart};

#[component]
pub fn RootDashboard() -> Element {
    let summary = use_resource(|| async move { client::fetch_json("/api/admin/stats/summary").await });
    let monthly = use_resource(|| async move { client::fetch_json("/api/admin/stats/monthly").await });
    let distribution = use_resource(|| async move { client::fetch_json("/api/admin/stats/license-distribution").await });
    let corporations = use_resource(|| async move { client::fetch_json("/api/admin/corporations").await });
    let activity = use_resource(|| async move { client::fetch_json("/api/admin/activity-log?limit=20").await });

    rsx! {
        div { class: "page-header",
            h1 { "Panel de Administración Root" }
            p { "Gestión global de corporaciones, licencias y monitoreo de la plataforma" }
        }
        KpiSection { summary_data: summary() }
        ChartsRow { monthly_data: monthly(), dist_data: distribution() }
        CorpSection { corps_data: corporations() }
        ActivitySection { log_data: activity() }
    }
}

fn unwrap_data(data: &Option<Result<Value, String>>) -> Option<&Value> {
    match data {
        Some(Ok(v)) => Some(v),
        _ => None,
    }
}

#[component]
fn KpiSection(summary_data: Option<Result<Value, String>>) -> Element {
    let data = match unwrap_data(&summary_data) {
        Some(d) => d,
        None => return rsx! { div { class: "loading-spinner", "Cargando..." } },
    };
    let active = data["active_corporations"].as_i64().unwrap_or(0);
    let schools = data["total_schools"].as_i64().unwrap_or(0);
    let students = data["total_students"].as_i64().unwrap_or(0);
    let employees = data["total_employees"].as_i64().unwrap_or(0);
    let licenses = data["active_licenses"].as_i64().unwrap_or(0);
    let expiring = data["expiring_licenses"].as_i64().unwrap_or(0);
    rsx! {
        div { class: "kpi-grid",
            KpiCard { label: "Corporaciones Activas".to_string(), value: "{active}", color: Some("#4fc3f7".to_string()), icon: Some("🏢".to_string()) }
            KpiCard { label: "Colegios".to_string(), value: "{schools}", color: Some("#66bb6a".to_string()), icon: Some("🏫".to_string()) }
            KpiCard { label: "Alumnos".to_string(), value: "{students}", color: Some("#ff7043".to_string()), icon: Some("👨‍🎓".to_string()) }
            KpiCard { label: "Empleados".to_string(), value: "{employees}", color: Some("#ab47bc".to_string()), icon: Some("👔".to_string()) }
            KpiCard { label: "Licencias Activas".to_string(), value: "{licenses}", color: Some("#26c6da".to_string()), icon: Some("✅".to_string()) }
            KpiCard { label: "Por Vencer (30d)".to_string(), value: "{expiring}", color: Some("#ffa726".to_string()), icon: Some("⚠️".to_string()) }
        }
    }
}

#[component]
fn ChartsRow(monthly_data: Option<Result<Value, String>>, dist_data: Option<Result<Value, String>>) -> Element {
    rsx! {
        div { class: "chart-row",
            match unwrap_data(&monthly_data) {
                Some(data) => {
                    let list = data["monthly"].as_array().cloned().unwrap_or_default();
                    if !list.is_empty() {
                        rsx! {
                            div { class: "chart-card",
                                h3 { "Ingresos Mensuales (12 meses)" }
                                BarChart { data: list, label_key: "month".to_string(), value_key: "revenue".to_string(), height: Some(220), color: Some("#4fc3f7".to_string()) }
                            }
                        }
                    } else { rsx! { div { class: "chart-card", h3 { "Ingresos Mensuales" }, p { "Sin datos" } } } }
                }
                None => rsx! { div { class: "loading-spinner", "Cargando ingresos..." } },
            }
            match unwrap_data(&dist_data) {
                Some(data) => {
                    let raw = data["distribution"].as_array().cloned().unwrap_or_default();
                    let dist: Vec<Value> = raw.iter().map(|d| serde_json::json!({"name": d[0], "count": d[1]})).collect();
                    if !dist.is_empty() {
                        rsx! {
                            div { class: "chart-card",
                                h3 { "Distribución de Planes" }
                                DoughnutChart { data: dist, label_key: "name".to_string(), value_key: "count".to_string(), size: Some(200) }
                            }
                        }
                    } else { rsx! { div { class: "chart-card", h3 { "Distribución de Planes" }, p { "Sin datos" } } } }
                }
                None => rsx! { div { class: "loading-spinner", "Cargando distribución..." } },
            }
        }
    }
}

#[component]
fn CorpSection(corps_data: Option<Result<Value, String>>) -> Element {
    let data = match unwrap_data(&corps_data) {
        Some(d) => d,
        None => return rsx! { div { class: "loading-spinner", "Cargando corporaciones..." } },
    };
    let list = data["corporations"].as_array().cloned().unwrap_or_default();
    let rows: Vec<Element> = list.into_iter().map(|c| {
        let id = c["id"].as_str().unwrap_or("").to_string();
        let name = c["name"].as_str().unwrap_or("").to_string();
        let rut = c["rut"].as_str().unwrap_or("-").to_string();
        let plan = c["plan_name"].as_str().unwrap_or("-").to_string();
        let status = c["license_status"].as_str().unwrap_or("-").to_string();
        let schools = c["total_schools"].as_i64().unwrap_or(0);
        let students = c["total_students"].as_i64().unwrap_or(0);
        let sc = if status == "active" { "badge-success" } else { "badge-warning" };
        rsx! {
            tr { key: "{id}",
                td { "{name}" } td { "{rut}" } td { "{plan}" }
                td { span { class: "badge {sc}", "{status}" } }
                td { "{schools}" } td { "{students}" }
            }
        }
    }).collect();
    rsx! {
        div { class: "section-card",
            h3 { "Corporaciones" }
            table { class: "data-table",
                thead { tr { th { "Nombre" } th { "RUT" } th { "Plan" } th { "Estado" } th { "Colegios" } th { "Alumnos" } } }
                tbody { {rows.into_iter()} }
            }
        }
    }
}

#[component]
fn ActivitySection(log_data: Option<Result<Value, String>>) -> Element {
    let data = match unwrap_data(&log_data) {
        Some(d) => d,
        None => return rsx! { div { class: "loading-spinner", "Cargando actividad..." } },
    };
    let list = data["activity_log"].as_array().cloned().unwrap_or_default();
    if list.is_empty() {
        return rsx! { div { class: "section-card", h3 { "Actividad Reciente" }, p { "Sin actividad registrada" } } };
    }
    let rows: Vec<Element> = list.into_iter().map(|a| {
        let admin = a["admin"].as_str().unwrap_or("").to_string();
        let action = a["action"].as_str().unwrap_or("").to_string();
        let entity = a["entity_type"].as_str().unwrap_or("").to_string();
        let created = a["created_at"].as_str().unwrap_or("").to_string();
        rsx! {
            tr {
                td { "{created}" } td { "{admin}" }
                td { span { class: "badge badge-info", "{action}" } }
                td { "{entity}" }
            }
        }
    }).collect();
    rsx! {
        div { class: "section-card",
            h3 { "Actividad Reciente" }
            table { class: "data-table",
                thead { tr { th { "Fecha" } th { "Admin" } th { "Acción" } th { "Entidad" } } }
                tbody { {rows.into_iter()} }
            }
        }
    }
}
