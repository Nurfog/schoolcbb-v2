use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;
use crate::components::widgets::kpi_card::KpiCard;
use crate::components::widgets::simple_chart::{BarChart, DoughnutChart};

#[component]
pub fn RootDashboard() -> Element {
    let summary = use_resource(|| async move { client::fetch_json("/api/admin/stats/summary").await });
    let monthly = use_resource(|| async move { client::fetch_json("/api/admin/stats/monthly").await });
    let distribution = use_resource(|| async move { client::fetch_json("/api/admin/stats/license-distribution").await });
    let mut corps_signal = use_resource(|| async move { client::admin_list_corporations().await });
    let activity = use_resource(|| async move { client::fetch_json("/api/admin/activity-log?limit=20").await });

    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut corp_name = use_signal(String::new);
    let mut corp_rut = use_signal(String::new);
    let mut rep_name = use_signal(String::new);
    let mut rep_rut = use_signal(String::new);
    let mut rep_email = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let corps_data = corps_signal();

    let open_new = move |_| {
        edit_id.set(None);
        corp_name.set(String::new());
        corp_rut.set(String::new());
        rep_name.set(String::new());
        rep_rut.set(String::new());
        rep_email.set(String::new());
        show_form.set(true);
    };

    let open_edit = move |c: Value| {
        edit_id.set(c["id"].as_str().map(|s| s.to_string()));
        corp_name.set(c["name"].as_str().unwrap_or("").to_string());
        corp_rut.set(c["rut"].as_str().unwrap_or("").to_string());
        rep_name.set(c["legal_representative_name"].as_str().unwrap_or("").to_string());
        rep_rut.set(c["legal_representative_rut"].as_str().unwrap_or("").to_string());
        rep_email.set(c["legal_representative_email"].as_str().unwrap_or("").to_string());
        show_form.set(true);
    };

    let do_save = move |_| {
        let payload = json!({
            "name": corp_name(),
            "rut": corp_rut(),
            "legal_representative_name": rep_name(),
            "legal_representative_rut": rep_rut(),
            "legal_representative_email": rep_email(),
        });
        saving.set(true);
        let eid = edit_id();
        spawn(async move {
            if let Some(id) = eid {
                let _ = client::admin_update_corporation(&id, &payload).await;
            } else {
                let _ = client::admin_create_corporation(&payload).await;
            }
            saving.set(false);
            show_form.set(false);
            corps_signal.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Panel de Administración Root" }
            p { "Gestión global de corporaciones, licencias y monitoreo de la plataforma" }
        }
        KpiSection { summary_data: summary() }
        ChartsRow { monthly_data: monthly(), dist_data: distribution() }
        div { class: "section-card",
            div { class: "section-header",
                h3 { "Corporaciones" }
                button { class: "btn btn-sm btn-primary", onclick: open_new, "Nueva Corporación" }
            }
            if show_form() {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Nombre:" }
                            input { class: "form-input", value: "{corp_name}", oninput: move |e| corp_name.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "RUT:" }
                            input { class: "form-input", value: "{corp_rut}", oninput: move |e| corp_rut.set(e.value()) }
                        }
                    }
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Representante Legal (Nombre):" }
                            input { class: "form-input", value: "{rep_name}", oninput: move |e| rep_name.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "RUT Representante:" }
                            input { class: "form-input", value: "{rep_rut}", oninput: move |e| rep_rut.set(e.value()) }
                        }
                    }
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Email Representante:" }
                            input { class: "form-input", value: "{rep_email}", oninput: move |e| rep_email.set(e.value()) }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving() || corp_name().trim().is_empty(), onclick: do_save,
                            if saving() { "Guardando..." } else if edit_id().is_some() { "Actualizar" } else { "Crear" }
                        }
                        button { class: "btn", onclick: move |_| show_form.set(false), "Cancelar" }
                    }
                }
            }
            match corps_data {
                Some(Ok(ref data)) => {
                    let list = data["corporations"].as_array().cloned().unwrap_or_default();
                    let corps_clone = corps_signal.clone();
                    let on_toggle_corp = move |id: String| {
                        let mut r = corps_clone.clone();
                        spawn(async move {
                            let _ = client::admin_toggle_corporation(&id).await;
                            r.restart();
                        });
                    };
                    rsx! {
                        table { class: "data-table",
                            thead { tr {
                                th { "Nombre" } th { "RUT" } th { "Estado" } th { "Plan" }
                                th { "Colegios" } th { "Alumnos" } th { "Acciones" }
                            } }
                            tbody {
                                for c in &list {
                                    CorporationAdminRow {
                                        key: c["id"].as_str().unwrap_or(""),
                                        corp: c.clone(),
                                        on_edit: open_edit.clone(),
                                        on_toggle: on_toggle_corp.clone(),
                                    }
                                }
                                if list.is_empty() {
                                    tr { td { colspan: "7", class: "empty-state", "No hay corporaciones registradas" } }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
        ActivitySection { log_data: activity() }
    }
}

#[component]
fn CorporationAdminRow(corp: Value, on_edit: EventHandler<Value>, on_toggle: EventHandler<String>) -> Element {
    let id = corp["id"].as_str().unwrap_or("").to_string();
    let name = corp["name"].as_str().unwrap_or("").to_string();
    let rut = corp["rut"].as_str().unwrap_or("-").to_string();
    let active = corp["active"].as_bool().unwrap_or(true);
    let plan = corp["plan_name"].as_str().unwrap_or("-").to_string();
    let schools = corp["total_schools"].as_i64().unwrap_or(0);
    let students = corp["total_students"].as_i64().unwrap_or(0);
    let toggle_label = if active { "Deshabilitar" } else { "Habilitar" };
    let toggle_class = if active { "btn btn-xs btn-outline-danger" } else { "btn btn-xs btn-outline-success" };
    let id_toggle = id.clone();
    let mut expanded = use_signal(|| false);
    let mut show_form = use_signal(|| false);
    let mut edit_rep_id = use_signal(|| None::<String>);
    let mut rep_rut = use_signal(String::new);
    let mut rep_first = use_signal(String::new);
    let mut rep_last = use_signal(String::new);
    let mut rep_email = use_signal(String::new);
    let mut rep_phone = use_signal(String::new);
    let mut rep_addr = use_signal(String::new);
    let mut saving_rep = use_signal(|| false);
    let cid = id.clone();
    let reps = use_resource({
        let cid = cid.clone();
        move || {
            let cid = cid.clone();
            async move {
                if expanded() {
                    client::admin_fetch_legal_reps(&cid).await
                } else {
                    Err("".into())
                }
            }
        }
    });
    let corps_id = id.clone();

    let open_add = move |_| {
        edit_rep_id.set(None);
        rep_rut.set(String::new());
        rep_first.set(String::new());
        rep_last.set(String::new());
        rep_email.set(String::new());
        rep_phone.set(String::new());
        rep_addr.set(String::new());
        show_form.set(true);
    };

    let mut open_edit_rep = move |r: Value| {
        edit_rep_id.set(r["id"].as_str().map(|s| s.to_string()));
        rep_rut.set(r["rut"].as_str().unwrap_or("").to_string());
        rep_first.set(r["first_name"].as_str().unwrap_or("").to_string());
        rep_last.set(r["last_name"].as_str().unwrap_or("").to_string());
        rep_email.set(r["email"].as_str().unwrap_or("").to_string());
        rep_phone.set(r["phone"].as_str().unwrap_or("").to_string());
        rep_addr.set(r["address"].as_str().unwrap_or("").to_string());
        show_form.set(true);
    };

    let do_save_rep = move |_| {
        let payload = serde_json::json!({
            "corporation_id": corps_id,
            "rut": rep_rut(),
            "first_name": rep_first(),
            "last_name": rep_last(),
            "email": rep_email(),
            "phone": rep_phone(),
            "address": rep_addr(),
        });
        saving_rep.set(true);
        let eid = edit_rep_id();
        let mut r = reps.clone();
        spawn(async move {
            if let Some(ref rid) = eid {
                let _ = client::admin_update_legal_rep(rid, &payload).await;
            } else {
                let _ = client::admin_create_legal_rep(&payload).await;
            }
            saving_rep.set(false);
            show_form.set(false);
            r.restart();
        });
    };

    let do_deactivate_rep = move |rid: String| {
        let mut r = reps.clone();
        spawn(async move {
            let _ = client::admin_delete_legal_rep(&rid).await;
            r.restart();
        });
    };

    rsx! {
        tr { key: "{id}",
            td { "{name}" }
            td { class: "cell-mono", "{rut}" }
            td { if active { span { class: "badge badge-success", "Activo" } } else { span { class: "badge badge-danger", "Inactivo" } } }
            td { "{plan}" }
            td { "{schools}" }
            td { "{students}" }
            td { class: "cell-actions",
                button { class: "btn btn-xs btn-outline", onclick: move |_| on_edit.call(corp.clone()), "Editar" }
                button { class: "btn btn-xs btn-outline", onclick: move |_| expanded.set(!expanded()), if expanded() { "Ocultar Rep." } else { "Ver Rep." } }
                button { class: "{toggle_class}", onclick: move |_| on_toggle.call(id_toggle.clone()), "{toggle_label}" }
            }
        }
        if expanded() {
            tr { class: "expand-row", td { colspan: "7",
                div { class: "expand-content",
                    div { class: "section-header",
                        h4 { "Contactos de la Corporación" }
                        button { class: "btn btn-xs btn-primary", onclick: open_add, "Agregar Contacto" }
                    }
                    if show_form() {
                        div { class: "form-card compact-form",
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "RUT:" }
                                    input { class: "form-input", value: "{rep_rut}", oninput: move |e| rep_rut.set(e.value()) }
                                }
                                div { class: "form-group",
                                    label { "Nombre:" }
                                    input { class: "form-input", value: "{rep_first}", oninput: move |e| rep_first.set(e.value()) }
                                }
                                div { class: "form-group",
                                    label { "Apellido:" }
                                    input { class: "form-input", value: "{rep_last}", oninput: move |e| rep_last.set(e.value()) }
                                }
                            }
                            div { class: "form-row",
                                div { class: "form-group",
                                    label { "Email:" }
                                    input { class: "form-input", value: "{rep_email}", oninput: move |e| rep_email.set(e.value()) }
                                }
                                div { class: "form-group",
                                    label { "Teléfono:" }
                                    input { class: "form-input", value: "{rep_phone}", oninput: move |e| rep_phone.set(e.value()) }
                                }
                                div { class: "form-group",
                                    label { "Dirección:" }
                                    input { class: "form-input", value: "{rep_addr}", oninput: move |e| rep_addr.set(e.value()) }
                                }
                            }
                            div { class: "form-actions",
                                button { class: "btn btn-sm btn-primary", disabled: saving_rep() || rep_rut().trim().is_empty(), onclick: do_save_rep,
                                    if saving_rep() { "Guardando..." } else if edit_rep_id().is_some() { "Actualizar" } else { "Agregar" }
                                }
                                button { class: "btn btn-sm", onclick: move |_| show_form.set(false), "Cancelar" }
                            }
                        }
                    }
                    match reps() {
                        Some(Ok(data)) => {
                            let items: Vec<Value> = data["legal_representatives"].as_array().cloned().unwrap_or_default();
                            let rows: Vec<_> = items.into_iter().map(|r| {
                                let rid = r["id"].as_str().unwrap_or("").to_string();
                                let rut_v = r["rut"].as_str().unwrap_or("-").to_string();
                                let first = r["first_name"].as_str().unwrap_or("").to_string();
                                let last = r["last_name"].as_str().unwrap_or("").to_string();
                                let email_v = r["email"].as_str().unwrap_or("-").to_string();
                                let phone_v = r["phone"].as_str().unwrap_or("-").to_string();
                                let rep_active = r["active"].as_bool().unwrap_or(true);
                                let r_val = r.clone();
                                let rid_val = rid.clone();
                                rsx! {
                                    tr {
                                        td { class: "cell-mono", "{rut_v}" }
                                        td { "{first} {last}" }
                                        td { "{email_v}" }
                                        td { "{phone_v}" }
                                        td {
                                            if rep_active { span { class: "badge badge-success", "Activo" } }
                                            else { span { class: "badge badge-danger", "Inactivo" } }
                                        }
                                        td { class: "cell-actions",
                                            button { class: "btn btn-xs btn-outline", onclick: move |_| open_edit_rep(r_val.clone()), "Editar" }
                                            if rep_active {
                                                button { class: "btn btn-xs btn-outline-danger", onclick: move |_| do_deactivate_rep(rid_val.clone()), "Desactivar" }
                                            }
                                        }
                                    }
                                }
                            }).collect();
                            rsx! {
                                table { class: "data-table mini-table",
                                    thead { tr { th { "RUT" } th { "Nombre" } th { "Email" } th { "Teléfono" } th { "Estado" } th { "Acciones" } } }
                                    tbody { {rows.into_iter()} }
                                }
                            }
                        }
                        Some(Err(_)) => rsx! { div { class: "empty-state", "Error al cargar" } },
                        None => rsx! { div { class: "loading-spinner", "Cargando..." } },
                    }
                }
            } }
        }
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
