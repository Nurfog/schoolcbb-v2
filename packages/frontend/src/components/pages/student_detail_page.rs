use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn StudentDetailPage(student_id: String) -> Element {
    let sid_report = student_id.clone();
    let sid_s1 = student_id.clone();
    let sid_s2 = student_id.clone();
    let sid_int = student_id.clone();
    let sid_fees = student_id.clone();

    let report = use_resource(move || {
        let sid = sid_report.clone();
        async move { client::fetch_student_report(&sid, 2025).await }
    });

    let grades_s1 = use_resource(move || {
        let sid = sid_s1.clone();
        async move { client::fetch_grades_student(&sid, 1, 2025).await }
    });

    let grades_s2 = use_resource(move || {
        let sid = sid_s2.clone();
        async move { client::fetch_grades_student(&sid, 2, 2025).await }
    });

    let interviews = use_resource(move || {
        let sid = sid_int.clone();
        async move { client::fetch_interviews_student(&sid).await }
    });

    let fees = use_resource(move || {
        let sid = sid_fees.clone();
        async move { client::fetch_fees_student(&sid).await }
    });

    rsx! {
        div { class: "page-header",
            h1 { "Vista 360° del Alumno" }
            p { "Información consolidada del estudiante" }
        }
        div { class: "dashboard-grid",
            { match report() {
                Some(Ok(data)) => rsx! {
                    StudentInfoCard { data: data.clone() }
                    PromotionCard { data: data.clone() }
                },
                _ => rsx! {},
            }}
            { match grades_s1() {
                Some(Ok(data)) => rsx! { GradesCard { title: "Semestre 1".to_string(), data: data.clone() } },
                _ => rsx! {},
            }}
            { match grades_s2() {
                Some(Ok(data)) => rsx! { GradesCard { title: "Semestre 2".to_string(), data: data.clone() } },
                _ => rsx! {},
            }}
            { match interviews() {
                Some(Ok(data)) => rsx! { InterviewCard { data: data.clone() } },
                _ => rsx! {},
            }}
            { match fees() {
                Some(Ok(data)) => rsx! { FinanceCard { data: data.clone() } },
                _ => rsx! {},
            }}
        }
    }
}

#[component]
fn StudentInfoCard(data: Value) -> Element {
    let name = data["student_name"].as_str().unwrap_or("-").to_string();
    let avatar = first_letter(&name);

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Información Personal" }
            }
            div { class: "widget-card-body",
                div { class: "student-info",
                    div { class: "student-avatar-lg", "{avatar}" }
                    div { class: "student-details",
                        p { strong { "Nombre: " } "{name}" }
                    }
                }
            }
        }
    }
}

#[component]
fn PromotionCard(data: Value) -> Element {
    let promotion = data["final_promotion"]
        .as_str()
        .unwrap_or("Pendiente")
        .to_string();
    let pclass = if promotion == "Promovido" {
        "status-active"
    } else if promotion == "Reprobado" {
        "status-inactive"
    } else {
        "pct-warning"
    };

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Estado de Promoción" }
            }
            div { class: "widget-card-body",
                div { class: "promotion-status",
                    span { class: "{pclass}", style: "font-size: 18px; padding: 8px 20px;", "{promotion}" }
                }
            }
        }
    }
}

#[component]
fn GradesCard(title: String, data: Value) -> Element {
    let subjects = data["subjects"].as_array().cloned().unwrap_or_default();
    let global_avg = data["global_average"].as_f64().unwrap_or(0.0);
    let global_str = format!("{:.1}", global_avg);

    let rows: Vec<_> = subjects
        .iter()
        .map(|s| {
            let name = s["subject"].as_str().unwrap_or("-").to_string();
            let avg = s["average"].as_f64().unwrap_or(0.0);
            let avg_str = format!("{:.1}", avg);
            let avg_class = if avg < 4.0 { "pct-danger" } else { "pct-good" }.to_string();
            let min_str = format!("{:.1}", s["min_grade"].as_f64().unwrap_or(0.0));
            let max_str = format!("{:.1}", s["max_grade"].as_f64().unwrap_or(0.0));
            (name, avg_str, avg_class, min_str, max_str)
        })
        .collect();

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "{title}" }
                span { "Promedio: {global_str}" }
            }
            div { class: "widget-card-body",
                table { class: "data-table", style: "font-size: 13px;",
                    thead {
                        tr {
                            th { "Asignatura" }
                            th { "Promedio" }
                            th { "Notas" }
                            th { "Mín" }
                            th { "Máx" }
                        }
                    }
                    tbody {
                        for (name, avg_str, avg_class, min_str, max_str) in &rows {
                            tr {
                                td { "{name}" }
                                td { class: "{avg_class}", "{avg_str}" }
                                td { "0" }
                                td { "{min_str}" }
                                td { "{max_str}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InterviewCard(data: Value) -> Element {
    let list = data["interviews"].as_array().cloned().unwrap_or_default();

    let rows: Vec<(String, String)> = list
        .iter()
        .map(|e| {
            (
                e["date"].as_str().unwrap_or("").to_string(),
                e["reason"].as_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    if rows.is_empty() {
        return rsx! {
            div { class: "widget-card",
                div { class: "widget-card-header",
                    h3 { "Bitácora de Entrevistas" }
                }
                div { class: "widget-card-body",
                    p { class: "empty-state", "Sin registros de entrevistas" }
                }
            }
        };
    }

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Bitácora de Entrevistas" }
                span { "Últimas entrevistas" }
            }
            div { class: "widget-card-body",
                table { class: "data-table", style: "font-size: 13px;",
                    thead {
                        tr {
                            th { "Fecha" }
                            th { "Motivo" }
                        }
                    }
                    tbody {
                        for (date, reason) in &rows {
                            tr {
                                td { "{date}" }
                                td { "{reason}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FinanceCard(data: Value) -> Element {
    let fees = data["fees"].as_array().cloned().unwrap_or_default();
    let total_pending: f64 = fees
        .iter()
        .filter(|f| !f["paid"].as_bool().unwrap_or(true))
        .map(|f| f["amount"].as_f64().unwrap_or(0.0))
        .sum();
    let debt_str = format!("${:.0}", total_pending);

    let rows: Vec<(String, String, String, bool)> = fees
        .iter()
        .map(|f| {
            let desc = f["description"].as_str().unwrap_or("").to_string();
            let amount = f["amount"].as_f64().unwrap_or(0.0);
            (
                desc,
                format!("${:.0}", amount),
                f["due_date"].as_str().unwrap_or("").to_string(),
                f["paid"].as_bool().unwrap_or(false),
            )
        })
        .collect();

    if rows.is_empty() {
        return rsx! {
            div { class: "widget-card",
                div { class: "widget-card-header",
                    h3 { "Situación Financiera" }
                    span { "Deuda: {debt_str}" }
                }
                div { class: "widget-card-body",
                    p { class: "empty-state", "Sin movimientos financieros" }
                }
            }
        };
    }

    rsx! {
        div { class: "widget-card",
            div { class: "widget-card-header",
                h3 { "Situación Financiera" }
                span { "Deuda: {debt_str}" }
            }
            div { class: "widget-card-body",
                table { class: "data-table", style: "font-size: 13px;",
                    thead {
                        tr {
                            th { "Concepto" }
                            th { "Monto" }
                            th { "Vencimiento" }
                            th { "Estado" }
                        }
                    }
                    tbody {
                        for (desc, amt_str, due, paid) in &rows {
                            tr {
                                td { "{desc}" }
                                td { "{amt_str}" }
                                td { "{due}" }
                                td {
                                    if *paid {
                                        span { class: "status-active", "Pagado" }
                                    } else {
                                        span { class: "status-inactive", "Pendiente" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
