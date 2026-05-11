use crate::api::client;
use crate::components::widgets::admission_metrics::AdmissionMetricsWidget;
use crate::components::widgets::business_process_flow::BusinessProcessFlow;
use crate::components::widgets::custom_fields_section::CustomFieldsSection;
use dioxus::prelude::*;

#[component]
pub fn AdmissionPage() -> Element {
    let stages = use_resource(|| client::fetch_pipeline_stages());
    let mut prospects = use_resource(|| client::fetch_prospects());
    let mut selected_id = use_signal(|| None::<String>);
    let mut prospect_detail = use_resource(move || {
        let sid = selected_id();
        async move {
            match sid {
                Some(id) => client::fetch_prospect(&id).await,
                None => Err("none".to_string()),
            }
        }
    });
    let mut show_new = use_signal(|| false);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut rut = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut source = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let mut editing_prospect = use_signal(|| false);
    let mut view_mode = use_signal(|| "table".to_string());
    let mut edit_first_name = use_signal(String::new);
    let mut edit_last_name = use_signal(String::new);
    let mut edit_rut = use_signal(String::new);
    let mut edit_email = use_signal(String::new);
    let mut edit_phone = use_signal(String::new);
    let mut edit_source = use_signal(String::new);
    let mut edit_notes = use_signal(String::new);
    let vacancies = use_resource(|| client::check_vacancies());

    let do_create = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "first_name": first_name(),
            "last_name": last_name(),
            "rut": rut(),
            "email": email(),
            "phone": phone(),
            "source": source(),
            "notes": notes(),
        });
        spawn(async move {
            let _ = client::create_prospect(&payload).await;
            saving.set(false);
            show_new.set(false);
            first_name.set(String::new());
            last_name.set(String::new());
            rut.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            source.set(String::new());
            notes.set(String::new());
            prospects.restart();
        });
    };

    let columns: Vec<Element> = match (stages(), prospects()) {
        (Some(Ok(sj)), Some(Ok(pj))) => {
            let stage_list = sj["stages"].as_array().cloned().unwrap_or_default();
            let prospect_list = pj["prospects"].as_array().cloned().unwrap_or_default();
            stage_list.iter().map(|stage| {
                let stage_id = stage["id"].as_str().unwrap_or("").to_string();
                let stage_name = stage["name"].as_str().unwrap_or("").to_string();
                let cards: Vec<(&str, String, String)> = prospect_list.iter()
                    .filter(|p| p["current_stage_id"].as_str().unwrap_or("") == stage_id)
                    .map(|p| {
                        let pid = p["id"].as_str().unwrap_or("");
                        let pname = format!("{} {}",
                            p["first_name"].as_str().unwrap_or(""),
                            p["last_name"].as_str().unwrap_or(""),
                        );
                        let prut = p["rut"].as_str().unwrap_or("").to_string();
                        (pid, pname, prut)
                    })
                    .collect();
                let card_count = cards.len();
                let sel = selected_id();
                let card_elements: Vec<Element> = cards.iter().map(|(pid, pname, prut)| {
                    let pid_s = pid.to_string();
                    let is_sel = sel.as_deref() == Some(pid);
                    rsx! {
                        div {
                            class: "kanban-card",
                            class: if is_sel { "selected" } else { "" },
                            onclick: move |_| { selected_id.set(Some(pid_s.clone())); prospect_detail.restart(); },
                            div { class: "card-name", "{pname}" }
                            div { class: "card-rut", "{prut}" }
                        }
                    }
                }).collect();

                rsx! {
                    div { class: "kanban-column", key: "{stage_id}",
                        div { class: "kanban-column-header",
                            h3 { "{stage_name}" }
                            span { class: "kanban-count", "{card_count}" }
                        }
                        div { class: "kanban-cards", { card_elements.into_iter() } }
                    }
                }
            }).collect()
        }
        _ => vec![],
    };

    rsx! {
        div { class: "page-header",
            h1 { "Admisiones" }
            p { "Pipeline de postulantes - gestione el ciclo de admisión" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_new.set(!show_new()), if show_new() { "Cancelar" } else { "Nuevo Postulante" } }
            button { class: "btn btn-secondary", onclick: move |_| view_mode.set(if view_mode() == "kanban" { "table".to_string() } else { "kanban".to_string() }),
                if view_mode() == "kanban" { "Vista Tabla" } else { "Vista Kanban" }
            }
        }
        {
            if show_new() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombres:" }
                                input { class: "form-input", value: "{first_name}", oninput: move |e| first_name.set(e.value()), placeholder: "Juan" }
                            }
                            div { class: "form-group",
                                label { "Apellidos:" }
                                input { class: "form-input", value: "{last_name}", oninput: move |e| last_name.set(e.value()), placeholder: "Pérez" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "RUT:" }
                                input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" }
                            }
                            div { class: "form-group",
                                label { "Email:" }
                                input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "juan@ejemplo.cl" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Teléfono:" }
                                input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()), placeholder: "+56 9 1234 5678" }
                            }
                            div { class: "form-group",
                                label { "Origen:" }
                                select { class: "form-input", value: "{source}", oninput: move |e| source.set(e.value()),
                                    option { value: "", "Seleccionar..." }
                                    option { value: "web", "Sitio Web" }
                                    option { value: "referido", "Referido" }
                                    option { value: "red_social", "Red Social" }
                                    option { value: "feria", "Feria Educativa" }
                                    option { value: "otro", "Otro" }
                                }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_create, if saving() { "Guardando..." } else { "Crear Postulante" } }
                            button { class: "btn", onclick: move |_| show_new.set(false), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "dashboard-grid", AdmissionMetricsWidget {} }
        { if view_mode() == "kanban" {
            rsx! {
                div { class: "kanban-board",
                    {
                        if columns.is_empty() && stages().is_some() {
                            rsx! { div { class: "empty-state", "No hay postulantes en ninguna etapa" } }
                        } else {
                            rsx! { { columns.into_iter() } }
                        }
                    }
                }
            }
        } else {
            rsx! { ProspectTable { prospects: prospects } }
        }}
        div { class: "vacancy-section",
            h3 { "Disponibilidad por Nivel" }
            {
                match vacancies() {
                    Some(Ok(j)) => {
                        let list = j["vacancies"].as_array().cloned().unwrap_or_default();
                        let rows: Vec<(String, i64, i64, i64)> = list.iter().map(|v| {
                            (v["grade_level"].as_str().unwrap_or("").to_string(),
                             v["total_capacity"].as_i64().unwrap_or(0),
                             v["enrolled_count"].as_i64().unwrap_or(0),
                             v["available"].as_i64().unwrap_or(0))
                        }).collect();
                        rsx! {
                            div { class: "vacancy-grid",
                                for (level, cap, enrolled, avail) in &rows {
                                    div { class: "vacancy-card",
                                        div { class: "vacancy-level", "{level}" }
                                        div { class: "vacancy-numbers",
                                            span { "Cupos: {cap}  |  " }
                                            span { "Matriculados: {enrolled}  |  " }
                                            span { class: if *avail > 0 { "vacancy-ok" } else { "vacancy-full" }, "Disponibles: {avail}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando disponibilidad..." } },
                }
            }
        }

        {
            match prospect_detail() {
                Some(Ok(j)) => {
                    let p = &j["prospect"];
                    let activities = j["activities"].as_array().cloned().unwrap_or_default();
                    let documents = j["documents"].as_array().cloned().unwrap_or_default();
                    let pid = p["id"].as_str().unwrap_or("").to_string();
                    let pname = format!("{} {}", p["first_name"].as_str().unwrap_or(""), p["last_name"].as_str().unwrap_or(""));
                    let prut = p["rut"].as_str().unwrap_or("-").to_string();
                    let pemail = p["email"].as_str().unwrap_or("-").to_string();
                    let pphone = p["phone"].as_str().unwrap_or("-").to_string();
                    let psource = p["source"].as_str().unwrap_or("-").to_string();
                    let pstage = p["current_stage_id"].as_str().unwrap_or("").to_string();

                    let stage_options: Vec<Element> = stages().and_then(|r| r.ok()).map(|sj| {
                        sj["stages"].as_array().cloned().unwrap_or_default().into_iter().filter_map(|s| {
                            let sid = s["id"].as_str()?.to_string();
                            let sname = s["name"].as_str()?.to_string();
                            Some(rsx! { option { value: "{sid}", selected: sid == pstage, "{sname}" } })
                        }).collect::<Vec<_>>()
                    }).unwrap_or_default();

                    let do_stage_change = { let pid = pid.clone(); move |e: Event<FormData>| {
                        let new_stage = e.value();
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::change_prospect_stage(&pid, &new_stage).await;
                                prospect_detail.restart();
                                prospects.restart();
                            }
                        });
                    }};

                    let do_delete_prospect = { let pid = pid.clone(); move |_| {
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::delete_prospect(&pid).await;
                                selected_id.set(None);
                                prospects.restart();
                            }
                        });
                    }};

                    let do_edit_save = { let pid = pid.clone(); move |_| {
                        saving.set(true);
                        let payload = serde_json::json!({
                            "first_name": edit_first_name(),
                            "last_name": edit_last_name(),
                            "rut": edit_rut(),
                            "email": edit_email(),
                            "phone": edit_phone(),
                            "source": edit_source(),
                            "notes": edit_notes(),
                        });
                        spawn({
                            let pid = pid.clone();
                            async move {
                                let _ = client::update_prospect(&pid, &payload).await;
                                saving.set(false);
                                editing_prospect.set(false);
                                prospect_detail.restart();
                                prospects.restart();
                            }
                        });
                    }};

                    let activity_items: Vec<Element> = activities.iter().map(|a| {
                        let atype = a["activity_type"].as_str().unwrap_or("").to_string();
                        let asubj = a["subject"].as_str().unwrap_or("").to_string();
                        rsx! { div { class: "activity-item",
                            span { class: "activity-type", "{atype}" }
                            span { "{asubj}" }
                        }}
                    }).collect();

                    let doc_items: Vec<Element> = documents.iter().map(|d| {
                        let fname = d["file_name"].as_str().unwrap_or("").to_string();
                        let verified = d["is_verified"].as_bool().unwrap_or(false);
                        rsx! { div { class: "doc-item",
                            span { "{fname}" }
                            span { class: "doc-status",
                                if verified { "✓ Verificado" } else { "⏳ Pendiente" }
                            }
                        }}
                    }).collect();

                    rsx! {
                        div { class: "modal-overlay", onclick: move |_| selected_id.set(None),
                            div { class: "modal-content", onclick: move |e| e.stop_propagation(),
                                div { class: "modal-header",
                                    h2 { "{pname}" }
                                    button { class: "btn-icon", onclick: move |_| selected_id.set(None), "✕" }
                                }
                                {
                                    let stages_data = stages();
                                    let p_stage = pstage.clone();
                                    match stages_data {
                                        Some(Ok(sj)) => {
                                            let list = sj["stages"].as_array().cloned().unwrap_or_default();
                                            rsx! {
                                                BusinessProcessFlow { stages: list, current_stage_id: p_stage }
                                            }
                                        }
                                        _ => rsx! {},
                                    }
                                }
                                div { class: "modal-body",
                                    if editing_prospect() {
                                        div { class: "form-card",
                                            h4 { "Editar Postulante" }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "Nombres:" }
                                                    input { class: "form-input", value: "{edit_first_name}", oninput: move |e| edit_first_name.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Apellidos:" }
                                                    input { class: "form-input", value: "{edit_last_name}", oninput: move |e| edit_last_name.set(e.value()) }
                                                }
                                            }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "RUT:" }
                                                    input { class: "form-input", value: "{edit_rut}", oninput: move |e| edit_rut.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Email:" }
                                                    input { class: "form-input", value: "{edit_email}", oninput: move |e| edit_email.set(e.value()) }
                                                }
                                            }
                                            div { class: "form-row",
                                                div { class: "form-group",
                                                    label { "Teléfono:" }
                                                    input { class: "form-input", value: "{edit_phone}", oninput: move |e| edit_phone.set(e.value()) }
                                                }
                                                div { class: "form-group",
                                                    label { "Origen:" }
                                                    select { class: "form-input", value: "{edit_source}", oninput: move |e| edit_source.set(e.value()),
                                                        option { value: "", "Seleccionar..." }
                                                        option { value: "web", "Sitio Web" }
                                                        option { value: "referido", "Referido" }
                                                        option { value: "red_social", "Red Social" }
                                                        option { value: "feria", "Feria Educativa" }
                                                        option { value: "otro", "Otro" }
                                                    }
                                                }
                                            }
                                            div { class: "form-actions",
                                                button { class: "btn btn-primary", disabled: saving(), onclick: do_edit_save, if saving() { "Guardando..." } else { "Guardar" } }
                                                button { class: "btn", onclick: move |_| editing_prospect.set(false), "Cancelar" }
                                            }
                                        }
                                    } else {
                                        div { class: "detail-grid",
                                            div { class: "detail-section",
                                                h4 { "Datos Personales" }
                                                p { "RUT: {prut}" }
                                                p { "Email: {pemail}" }
                                                p { "Teléfono: {pphone}" }
                                                p { "Origen: {psource}" }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Cambiar Etapa" }
                                                select { class: "form-input", oninput: do_stage_change, {stage_options.into_iter()} }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Actividades ({activities.len()})" }
                                                {
                                                    if activity_items.is_empty() {
                                                        rsx! { p { "Sin actividades registradas" } }
                                                    } else {
                                                        rsx! { { activity_items.into_iter() } }
                                                    }
                                                }
                                            }
                                            div { class: "detail-section",
                                                h4 { "Documentos ({documents.len()})" }
                                                {
                                                    if doc_items.is_empty() {
                                                        rsx! { p { "Sin documentos" } }
                                                    } else {
                                                        rsx! { { doc_items.into_iter() } }
                                                    }
                                                }
                                            }
                                            CustomFieldsSection { entity_id: pid.clone(), entity_type: "prospect".to_string() }
                                        }
                                    }
                                }
                                div { class: "modal-footer",
                                    button { class: "btn", onclick: { let fn_ = p["first_name"].as_str().unwrap_or("").to_string(); let ln = p["last_name"].as_str().unwrap_or("").to_string(); let r = p["rut"].as_str().unwrap_or("").to_string(); let e = p["email"].as_str().unwrap_or("").to_string(); let ph = p["phone"].as_str().unwrap_or("").to_string(); let s = p["source"].as_str().unwrap_or("").to_string(); let n = p["notes"].as_str().unwrap_or("").to_string(); move |_| { edit_first_name.set(fn_.clone()); edit_last_name.set(ln.clone()); edit_rut.set(r.clone()); edit_email.set(e.clone()); edit_phone.set(ph.clone()); edit_source.set(s.clone()); edit_notes.set(n.clone()); editing_prospect.set(true); } }, "Editar" }
                                    button { class: "btn btn-danger", onclick: do_delete_prospect, "Eliminar" }
                                }
                            }
                        }
                    }
                }
                Some(Err(_)) => rsx! {},
                None => rsx! {},
            }
        }
    }
}

#[component]
fn ProspectTable(prospects: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match prospects() {
                Some(Ok(data)) => {
                    let list = data["prospects"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "No hay postulantes" } }
                    } else {
                        let _stages_data = prospects();
                        let rows: Vec<Element> = list.iter().map(|p| {
                            let _pid = p["id"].as_str().unwrap_or("").to_string();
                            let name = format!("{} {}",
                                p["first_name"].as_str().unwrap_or(""),
                                p["last_name"].as_str().unwrap_or("")
                            );
                            let rut = p["rut"].as_str().unwrap_or("-").to_string();
                            let stage = p["current_stage_name"].as_str().unwrap_or("-").to_string();
                            let source = p["source"].as_str().unwrap_or("-").to_string();
                            let date = p["created_at"].as_str().unwrap_or("").to_string();
                            rsx! {
                                tr { class: "clickable-row", onclick: move |_| {
                                    // select this prospect
                                },
                                    td { "{name}" }
                                    td { "{rut}" }
                                    td { span { class: "role-badge", "{stage}" } }
                                    td { "{source}" }
                                    td { "{date}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Nombre" }
                                    th { "RUT" }
                                    th { "Etapa" }
                                    th { "Origen" }
                                    th { "Creado" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando..." } },
            }
        }
    }
}
