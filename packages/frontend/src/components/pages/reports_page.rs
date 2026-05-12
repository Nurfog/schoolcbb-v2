use dioxus::prelude::*;

use crate::api::client;

fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

fn current_month() -> u32 {
    js_sys::Date::new_0().get_month() + 1
}

#[derive(PartialEq, Clone)]
enum ReportTab {
    Individual,
    Course,
    Sige,
}

#[component]
pub fn ReportsPage() -> Element {
    let mut active_tab = use_signal(|| ReportTab::Individual);

    rsx! {
        div { class: "page-header",
            h1 { "Reportes" }
            p { "Certificados, concentraciones de notas y actas oficiales" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == ReportTab::Individual { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Individual),
                "Individuales"
            }
            button {
                class: if active_tab() == ReportTab::Course { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Course),
                "Por Curso"
            }
            button {
                class: if active_tab() == ReportTab::Sige { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(ReportTab::Sige),
                "Exportaciones SIGE"
            }
        }
        div { class: "tab-content",
            {
                match active_tab() {
                    ReportTab::Individual => rsx! { IndividualReports {} },
                    ReportTab::Course => rsx! { CourseReports {} },
                    ReportTab::Sige => rsx! { SigeReports {} },
                }
            }
        }
    }
}

#[component]
fn IndividualReports() -> Element {
    let mut search_query = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut selected_year = use_signal(current_year);
    let mut report_type = use_signal(|| "certificate".to_string());
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = search_query();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let on_search = move |evt: Event<FormData>| {
        search_query.set(evt.value());
    };

    let clear_student = move |_| {
        selected_student.set(None);
        result.set(None);
    };

    let generate_report = move |_| {
        if let Some(ref student) = selected_student() {
            let sid = student["id"].as_str().unwrap_or("").to_string();
            let y = selected_year();
            let rt = report_type();
            loading.set(true);
            result.set(None);
            spawn(async move {
                let res = match rt.as_str() {
                    "certificate" => client::fetch_student_certificate(&sid).await,
                    "concentration" => client::fetch_student_concentration(&sid, y).await,
                    _ => Err("Tipo no válido".to_string()),
                };
                loading.set(false);
                result.set(Some(res));
            });
        }
    };

    let mut select_student_fn = move |s: serde_json::Value| {
        selected_student.set(Some(s));
        search_query.set(String::new());
    };

    let search_data: Option<Vec<serde_json::Value>> = match search_results() {
        Some(Ok(j)) => {
            let list = j["students"].as_array().cloned().unwrap_or_default();
            if !list.is_empty() && search_query().len() >= 2 {
                Some(list)
            } else {
                None
            }
        }
        _ => None,
    };

    rsx! {
        div { class: "report-section",
            div { class: "filter-group",
                label { "Tipo de Reporte:" }
                select { value: "{report_type}", onchange: move |evt| report_type.set(evt.value()),
                    option { value: "certificate", "Certificado Alumno Regular" }
                    option { value: "concentration", "Concentración de Notas" }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select {
                    value: "{selected_year}",
                    onchange: move |evt| { if let Ok(y) = evt.value().parse() { selected_year.set(y); } },
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            div { class: "student-selector",
                label { "Estudiante:" }
                {
                    match selected_student() {
                        Some(ref s) => {
                            let sname = format!("{} {}",
                                s["first_name"].as_str().unwrap_or(""),
                                s["last_name"].as_str().unwrap_or("")
                            );
                            let srut = s["rut"].as_str().unwrap_or("").to_string();
                            rsx! {
                                div { class: "selected-student",
                                    span { "{sname} ({srut})" }
                                    button { class: "btn-icon", "aria-label": "Limpiar seleccion", onclick: clear_student, "✕" }
                                }
                            }
                        }
                        None => rsx! {
                            input {
                                class: "search-input",
                                value: "{search_query}",
                                oninput: on_search,
                                placeholder: "Buscar estudiante por nombre o RUT..."
                            }
                        }
                    }
                }
                {
                    match search_data {
                        Some(ref list) => {
                            let search_items: Vec<Element> = list.iter().map(|s| {
                                let sid = s["id"].as_str().unwrap_or("").to_string();
                                let sname = format!("{} {}",
                                    s["first_name"].as_str().unwrap_or(""),
                                    s["last_name"].as_str().unwrap_or("")
                                );
                                let srut = s["rut"].as_str().unwrap_or("").to_string();
                                rsx! {
                                    div {
                                        class: "search-result-item",
                                        onclick: move |_| {
                                            let sv = serde_json::json!({
                                                "id": sid.clone(),
                                                "first_name": sname.clone(),
                                                "rut": srut.clone(),
                                            });
                                            select_student_fn(sv);
                                        },
                                        span { "{sname}" }
                                        span { class: "result-rut", "{srut}" }
                                    }
                                }
                            }).collect();
                            rsx! { div { class: "search-results", { search_items.into_iter() } } }
                        }
                        None => rsx! {},
                    }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: selected_student().is_none() || loading(),
                    onclick: generate_report,
                    if loading() { "Generando..." } else { "Generar Reporte" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => {
                        match report_type().as_str() {
                            "certificate" => rsx! { CertificateResult { data: j } },
                            "concentration" => rsx! { ConcentrationResult { data: j } },
                            _ => rsx! {},
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
fn CertificateResult(data: serde_json::Value) -> Element {
    let name = data["certificate"]["student_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let rut = data["certificate"]["rut"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let grade_level = data["certificate"]["grade_level"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let section = data["certificate"]["section"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let year = data["certificate"]["year"].as_i64().unwrap_or(0);
    let status = data["certificate"]["enrollment_status"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let issued_at = data["certificate"]["issued_at"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let issuer = data["certificate"]["issuer_name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "report-result certificate",
            div { class: "certificate-header",
                h2 { "Certificado Alumno Regular" }
            }
            div { class: "certificate-body",
                p { "El/la estudiante {name}, Rut {rut}, se encuentra matriculado(a) en {grade_level} {section}, durante el año académico {year}." }
                p { "Estado: {status}" }
                hr {}
                p { class: "certificate-meta", "Emitido el {issued_at} por {issuer}" }
            }
        }
    }
}

#[component]
fn ConcentrationResult(data: serde_json::Value) -> Element {
    let student_name = data["concentration"]["student_name"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let rut = data["concentration"]["rut"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let year = data["concentration"]["year"].as_i64().unwrap_or(0);
    let final_avg = data["concentration"]["final_average"]
        .as_f64()
        .unwrap_or(0.0);
    let final_prom = data["concentration"]["final_promotion"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let semesters = data["concentration"]["semesters"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let semester_cards: Vec<(i64, f64, Vec<(String, i64, String, f64, f64, f64)>)> = semesters
        .iter()
        .map(|sem| {
            let sem_num = sem["semester"].as_i64().unwrap_or(0);
            let global = sem["global_average"].as_f64().unwrap_or(0.0);
            let subjects = sem["subjects"].as_array().cloned().unwrap_or_default();
            let subject_rows: Vec<(String, i64, String, f64, f64, f64)> = subjects
                .iter()
                .map(|s| {
                    let sname = s["subject_name"].as_str().unwrap_or("-").to_string();
                    let gcount = s["grades_count"].as_i64().unwrap_or(0);
                    let avg = s["average"].as_f64().unwrap_or(0.0);
                    let min_g = s["min_grade"].as_f64().unwrap_or(0.0);
                    let max_g = s["max_grade"].as_f64().unwrap_or(0.0);
                    let avg_class = if avg >= 4.0 {
                        "grade-good".to_string()
                    } else {
                        "grade-bad".to_string()
                    };
                    (sname, gcount, avg_class, avg, min_g, max_g)
                })
                .collect();
            (sem_num, global, subject_rows)
        })
        .collect();

    rsx! {
        div { class: "report-result",
            h2 { "Concentración de Notas" }
            p { "{student_name} - {rut} - Año {year}" }
            {
                semester_cards.iter().map(|(sem_num, global, subject_rows)| {
                    rsx! {
                        div { class: "widget-card", style: "margin-top: 12px;",
                            div { class: "widget-card-header",
                                h3 { "Semestre {sem_num}" }
                                span { "Promedio: {global:.1}" }
                            }
                            table { class: "data-table",
                                thead { tr {
                                    th { "Asignatura" }
                                    th { "Notas" }
                                    th { "Promedio" }
                                    th { "Mín" }
                                    th { "Máx" }
                                }}
                                tbody { for (sname, gcount, avg_class, avg, min_g, max_g) in subject_rows {
                                    tr {
                                        td { "{sname}" }
                                        td { "{gcount}" }
                                        td { class: "{avg_class}", "{avg:.1}" }
                                        td { "{min_g:.1}" }
                                        td { "{max_g:.1}" }
                                    }
                                }}
                            }
                        }
                    }
                })
            }
            div { class: "promotion-banner",
                span { class: "promotion-text", "Promedio Final: {final_avg:.1} - {final_prom}" }
            }
        }
    }
}

#[component]
fn CourseReports() -> Element {
    let mut selected_year = use_signal(current_year);
    let mut selected_course = use_signal(|| None::<serde_json::Value>);
    let mut search_course = use_signal(String::new);
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);
    let courses = use_resource(move || {
        let q = search_course();
        async move {
            client::fetch_json(&format!("/api/courses?search={}", q.replace(' ', "%20"))).await
        }
    });

    let generate_record = move |_| {
        if let Some(ref course) = selected_course() {
            let cid = course["id"].as_str().unwrap_or("").to_string();
            let y = selected_year();
            loading.set(true);
            result.set(None);
            spawn(async move {
                let res = client::fetch_final_record(&cid, y).await;
                loading.set(false);
                result.set(Some(res));
            });
        }
    };

    let course_elements: Vec<Element> = match courses() {
        Some(Ok(j)) => {
            let list = j["courses"].as_array().cloned().unwrap_or_default();
            list.iter()
                .map(|c| {
                    let cid = c["id"].as_str().unwrap_or("").to_string();
                    let cname = c["name"].as_str().unwrap_or("").to_string();
                    let level = c["grade_level"].as_str().unwrap_or("").to_string();
                    let section = c["section"].as_str().unwrap_or("").to_string();
                    let cinfo = format!("{} - {}", level, section);
                    let mut selected = selected_course.clone();
                    let mut search = search_course.clone();
                    rsx! {
                        div {
                            class: "search-result-item",
                            onclick: move |_| {
                                selected.set(Some(serde_json::json!({
                                    "id": cid.clone(),
                                    "name": cname.clone(),
                                })));
                                search.set(String::new());
                            },
                            span { "{cname}" }
                            span { class: "result-rut", "{cinfo}" }
                        }
                    }
                })
                .collect()
        }
        _ => vec![],
    };

    rsx! {
        div { class: "report-section",
            div { class: "filter-group",
                label { "Año:" }
                select {
                    value: "{selected_year}",
                    onchange: move |evt| { if let Ok(y) = evt.value().parse() { selected_year.set(y); } },
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            div { class: "student-selector",
                label { "Curso:" }
                {
                    match selected_course() {
                        Some(ref c) => {
                            let cname = c["name"].as_str().unwrap_or("").to_string();
                            rsx! {
                                div { class: "selected-student",
                                    span { "{cname}" }
                                    button { class: "btn-icon", "aria-label": "Cerrar", onclick: move |_| { selected_course.set(None); result.set(None); }, "✕" }
                                }
                            }
                        }
                        None => rsx! {
                            input {
                                class: "search-input",
                                value: "{search_course}",
                                oninput: move |evt| search_course.set(evt.value()),
                                placeholder: "Buscar curso..."
                            }
                        }
                    }
                }
                {
                    if !course_elements.is_empty() {
                        rsx! { div { class: "search-results", { course_elements.into_iter() } } }
                    } else {
                        rsx! {}
                    }
                }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: selected_course().is_none() || loading(),
                    onclick: generate_record,
                    if loading() { "Generando..." } else { "Generar Acta Final" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => rsx! { FinalRecordResult { data: j } },
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! {},
                }
            }
        }
    }
}

#[component]
fn FinalRecordResult(data: serde_json::Value) -> Element {
    let record = &data["final_record"];
    let course_name = record["course_name"].as_str().unwrap_or("").to_string();
    let year = record["year"].as_i64().unwrap_or(0);
    let promoted = record["summary"]["promoted"].as_i64().unwrap_or(0);
    let failed = record["summary"]["failed"].as_i64().unwrap_or(0);
    let rate = record["summary"]["average_promotion_rate"]
        .as_f64()
        .unwrap_or(0.0);
    let students = record["students"].as_array().cloned().unwrap_or_default();

    let student_rows: Vec<(String, String, f64, String, String)> = students
        .iter()
        .map(|s| {
            let name = s["student_name"].as_str().unwrap_or("-").to_string();
            let rut = s["rut"].as_str().unwrap_or("-").to_string();
            let avg = s["final_average"].as_f64().unwrap_or(0.0);
            let prom = s["promotion"].as_str().unwrap_or("").to_string();
            let prom_class = if prom == "Reprobado" {
                "grade-bad".to_string()
            } else {
                "grade-good".to_string()
            };
            (name, rut, avg, prom, prom_class)
        })
        .collect();

    rsx! {
        div { class: "report-result",
            div { class: "summary-cards",
                div { class: "summary-card",
                    span { class: "summary-value", "{course_name}" }
                    span { class: "summary-label", "Curso - {year}" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{promoted}" }
                    span { class: "summary-label", "Promovidos" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{failed}" }
                    span { class: "summary-label", "Reprobados" }
                }
                div { class: "summary-card",
                    span { class: "summary-value", "{rate:.1}%" }
                    span { class: "summary-label", "Tasa de Promoción" }
                }
            }
            table { class: "data-table", style: "margin-top: 16px;",
                thead { tr {
                    th { "Estudiante" }
                    th { "RUT" }
                    th { "Prom. Final" }
                    th { "Promoción" }
                }}
                tbody { for (name, rut, avg, prom, prom_class) in &student_rows {
                    tr {
                        td { class: "cell-name", "{name}" }
                        td { "{rut}" }
                        td { "{avg:.1}" }
                        td { class: "{prom_class}", "{prom}" }
                    }
                }}
            }
        }
    }
}

#[component]
fn SigeReports() -> Element {
    let mut selected_year = use_signal(current_year);
    let mut selected_month = use_signal(current_month);
    let mut export_type = use_signal(|| "students".to_string());
    let mut result = use_signal(|| None::<Result<serde_json::Value, String>>);
    let mut loading = use_signal(|| false);

    let generate_export = move |_| {
        loading.set(true);
        result.set(None);
        let et = export_type();
        let y = selected_year();
        let m = selected_month();
        spawn(async move {
            let res = match et.as_str() {
                "students" => client::fetch_sige_students().await,
                "attendance" => client::fetch_sige_attendance(y, m).await,
                _ => Err("Tipo no válido".to_string()),
            };
            loading.set(false);
            result.set(Some(res));
        });
    };

    rsx! {
        div { class: "report-section",
            div { class: "filter-group",
                label { "Exportar:" }
                select { value: "{export_type}", onchange: move |evt| export_type.set(evt.value()),
                    option { value: "students", "Datos de Estudiantes (SIGE)" }
                    option { value: "attendance", "Asistencia Mensual (SIGE)" }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select {
                    value: "{selected_year}",
                    onchange: move |evt| { if let Ok(y) = evt.value().parse() { selected_year.set(y); } },
                    option { value: "2026", "2026" }
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
            {
                if export_type() == "attendance" {
                    rsx! {
                        div { class: "filter-group",
                            label { "Mes:" }
                            select {
                                value: "{selected_month}",
                                onchange: move |evt| { if let Ok(m) = evt.value().parse() { selected_month.set(m); } },
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
                } else { rsx! {} }
            }
            div { class: "form-actions",
                button {
                    class: "btn btn-primary",
                    disabled: loading(),
                    onclick: generate_export,
                    if loading() { "Exportando..." } else { "Exportar" }
                }
            }
            {
                match result() {
                    Some(Ok(j)) => {
                        let total = j["total"].as_i64().unwrap_or(0);
                        let rows = j["rows"].as_array().cloned().unwrap_or_default();
                        rsx! {
                            div { class: "report-result",
                                p { "Total registros: {total}" }
                                table { class: "data-table",
                                    thead {
                                        tr {
                                            { rows.first().map(|first| {
                                                rsx! {
                                                    { first.as_object().map(|obj| {
                                                        rsx! { { obj.keys().map(|k| rsx! { th { "{k}" } }) } }
                                                    })}
                                                }
                                            })}
                                        }
                                    }
                                    tbody {
                                        for row in &rows {
                                            tr {
                                                { row.as_object().map(|obj| {
                                                    rsx! { { obj.values().map(|v| {
                                                        let val = match v {
                                                            serde_json::Value::String(s) => s.clone(),
                                                            serde_json::Value::Number(n) => n.to_string(),
                                                            serde_json::Value::Bool(b) => b.to_string(),
                                                            _ => "".to_string(),
                                                        };
                                                        rsx! { td { "{val}" } }
                                                    }) } }
                                                })}
                                            }
                                        }
                                    }
                                }
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
