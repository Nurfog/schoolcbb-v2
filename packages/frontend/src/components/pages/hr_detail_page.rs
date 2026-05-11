use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn HrDetailPage(employee_id: String) -> Element {
    let eid_emp = employee_id.clone();
    let eid_att = employee_id.clone();

    let employee = use_resource(move || {
        let eid = eid_emp.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}", eid)).await }
    });

    let attendance = use_resource(move || {
        let eid = eid_att.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/attendance", eid)).await }
    });

    let mut tab = use_signal(|| "info".to_string());

    rsx! {
        div { class: "page-header",
            h1 { "Ficha del Empleado" }
            p { "Informacion detallada, asistencia y permisos" }
        }
        { match employee() {
            Some(Ok(data)) => {
                let emp = &data["employee"];
                let name = format!("{} {}",
                    emp["first_name"].as_str().unwrap_or(""),
                    emp["last_name"].as_str().unwrap_or("")
                );
                let category = emp["category"].as_str().unwrap_or("-");
                let position = emp["position"].as_str().unwrap_or("-");
                let vac_days = emp["vacation_days_available"].as_f64().unwrap_or(0.0);
                let rut = emp["rut"].as_str().unwrap_or("");
                rsx! {
                    div { class: "employee-summary-card",
                        div { class: "emp-avatar", "{name.chars().next().unwrap_or('?')}" }
                        div { class: "emp-info",
                            h2 { "{name}" }
                            p { "RUT: {rut} | Categoria: {category} | Cargo: {position}" }
                            p { "Dias de vacaciones disponibles: {vac_days}" }
                        }
                    }
                    div { class: "tabs-container",
                        div { class: "tabs-header",
                            button { class: if tab() == "info" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("info".to_string()), "Contratos" }
                            button { class: if tab() == "attendance" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("attendance".to_string()), "Asistencia" }
                            button { class: if tab() == "leave" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("leave".to_string()), "Vacaciones y Permisos" }
                        }
                        div { class: "tab-content",
                            match tab() {
                            s if s == "info" => rsx! {
                                ContractsSection { employee_id: employee_id.clone() }
                            },
                            s if s == "attendance" => rsx! {
                                AttendanceSection { data: attendance, employee_id: employee_id.clone() }
                            },
                            s if s == "leave" => rsx! {
                                LeaveSection { employee_id: employee_id.clone() }
                            },
                            _ => rsx! {}
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
            None => rsx! { div { class: "loading-spinner", "Cargando..." } },
        }}
    }
}

#[component]
fn ContractsSection(employee_id: String) -> Element {
    let eid = employee_id.clone();
    let data = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}", id)).await }
    });

    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let contracts = json["contracts"].as_array().cloned().unwrap_or_default();
                    if contracts.is_empty() {
                        rsx! { div { class: "empty-state", "Sin contratos registrados" } }
                    } else {
                        let rows: Vec<Element> = contracts.iter().map(|c| {
                            let ct = c["contract_type"].as_str().unwrap_or("").to_string();
                            let sal = c["salary_base"].as_f64().unwrap_or(0.0);
                            let hrs = c["weekly_hours"].as_i64().unwrap_or(0);
                            let start = c["start_date"].as_str().unwrap_or("").to_string();
                            let end = c["end_date"].as_str().unwrap_or("-").to_string();
                            let karin = c["ley_karin_signed"].as_bool().unwrap_or(false);
                            let active = c["active"].as_bool().unwrap_or(false);
                            rsx! {
                                tr {
                                    td { "{ct}" }
                                    td { "${sal:.0}" }
                                    td { "{hrs}" }
                                    td { "{start}" }
                                    td { "{end}" }
                                    td { if karin { "Firmado" } else { "-" } }
                                    td {
                                        if active { span { class: "status-active", "Activo" } }
                                        else { span { class: "status-inactive", "Inactivo" } }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Tipo" }
                                    th { "Salario Base" }
                                    th { "Horas Semanales" }
                                    th { "Inicio" }
                                    th { "Termino" }
                                    th { "Ley Karin" }
                                    th { "Estado" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando contratos..." } },
            }
        }
    }
}

#[component]
fn AttendanceSection(data: Resource<Result<Value, String>>, employee_id: String) -> Element {
    let eid = employee_id.clone();
    let mut show_sync = use_signal(|| false);
    let mut sync_timestamp = use_signal(String::new);
    let mut sync_entry_type = use_signal(|| "Entrada".to_string());
    let mut syncing = use_signal(|| false);
    let mut attendance = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/attendance", id)).await }
    });

    let do_sync = move |_| {
        if sync_timestamp().trim().is_empty() {
            return;
        }
        syncing.set(true);
        let payload = serde_json::json!({
            "employee_id": employee_id.clone(),
            "timestamp": sync_timestamp(),
            "entry_type": sync_entry_type(),
        });
        spawn(async move {
            let _ = client::post_json("/api/hr/attendance/sync", &payload).await;
            syncing.set(false);
            show_sync.set(false);
            sync_timestamp.set(String::new());
            attendance.restart();
        });
    };

    rsx! {
        div { class: "attendance-section",
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| show_sync.set(!show_sync()),
                    if show_sync() { "Cancelar" } else { "Registrar Marcacion" }
                }
            }
            { if show_sync() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Fecha y Hora:" }
                                input { class: "form-input", value: "{sync_timestamp}", oninput: move |e| sync_timestamp.set(e.value()), placeholder: "2026-05-11T09:00:00", type: "datetime-local" }
                            }
                            div { class: "form-group",
                                label { "Tipo:" }
                                select { class: "form-input", value: "{sync_entry_type}", onchange: move |e| sync_entry_type.set(e.value()),
                                    option { value: "Entrada", "Entrada" }
                                    option { value: "Salida Colacion", "Salida Colacion" }
                                    option { value: "Retorno Colacion", "Retorno Colacion" }
                                    option { value: "Salida", "Salida" }
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: syncing(), onclick: do_sync,
                                if syncing() { "Registrando..." } else { "Registrar" }
                            }
                        }
                    }
                }
            } else { rsx! {} }}
            div { class: "data-table-container",
                match attendance() {
                    Some(Ok(json)) => {
                        let logs = json["attendance_logs"].as_array().cloned().unwrap_or_default();
                        if logs.is_empty() {
                            rsx! { div { class: "empty-state", "Sin registros de asistencia" } }
                        } else {
                            let rows: Vec<Element> = logs.iter().map(|l| {
                                let ts = l["timestamp"].as_str().unwrap_or("").to_string();
                                let etype = l["entry_type"].as_str().unwrap_or("").to_string();
                                let dev = l["device_id"].as_str().unwrap_or("-").to_string();
                                let src = l["source"].as_str().unwrap_or("-").to_string();
                                rsx! {
                                    tr {
                                        td { "{ts}" }
                                        td { span { class: "role-badge", "{etype}" } }
                                        td { "{dev}" }
                                        td { "{src}" }
                                    }
                                }
                            }).collect();
                            rsx! {
                                table { class: "data-table",
                                    thead { tr {
                                        th { "Fecha/Hora" }
                                        th { "Tipo" }
                                        th { "Dispositivo" }
                                        th { "Origen" }
                                    }}
                                    tbody { { rows.into_iter() } }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "empty-state", "Cargando asistencia..." } },
                }
            }
        }
    }
}

#[component]
fn LeaveSection(employee_id: String) -> Element {
    let mut show_request = use_signal(|| false);
    let mut leave_type = use_signal(|| "Vacaciones".to_string());
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut reason = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let eid = employee_id.clone();
    let mut leave_requests = use_resource(move || {
        let id = eid.clone();
        async move { client::fetch_json(&format!("/api/hr/employees/{}/leave-requests", id)).await }
    });

    let do_request = move |_| {
        if start_date().trim().is_empty() || end_date().trim().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "employee_id": employee_id.clone(),
            "leave_type": leave_type(),
            "start_date": start_date(),
            "end_date": end_date(),
            "reason": reason(),
        });
        let eid2 = employee_id.clone();
        spawn(async move {
            let _ = client::post_json(
                &format!("/api/hr/employees/{}/leave-requests", eid2),
                &payload,
            )
            .await;
            saving.set(false);
            show_request.set(false);
            start_date.set(String::new());
            end_date.set(String::new());
            reason.set(String::new());
            leave_requests.restart();
        });
    };

    rsx! {
        div { class: "leave-section",
            div { class: "page-toolbar",
                button { class: "btn btn-primary", onclick: move |_| show_request.set(!show_request()),
                    if show_request() { "Cancelar" } else { "Nueva Solicitud" }
                }
            }
            { if show_request() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Tipo:" }
                                select { class: "form-input", value: "{leave_type}", onchange: move |e| leave_type.set(e.value()),
                                    option { value: "Vacaciones", "Vacaciones" }
                                    option { value: "Licencia Medica", "Licencia Medica" }
                                    option { value: "Permiso Personal", "Permiso Personal" }
                                    option { value: "Capacitacion", "Capacitacion" }
                                    option { value: "Otro", "Otro" }
                                }
                            }
                            div { class: "form-group",
                                label { "Fecha Inicio:" }
                                input { class: "form-input", value: "{start_date}", oninput: move |e| start_date.set(e.value()), type: "date" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Fecha Termino:" }
                                input { class: "form-input", value: "{end_date}", oninput: move |e| end_date.set(e.value()), type: "date" }
                            }
                            div { class: "form-group",
                                label { "Motivo (opcional):" }
                                input { class: "form-input", value: "{reason}", oninput: move |e| reason.set(e.value()) }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_request,
                                if saving() { "Enviando..." } else { "Solicitar" }
                            }
                        }
                    }
                }
            } else { rsx! {} }}
            div { class: "data-table-container",
                match leave_requests() {
                    Some(Ok(json)) => {
                        let requests = json["leave_requests"].as_array().cloned().unwrap_or_default();
                        if requests.is_empty() {
                            rsx! { div { class: "empty-state", "Sin solicitudes" } }
                        } else {
                            let rows: Vec<Element> = requests.iter().map(|r| {
                                let lt = r["leave_type"].as_str().unwrap_or("").to_string();
                                let sd = r["start_date"].as_str().unwrap_or("").to_string();
                                let ed = r["end_date"].as_str().unwrap_or("").to_string();
                                let status = r["status"].as_str().unwrap_or("Pendiente").to_string();
                                let approved = r["approved_by"].as_str().map(|s| if s.is_empty() { "-" } else { s }).unwrap_or("-").to_string();
                                rsx! {
                                    tr {
                                        td { "{lt}" }
                                        td { "{sd}" }
                                        td { "{ed}" }
                                        td { "-" }
                                        td {
                                            if status == "Aprobado" { span { class: "status-active", "Aprobado" } }
                                            else if status == "Rechazado" { span { class: "status-inactive", "Rechazado" } }
                                            else { span { class: "status-pending", "Pendiente" } }
                                        }
                                        td { "{approved}" }
                                    }
                                }
                            }).collect();
                            rsx! {
                                table { class: "data-table",
                                    thead { tr {
                                        th { "Tipo" }
                                        th { "Inicio" }
                                        th { "Termino" }
                                        th { "Dias" }
                                        th { "Estado" }
                                        th { "Aprobado por" }
                                    }}
                                    tbody { { rows.into_iter() } }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "empty-state", "Cargando solicitudes..." } },
                }
            }
        }
    }
}
