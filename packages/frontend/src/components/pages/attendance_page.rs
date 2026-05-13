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
    let mut data =
        use_resource(move || client::fetch_attendance_monthly(selected_year(), selected_month()));

    let mut course_search = use_signal(String::new);
    let mut selected_course = use_signal(|| None::<serde_json::Value>);
    let mut date_str = use_signal(|| {
        let now = js_sys::Date::new_0();
        format!("{:04}-{:02}-{:02}", now.get_full_year() as i32, now.get_month() + 1, now.get_date() as u32)
    });
    let mut course_date_result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut course_date_loading = use_signal(|| false);
    let courses_rsrc = use_resource(move || {
        let q = course_search();
        async move {
            client::fetch_json(&format!("/api/courses?search={}", q.replace(' ', "%20"))).await
        }
    });

    let fetch_course_attendance = move |_| {
        if let Some(ref course) = selected_course() {
            let cid = course["id"].as_str().unwrap_or("").to_string();
            let d = date_str();
            course_date_loading.set(true);
            course_date_result.set(None);
            spawn(async move {
                let res = client::fetch_attendance_by_course_date(&cid, &d).await;
                course_date_loading.set(false);
                course_date_result.set(Some(res));
            });
        }
    };

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
        div { class: "widget-card", style: "margin-top: 20px; padding: 16px;",
            h2 { "Asistencia por Curso y Fecha" }
            div { class: "page-toolbar", style: "margin-top: 8px;",
                div { class: "filter-group",
                    label { "Curso:" }
                    {
                        match selected_course() {
                            Some(ref c) => {
                                let cname = c["name"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div { class: "selected-student", style: "display: inline-flex; align-items: center; gap: 8px;",
                                        span { "{cname}" }
                                        button { class: "btn-icon", "aria-label": "Cerrar", onclick: move |_| { selected_course.set(None); course_date_result.set(None); }, "✕" }
                                    }
                                }
                            }
                            None => rsx! {
                                input {
                                    class: "search-input",
                                    value: "{course_search}",
                                    oninput: move |evt| course_search.set(evt.value()),
                                    placeholder: "Buscar curso..."
                                }
                            }
                        }
                    }
                    {
                        let list = match courses_rsrc() {
                            Some(Ok(j)) => j["courses"].as_array().cloned().unwrap_or_default(),
                            _ => vec![],
                        };
                        if !list.is_empty() && course_search().len() >= 2 {
                            let items: Vec<Element> = list.iter().map(|c| {
                                let cid = c["id"].as_str().unwrap_or("").to_string();
                                let cname = c["name"].as_str().unwrap_or("").to_string();
                                let level = c["grade_level"].as_str().unwrap_or("").to_string();
                                let section = c["section"].as_str().unwrap_or("").to_string();
                                let cinfo = format!("{} - {}", level, section);
                                rsx! {
                                    div {
                                        class: "search-result-item",
                                        onclick: move |_| {
                                            selected_course.set(Some(serde_json::json!({
                                                "id": cid.clone(),
                                                "name": cname.clone(),
                                            })));
                                            course_search.set(String::new());
                                        },
                                        span { "{cname}" }
                                        span { class: "result-rut", "{cinfo}" }
                                    }
                                }
                            }).collect();
                            rsx! { div { class: "search-results", { items.into_iter() } } }
                        } else { rsx! {} }
                    }
                }
                div { class: "filter-group",
                    label { "Fecha:" }
                    input {
                        class: "form-input",
                        value: "{date_str}",
                        oninput: move |evt| date_str.set(evt.value()),
                        placeholder: "YYYY-MM-DD"
                    }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: selected_course().is_none() || course_date_loading(),
                    onclick: fetch_course_attendance,
                    if course_date_loading() { "Cargando..." } else { "Consultar Asistencia" }
                }
            }
            {
                match course_date_result() {
                    Some(Ok(j)) => {
                        let records = j["records"].as_array().cloned().unwrap_or_default();
                        let course_name = j["course_name"].as_str().unwrap_or("");
                        let date_val = j["date"].as_str().unwrap_or("");
                        let record_rows: Vec<(String, String, String, String, String)> = records.iter().map(|rec| {
                            let sname = rec["student_name"].as_str().unwrap_or("-").to_string();
                            let srut = rec["rut"].as_str().unwrap_or("-").to_string();
                            let status = rec["status"].as_str().unwrap_or("-").to_string();
                            let time = rec["time"].as_str().unwrap_or("—").to_string();
                            let status_class = match status.as_str() {
                                "presente" | "Presente" => "badge badge-success",
                                "ausente" | "Ausente" => "badge badge-warning",
                                "atraso" | "Atraso" => "badge badge-info",
                                _ => "",
                            };
                            (sname, srut, status, time, status_class.to_string())
                        }).collect();
                        rsx! {
                            div { class: "report-result", style: "margin-top: 12px;",
                                p { "Curso: {course_name} — Fecha: {date_val}" }
                                table { class: "data-table",
                                    thead { tr {
                                        th { "Estudiante" }
                                        th { "RUT" }
                                        th { "Estado" }
                                        th { "Hora" }
                                    }}
                                    tbody { for (sname, srut, status, time, status_class) in &record_rows {
                                        tr {
                                            td { class: "cell-name", "{sname}" }
                                            td { "{srut}" }
                                            td { class: "{status_class}", "{status}" }
                                            td { "{time}" }
                                        }
                                    }}
                                }
                                {if records.is_empty() {
                                    rsx! { p { class: "empty-state", "Sin registros para esta fecha" } }
                                } else { rsx! {} }}
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! {},
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
    let pct = if total_days > 0 {
        present as f64 / total_days as f64 * 100.0
    } else {
        100.0
    };
    let pct_str = format!("{:.1}%", pct);
    let pct_class = if pct < 75.0 {
        "pct-danger"
    } else if pct < 85.0 {
        "pct-warning"
    } else {
        "pct-good"
    };

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
