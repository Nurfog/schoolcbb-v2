use dioxus::prelude::*;

use crate::api::client;

fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn EmployeePortalPage() -> Element {
    let mut tab = use_signal(|| "profile".to_string());

    let profile = use_resource(|| async move { client::fetch_json("/api/hr/me").await });

    let payroll = use_resource(|| async move { client::fetch_json("/api/hr/me/payroll").await });

    let attendance =
        use_resource(|| async move { client::fetch_json("/api/hr/me/attendance").await });

    let leave =
        use_resource(|| async move { client::fetch_json("/api/hr/me/leave-requests").await });

    let documents =
        use_resource(|| async move { client::fetch_json("/api/hr/me/documents").await });

    let tab_content = if tab() == "profile" {
        rsx! { ProfileSection { data: profile } }
    } else if tab() == "payroll" {
        rsx! { PayrollSection { data: payroll } }
    } else if tab() == "attendance" {
        rsx! { AttendanceSection { data: attendance } }
    } else if tab() == "leave" {
        rsx! { LeaveSection { data: leave } }
    } else if tab() == "docs" {
        rsx! { DocumentsSection { data: documents } }
    } else {
        rsx! {}
    };

    let _profile = use_resource(|| async move { client::fetch_json("/api/hr/me").await });

    let _payroll = use_resource(|| async move { client::fetch_json("/api/hr/me/payroll").await });

    let _attendance =
        use_resource(|| async move { client::fetch_json("/api/hr/me/attendance").await });

    let _leave =
        use_resource(|| async move { client::fetch_json("/api/hr/me/leave-requests").await });

    let _documents =
        use_resource(|| async move { client::fetch_json("/api/hr/me/documents").await });

    rsx! {
            div { class: "page-header",
                h1 { "Mi Portal" }
                p { "Autoconsulta de datos laborales" }
            }
            div { class: "tabs-container",
                div { class: "tabs-header",
                    button { class: if tab() == "profile" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("profile".to_string()), "Mi Perfil" }
                    button { class: if tab() == "payroll" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("payroll".to_string()), "Liquidaciones" }
                    button { class: if tab() == "attendance" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("attendance".to_string()), "Asistencia" }
                    button { class: if tab() == "leave" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("leave".to_string()), "Vacaciones y Permisos" }
                    button { class: if tab() == "docs" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("docs".to_string()), "Documentos" }
                }
                div { class: "tab-content",
                    {tab_content}
                }
        }
    }
}

#[component]
fn DocumentsSection(data: Resource<Result<serde_json::Value, String>>) -> Element {
    let mut doc_type = use_signal(|| "certificado".to_string());
    let mut file_name = use_signal(String::new);
    let mut uploading = use_signal(|| false);
    let docs = data;

    let do_upload = move |_| {
        if file_name().trim().is_empty() {
            return;
        }
        uploading.set(true);
        let payload = serde_json::json!({
            "doc_type": doc_type(),
            "file_name": file_name(),
        });
        spawn(async move {
            let _ = client::post_json("/api/hr/me/documents", &payload).await;
            uploading.set(false);
            file_name.set(String::new());
        });
    };

    rsx! {
        div { class: "data-table-container",
            div { class: "form-card",
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Tipo:" }
                        select { class: "form-input", value: "{doc_type}", onchange: move |e| doc_type.set(e.value()),
                            option { value: "certificado", "Certificado de Titulo" }
                            option { value: "capacitacion", "Capacitacion" }
                            option { value: "titulo", "Titulo Profesional" }
                            option { value: "postgrado", "Postgrado" }
                            option { value: "otro", "Otro" }
                        }
                    }
                    div { class: "form-group",
                        label { "Nombre del documento:" }
                        input { class: "form-input", value: "{file_name}", oninput: move |e| file_name.set(e.value()), placeholder: "Certificado UChile 2024" }
                    }
                    button { class: "btn btn-primary", disabled: uploading(), onclick: do_upload,
                        if uploading() { "Subiendo..." } else { "Subir Documento" }
                    }
                }
            }
            { match docs() {
                Some(Ok(json)) => {
                    let list = json["documents"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin documentos subidos" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|d| {
                            let dt = d["doc_type"].as_str().unwrap_or("").to_string();
                            let fn_ = d["file_name"].as_str().unwrap_or("").to_string();
                            let date = d["created_at"].as_str().unwrap_or("").to_string();
                            rsx! {
                                tr {
                                    td { span { class: "role-badge", "{dt}" } }
                                    td { "{fn_}" }
                                    td { "{date}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr { th { "Tipo" } th { "Documento" } th { "Subido" } } }
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando..." } },
            }}
        }
    }
}

#[component]
fn ProfileSection(data: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let emp = &json["employee"];
                    let name = format!("{} {}",
                        emp["first_name"].as_str().unwrap_or(""),
                        emp["last_name"].as_str().unwrap_or("")
                    );
                    let avatar = first_letter(&name);
                    let rut = emp["rut"].as_str().unwrap_or("-").to_string();
                    let email = emp["email"].as_str().unwrap_or("-").to_string();
                    let phone = emp["phone"].as_str().unwrap_or("-").to_string();
                    let position = emp["position"].as_str().unwrap_or("-").to_string();
                    let category = emp["category"].as_str().unwrap_or("-").to_string();
                    let hire_date = emp["hire_date"].as_str().unwrap_or("-").to_string();
                    let vac_days = emp["vacation_days_available"].as_f64().unwrap_or(0.0);
                    let sup = &json["supervisor"];
                    let supervisor_name = if sup.is_null() { "-".to_string() } else {
                        format!("{} {}",
                            sup["first_name"].as_str().unwrap_or(""),
                            sup["last_name"].as_str().unwrap_or(""))
                    };
                    rsx! {
                        div { class: "employee-summary-card",
                            div { class: "emp-avatar", "{avatar}" }
                            div { class: "emp-info",
                                h2 { "{name}" }
                                table { class: "info-table",
                                    tbody {
                                        tr { td { "RUT:" } td { "{rut}" } }
                                        tr { td { "Email:" } td { "{email}" } }
                                        tr { td { "Telefono:" } td { "{phone}" } }
                                        tr { td { "Cargo:" } td { "{position}" } }
                                        tr { td { "Categoria:" } td { "{category}" } }
                                        tr { td { "Fecha Contratacion:" } td { "{hire_date}" } }
                                        tr { td { "Dias Vacaciones:" } td { "{vac_days}" } }
                                        tr { td { "Supervisor:" } td { "{supervisor_name}" } }
                                    }
                                }
                            }
                        }
                        div { class: "contract-card",
                            h3 { "Contrato Actual" }
                            { match json["contract"].as_object() {
                                Some(contract) => {
                                    let ct = contract["contract_type"].as_str().unwrap_or("-");
                                    let sb = contract["salary_base"].as_f64().unwrap_or(0.0);
                                    let hrs = contract["weekly_hours"].as_i64().unwrap_or(0);
                                    let sd = contract["start_date"].as_str().unwrap_or("-");
                                    let ed = contract["end_date"].as_str().unwrap_or("Indefinido");
                                    rsx! {
                                        table { class: "info-table",
                                            tbody {
                                                tr { td { "Tipo:" } td { "{ct}" } }
                                                tr { td { "Sueldo Base:" } td { "${sb:.0}" } }
                                                tr { td { "Horas Semanales:" } td { "{hrs}" } }
                                                tr { td { "Inicio:" } td { "{sd}" } }
                                                tr { td { "Termino:" } td { "{ed}" } }
                                            }
                                        }
                                    }
                                }
                                None => rsx! { p { "Sin contrato activo" } }
                            }}
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}

#[component]
fn PayrollSection(data: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let list = json["payrolls"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin liquidaciones disponibles" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|p| {
                            let month = p["month"].as_i64().unwrap_or(0);
                            let year = p["year"].as_i64().unwrap_or(0);
                            let sb = p["salary_base"].as_f64().unwrap_or(0.0);
                            let grat = p["gratificacion"].as_f64().unwrap_or(0.0);
                            let afp = p["afp_discount"].as_f64().unwrap_or(0.0);
                            let health = p["health_discount"].as_f64().unwrap_or(0.0);
                            let net = p["net_salary"].as_f64().unwrap_or(0.0);
                            rsx! {
                                tr {
                                    td { "{month}/{year}" }
                                    td { "${sb:.0}" }
                                    td { "${grat:.0}" }
                                    td { "${afp:.0}" }
                                    td { "${health:.0}" }
                                    td { strong { "${net:.0}" } }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Periodo" }
                                    th { "Sueldo Base" }
                                    th { "Gratificacion" }
                                    th { "AFP" }
                                    th { "Salud" }
                                    th { "Liquido" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}

#[component]
fn AttendanceSection(data: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let list = json["attendance_logs"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin registros de asistencia este mes" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|l| {
                            let ts = l["timestamp"].as_str().unwrap_or("").to_string();
                            let etype = l["entry_type"].as_str().unwrap_or("").to_string();
                            let dev = l["device_id"].as_str().unwrap_or("-").to_string();
                            rsx! {
                                tr {
                                    td { "{ts}" }
                                    td { span { class: "role-badge", "{etype}" } }
                                    td { "{dev}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Fecha/Hora" }
                                    th { "Tipo" }
                                    th { "Dispositivo" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}

#[component]
fn LeaveSection(data: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let list = json["leave_requests"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin solicitudes de vacaciones o permisos" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|r| {
                            let lt = r["leave_type"].as_str().unwrap_or("").to_string();
                            let sd = r["start_date"].as_str().unwrap_or("").to_string();
                            let ed = r["end_date"].as_str().unwrap_or("").to_string();
                            let status = r["status"].as_str().unwrap_or("Pendiente").to_string();
                            rsx! {
                                tr {
                                    td { "{lt}" }
                                    td { "{sd}" }
                                    td { "{ed}" }
                                    td {
                                        if status == "Aprobado" { span { class: "status-active", "Aprobado" } }
                                        else if status == "Rechazado" { span { class: "status-inactive", "Rechazado" } }
                                        else { span { class: "status-pending", "Pendiente" } }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Tipo" }
                                    th { "Inicio" }
                                    th { "Termino" }
                                    th { "Estado" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
