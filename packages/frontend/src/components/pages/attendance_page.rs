use dioxus::prelude::*;

use crate::api::client;

fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

fn current_month() -> u32 {
    js_sys::Date::new_0().get_month() + 1
}

fn get_students(json: &serde_json::Value) -> Vec<serde_json::Value> {
    json["students"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn AttendancePage() -> Element {
    let mut selected_year = use_signal(current_year);
    let mut selected_month = use_signal(current_month);
    let mut data = use_resource(move || client::fetch_attendance_monthly(selected_year(), selected_month()));

    let on_year_change = move |evt: Event<FormData>| {
        if let Ok(y) = evt.value().parse::<i32>() {
            selected_year.set(y);
            data.restart();
        }
    };

    let on_month_change = move |evt: Event<FormData>| {
        if let Ok(m) = evt.value().parse::<u32>() {
            selected_month.set(m);
            data.restart();
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Asistencia" }
            p { "Reporte mensual de asistencia" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                label { "Año:" }
                select { value: "{selected_year}", onchange: on_year_change,
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            div { class: "filter-group",
                label { "Mes:" }
                select { value: "{selected_month}", onchange: on_month_change,
                    option { value: "1", "Enero" }
                    option { value: "2", "Febrero" }
                    option { value: "3", "Marzo" }
                    option { value: "4", "Abril" }
                    option { value: "5", "Mayo" }
                    option { value: "6", "Junio" }
                    option { value: "7", "Julio" }
                    option { value: "8", "Agosto" }
                    option { value: "9", "Septiembre" }
                    option { value: "10", "Octubre" }
                    option { value: "11", "Noviembre" }
                    option { value: "12", "Diciembre" }
                }
            }
        }
        div { class: "data-table-container",
            {
                match data() {
                    Some(Ok(json)) => {
                        let summary = json["summary"].as_object().cloned().unwrap_or_default();
                        let students = get_students(&json);
                        let total = summary.get("total_students").and_then(|v| v.as_i64()).unwrap_or(0);
                        let below85 = summary.get("below_general_threshold").and_then(|v| v.as_i64()).unwrap_or(0);
                        let below75 = summary.get("below_nee_threshold").and_then(|v| v.as_i64()).unwrap_or(0);
                        rsx! {
                            div { class: "summary-cards",
                                div { class: "summary-card",
                                    span { class: "summary-value", "{total}" }
                                    span { class: "summary-label", "Total Alumnos" }
                                }
                                div { class: "summary-card warning",
                                    span { class: "summary-value", "{below85}" }
                                    span { class: "summary-label", "Bajo 85%" }
                                }
                                div { class: "summary-card danger",
                                    span { class: "summary-value", "{below75}" }
                                    span { class: "summary-label", "Bajo 75%" }
                                }
                            }
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Estudiante" }
                                        th { "RUT" }
                                        th { "Días" }
                                        th { "Presente" }
                                        th { "Ausente" }
                                        th { "Atraso" }
                                        th { "Justificado" }
                                        th { "% Asistencia" }
                                    }
                                }
                                tbody {
                                    for s in students {
                                        AttendanceRow { student: s }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => rsx! {
                        div { class: "empty-state", "Error: {e}" }
                    },
                    None => rsx! {
                        div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } }
                    },
                }
            }
        }
    }
}

#[component]
fn AttendanceRow(student: serde_json::Value) -> Element {
    let name = student["student_name"].as_str().unwrap_or("-").to_string();
    let rut = student["rut"].as_str().unwrap_or("-").to_string();
    let total_days = student["total_days"].as_i64().unwrap_or(0);
    let present = student["present"].as_i64().unwrap_or(0);
    let absent = student["absent"].as_i64().unwrap_or(0);
    let late = student["late"].as_i64().unwrap_or(0);
    let justified = student["justified"].as_i64().unwrap_or(0);
    let pct = if total_days > 0 { present as f64 / total_days as f64 * 100.0 } else { 100.0 };
    let pct_str = format!("{:.1}%", pct);
    let pct_class = if pct < 75.0 { "pct-danger" } else if pct < 85.0 { "pct-warning" } else { "pct-good" };

    rsx! {
        tr {
            td { class: "cell-name", "{name}" }
            td { "{rut}" }
            td { "{total_days}" }
            td { "{present}" }
            td { "{absent}" }
            td { "{late}" }
            td { "{justified}" }
            td { class: "{pct_class}", "{pct_str}" }
        }
    }
}
