use dioxus::prelude::*;
use serde_json::{Value, json};
use crate::api::client;

#[component]
pub fn SalesPage() -> Element {
    let stages = use_resource(|| client::fetch_json("/api/sales/stages"));
    let mut prospects = use_resource(|| client::fetch_json("/api/sales/prospects"));
    let mut selected_id = use_signal(|| None::<String>);
    let mut show_new = use_signal(|| false);
    let mut view_mode = use_signal(|| "kanban".to_string());
    let mut search_term = use_signal(String::new);

    let prospect_detail = use_resource(move || {
        let sid = selected_id();
        async move {
            match sid {
                Some(id) => client::fetch_json(&format!("/api/sales/prospects/{}", id)).await,
                None => Err("none".to_string()),
            }
        }
    });

    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut company = use_signal(String::new);
    let mut rut = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut source = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let do_create = move |_| {
        saving.set(true);
        let payload = json!({
            "first_name": first_name(), "last_name": last_name(), "email": email(),
            "phone": phone(), "company": company(), "rut": rut(), "position": position(),
            "source": source(), "notes": notes(),
        });
        spawn(async move {
            let _ = client::post_json("/api/sales/prospects", &payload).await;
            saving.set(false);
            show_new.set(false);
            first_name.set(String::new()); last_name.set(String::new()); email.set(String::new());
            phone.set(String::new()); company.set(String::new()); rut.set(String::new()); position.set(String::new());
            source.set(String::new()); notes.set(String::new());
            prospects.restart();
        });
    };

    let kanban_active = view_mode() == "kanban";
    let table_active = view_mode() == "table";

    rsx! {
        div { class: "page-header",
            h1 { "CRM de Ventas" }
            p { "Pipeline comercial — gesti\u{00f3}n de prospectos, propuestas y contratos" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_new.set(!show_new()),
                if show_new() { "Cancelar" } else { "Nuevo Prospecto" }
            }
            button { class: "btn {kanban_active}", onclick: move |_| view_mode.set("kanban".to_string()), "Kanban" }
            button { class: "btn {table_active}", onclick: move |_| view_mode.set("table".to_string()), "Tabla" }
            input { class: "search-input", placeholder: "Buscar prospecto...", value: "{search_term}", oninput: move |e| search_term.set(e.value()) }
        }
        if show_new() {
            div { class: "form-card",
                div { class: "form-row",
                    div { class: "form-group", label { "Nombre *" } input { class: "form-input", value: "{first_name}", oninput: move |e| first_name.set(e.value()) } }
                    div { class: "form-group", label { "Apellido *" } input { class: "form-input", value: "{last_name}", oninput: move |e| last_name.set(e.value()) } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Email" } input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()) } }
                    div { class: "form-group", label { "Tel\u{00e9}fono" } input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()) } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Colegio" } input { class: "form-input", value: "{company}", oninput: move |e| company.set(e.value()) } }
                    div { class: "form-group", label { "RUT (Empresa/Persona)" } input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" } }
                }
                div { class: "form-row",
                    div { class: "form-group", label { "Cargo" } input { class: "form-input", value: "{position}", oninput: move |e| position.set(e.value()) } }
                }
                div { class: "form-row",
                    div { class: "form-group",
                        label { "Fuente" }
                        select { class: "form-input", value: "{source}", oninput: move |e| source.set(e.value()),
                            option { value: "", "Seleccionar..." }
                            option { value: "web", "Web" } option { value: "referido", "Referido" }
                            option { value: "llamada", "Llamada" } option { value: "whatsapp", "WhatsApp" }
                            option { value: "email", "Email" } option { value: "feria", "Feria" } option { value: "otro", "Otro" }
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn btn-primary", disabled: saving() || first_name().trim().is_empty() || last_name().trim().is_empty(), onclick: do_create,
                        if saving() { "Creando..." } else { "Crear Prospecto" }
                    }
                }
            }
        }
        if view_mode() == "kanban" {
            match stages() {
                Some(Ok(data)) => {
                    let sl: Vec<Value> = data["stages"].as_array().cloned().unwrap_or_default();
                    rsx! { SalesKanbanBoard { stages: sl, prospects: prospects } }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando pipeline..." } },
            }
        } else {
            match prospects() {
                Some(Ok(pdata)) => {
                    let list: Vec<Value> = pdata["prospects"].as_array().cloned().unwrap_or_default();
                    let stages_map: std::collections::HashMap<String, String> = match stages() {
                        Some(Ok(d)) => d["stages"].as_array().cloned().unwrap_or_default()
                            .into_iter().filter_map(|s| {
                                Some((s["id"].as_str()?.to_string(), s["name"].as_str()?.to_string()))
                            }).collect(),
                        _ => std::collections::HashMap::new(),
                    };
                    rsx! { SalesTableView { prospects: list, stages_map: stages_map } }
                }
                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
        if selected_id().is_some() {
            ProspectDetailModal {
                detail: prospect_detail,
                on_close: move |_| selected_id.set(None),
            }
        }
    }
}

#[component]
fn SalesKanbanBoard(stages: Vec<Value>, prospects: Resource<Result<Value, String>>) -> Element {
    let prospect_list: Vec<Value> = match prospects() {
        Some(Ok(d)) => d["prospects"].as_array().cloned().unwrap_or_default(),
        _ => vec![],
    };

    let stage_infos: Vec<StageInfo> = stages.iter().map(|s| {
        let stage_id = s["id"].as_str().unwrap_or("").to_string();
        let prospect_ids: Vec<String> = prospect_list.iter()
            .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
            .map(|p| p["id"].as_str().unwrap_or("").to_string())
            .collect();
        let names: Vec<String> = prospect_list.iter()
            .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
            .map(|p| format!("{} {}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or("")))
            .collect();
        let companies: Vec<String> = prospect_list.iter()
            .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
            .map(|p| p["company"].as_str().unwrap_or("").to_string())
            .collect();
        StageInfo {
            id: stage_id,
            name: s["name"].as_str().unwrap_or("").to_string(),
            color: s["color"].as_str().unwrap_or("#6B7280").to_string(),
            prospect_ids,
            names,
            companies,
        }
    }).collect();

    rsx! {
        div { class: "kanban-board",
            for si in &stage_infos {
                div { class: "kanban-column",
                    div { class: "kanban-column-header", style: "border-top-color: {si.color}",
                        div { class: "kanban-column-title", "{si.name}" }
                        div { class: "kanban-column-count", "{si.prospect_ids.len()}" }
                    }
                    div { class: "kanban-column-body",
                        for i in 0..si.prospect_ids.len() {
                            SalesKanbanCard {
                                prospect_id: si.prospect_ids[i].clone(),
                                name: si.names[i].clone(),
                                company: si.companies[i].clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

struct StageInfo {
    _id: String,
    name: String,
    color: String,
    prospect_ids: Vec<String>,
    names: Vec<String>,
    companies: Vec<String>,
}

#[component]
fn SalesKanbanCard(prospect_id: String, name: String, company: String) -> Element {
    rsx! {
        div { class: "kanban-card",
            div { class: "kanban-card-name", "{name}" }
            if !company.is_empty() {
                div { class: "kanban-card-company", "{company}" }
            }
        }
    }
}

#[component]
fn SalesTableView(prospects: Vec<Value>, stages_map: std::collections::HashMap<String, String>) -> Element {
    rsx! {
        div { class: "data-table-container",
            table { class: "data-table",
                thead { tr { th { "Nombre" } th { "RUT" } th { "Email" } th { "Colegio" } th { "Etapa" } th { "Valor" } th { "Acciones" } } }
                tbody {
                    for p in &prospects {
                        SalesTableRow { prospect: p.clone(), stages_map: stages_map.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn SalesTableRow(prospect: Value, stages_map: std::collections::HashMap<String, String>) -> Element {
    let first = prospect["first_name"].as_str().unwrap_or("").to_string();
    let last = prospect["last_name"].as_str().unwrap_or("").to_string();
    let email = prospect["email"].as_str().unwrap_or("-").to_string();
    let _phone = prospect["phone"].as_str().unwrap_or("-").to_string();
    let company = prospect["company"].as_str().unwrap_or("-").to_string();
    let stage_id = prospect["current_stage_id"].as_str().unwrap_or("").to_string();
    let stage_name = stages_map.get(&stage_id).cloned().unwrap_or_else(|| "-".to_string());
    let val = prospect["estimated_value"].as_f64().map(|v| format!("${:.0}", v)).unwrap_or_else(|| "-".to_string());

    rsx! {
        tr {
            td { "{first} {last}" }
            td { "{prospect[\"rut\"].as_str().unwrap_or(\"-\")}" }
            td { "{email}" }
            td { "{company}" }
            td { span { class: "stage-badge", "{stage_name}" } }
            td { "{val}" }
            td { button { class: "btn btn-sm", "Ver" } }
        }
    }
}

#[component]
fn ProspectDetailModal(
    detail: Resource<Result<Value, String>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut show_timeline = use_signal(|| true);
    let mut activation_result = use_signal(|| None::<Value>);
    let mut is_activating = use_signal(|| false);

    let detail_data = match detail() {
        Some(Ok(ref data)) => Some(data.clone()),
        _ => None,
    };

    let (header_el, body_el, _pid) = match detail_data {
        Some(ref data) => {
            let p = &data["prospect"];
            let stage = &data["stage"];
            let assigned = &data["assigned_user"];
            let contracts = data["contracts"].as_array().cloned().unwrap_or_default();
            let prospect_id = p["id"].as_str().unwrap_or("").to_string();
            let first_name = p["first_name"].as_str().unwrap_or("").to_string();
            let last_name = p["last_name"].as_str().unwrap_or("").to_string();
            let stage_color = stage["color"].as_str().unwrap_or("#6B7280").to_string();
            let stage_name = stage["name"].as_str().unwrap_or("Sin etapa").to_string();
            let email_val = p["email"].as_str().unwrap_or("-").to_string();
            let phone_val = p["phone"].as_str().unwrap_or("-").to_string();
            let company_val = p["company"].as_str().unwrap_or("-").to_string();
            let source_val = p["source"].as_str().unwrap_or("-").to_string();
            let value_val = p["estimated_value"].as_f64().map(|v| format!("${:.0}", v)).unwrap_or_else(|| "-".to_string());
            let assigned_name = assigned["name"].as_str().unwrap_or("-").to_string();
            let contract_cards: Vec<_> = contracts.iter().map(|c| {
                let cs = c["status"].as_str().unwrap_or("draft").to_string();
                let cv = c["total_value"].as_f64().unwrap_or(0.0);
                let cid = c["id"].as_str().unwrap_or("").to_string();
                rsx! { 
                    div { class: "contract-card", 
                        div { class: "contract-status-{cs}", "{cs}" } 
                        div { "Valor: ${cv}" } 
                        if cs == "verified" {
                            button { 
                                class: "btn btn-sm btn-success", 
                                disabled: is_activating(),
                                onclick: move |_| {
                                    let id = cid.clone();
                                    spawn(async move {
                                        is_activating.set(true);
                                        match client::post_json(&format!("/api/sales/contracts/{}/activate", id), &json!({})).await {
                                            Ok(resp) => {
                                                activation_result.set(Some(resp));
                                                detail.restart(); // Auto-refresh data
                                            }
                                            Err(_e) => {
                                                // Handle error (maybe another signal for error message)
                                                // console log error
                                            }
                                        }
                                        is_activating.set(false);
                                    });
                                },
                                if is_activating() { "Activando..." } else { "Activar Licencia" }
                            }
                        }
                    } 
                }
            }).collect();

            let h = rsx! {
                h2 { "{first_name} {last_name}" }
                span { class: "stage-badge", style: "background: {stage_color}", "{stage_name}" }
            };

            (h, rsx! {
                div { class: "detail-tabs",
                    button { class: "tab", onclick: move |_| show_timeline.set(true), "Actividad" }
                    button { class: "tab", onclick: move |_| show_timeline.set(false), "Info" }
                }
                if show_timeline() {
                    ContactTimeline { prospect_id: prospect_id.clone() }
                } else {
                    div { class: "detail-grid",
                        div { class: "detail-section",
                            h3 { "Informaci\u{00f3}n" }
                            div { class: "detail-row", label { "Email:" }, span { "{email_val}" } }
                            div { class: "detail-row", label { "RUT:" }, span { "{p[\"rut\"].as_str().unwrap_or(\"-\")}" } }
                            div { class: "detail-row", label { "Tel\u{00e9}fono:" }, span { "{phone_val}" } }
                            div { class: "detail-row", label { "Colegio:" }, span { "{company_val}" } }
                            div { class: "detail-row", label { "Fuente:" }, span { "{source_val}" } }
                            div { class: "detail-row", label { "Valor est.:" }, span { "{value_val}" } }
                            div { class: "detail-row", label { "Asignado:" }, span { "{assigned_name}" } }
                        }
                    }
                }
                div { class: "detail-actions",
                    button { class: "btn btn-primary", onclick: move |_| {}, "Crear Propuesta" }
                    button { class: "btn", onclick: move |_| {}, "Crear Contrato" }
                }
                if !contract_cards.is_empty() {
                    div { class: "detail-section",
                        h3 { "Contratos" }
                        {contract_cards.into_iter()}
                    }
                }
            }, prospect_id)
        }
        None => (rsx! { h2 { "Cargando..." } }, rsx! { div { class: "loading-spinner", "Cargando..." } }, String::new()),
    };

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal-content modal-lg", onclick: move |e| e.stop_propagation(),
                if let Some(res) = activation_result() {
                    div { class: "p-8 text-center",
                        h3 { class: "text-2xl font-bold text-success mb-4", "✅ Licencia Activada" }
                        p { class: "mb-6", "La corporación y el colegio han sido creados exitosamente." }
                        div { class: "bg-gray-50 p-6 rounded-lg mb-6 text-left border border-gray-200",
                            div { class: "mb-2", b { "Email: " } "{res[\"admin_email\"].as_str().unwrap_or(\"-\")}" }
                            div { class: "mb-2", b { "Contraseña Temporal: " } span { class: "font-mono bg-blue-50 text-blue-700 px-2 py-1 rounded", "{res[\"temp_password\"].as_str().unwrap_or(\"-\")}" } }
                        }
                        p { class: "text-sm text-gray-500 mb-6", "Por favor, comparte estas credenciales con el sostenedor. Se recomienda cambiarlas tras el primer ingreso." }
                        button { 
                            class: "btn btn-primary w-full", 
                            onclick: move |_| activation_result.set(None), 
                            "Entendido" 
                        }
                    }
                } else {
                    div { class: "modal-header", 
                        {header_el} 
                        button { class: "btn-close", onclick: move |_| on_close.call(()) } 
                    }
                    div { class: "modal-body", {body_el} }
                }
            }
        }
    }
}

#[component]
fn ContactTimeline(prospect_id: String) -> Element {
    let activities = use_resource(move || {
        let pid = prospect_id.clone();
        async move { client::fetch_json(&format!("/api/sales/prospects/{}/activities", pid)).await }
    });

    let items = match activities() {
        Some(Ok(data)) => {
            let list = data["activities"].as_array().cloned().unwrap_or_default();
            let cards: Vec<_> = list.iter().map(|act| {
                let atype = act["activity_type"].as_str().unwrap_or("").to_string();
                let subject = act["subject"].as_str().unwrap_or("").to_string();
                let desc = act["description"].as_str().unwrap_or("").to_string();
                let created = act["created_at"].as_str().unwrap_or("").to_string();
                let icon = match atype.as_str() {
                    "call" => "\u{260E}", "email" => "\u{2709}", "whatsapp" => "\u{1F4AC}",
                    "meeting" => "\u{1F91D}", "proposal" => "\u{1F4C4}",
                    "contract" => "\u{1F4C3}", "activation" => "\u{2705}",
                    "stage_change" => "\u{1F500}", "assign" => "\u{1F464}",
                    _ => "\u{1F4CB}",
                };
                rsx! {
                    div { class: "timeline-item",
                        div { class: "timeline-icon", "{icon}" }
                        div { class: "timeline-content",
                            div { class: "timeline-subject", "{subject}" }
                            if !desc.is_empty() { div { class: "timeline-desc", "{desc}" } }
                            div { class: "timeline-date", "{created}" }
                        }
                    }
                }
            }).collect();
            if cards.is_empty() {
                rsx! { div { class: "empty-state", "Sin actividad registrada" } }
            } else {
                rsx! { div { class: "timeline", {cards.into_iter()} } }
            }
        }
        Some(Err(_)) => rsx! { div { class: "empty-state", "Error al cargar actividades" } },
        None => rsx! { div { class: "loading-spinner", "Cargando..." } },
    };

    rsx! {
        div { class: "timeline-container",
            h3 { "Historial de Actividad" }
            {items}
        }
    }
}
