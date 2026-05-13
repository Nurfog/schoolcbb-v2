use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

#[component]
pub fn CorporationsPage() -> Element {
    let mut corps = use_resource(|| client::fetch_corporations());
    let mut show_corp_form = use_signal(|| false);
    let mut corp_name = use_signal(String::new);
    let mut corp_rut = use_signal(String::new);
    let mut saving_corp = use_signal(|| false);
    let mut expanded_corp = use_signal(|| None::<String>);

    let do_create_corp = move |_| {
        let name = corp_name();
        let rut = corp_rut();
        if name.trim().is_empty() || rut.trim().is_empty() {
            return;
        }
        saving_corp.set(true);
        spawn({
            async move {
                let _ = client::create_corporation(&json!({"name": name, "rut": rut})).await;
                saving_corp.set(false);
                show_corp_form.set(false);
                corp_name.set(String::new());
                corp_rut.set(String::new());
                corps.restart();
            }
        });
    };

    let toggle_expand = move |id: String| {
        if expanded_corp() == Some(id.clone()) {
            expanded_corp.set(None);
        } else {
            expanded_corp.set(Some(id));
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Corporaciones y Colegios" }
            p { "Gesti\u{00f3}n de la estructura multi-tenant del sistema" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_corp_form.set(!show_corp_form()),
                if show_corp_form() { "Cancelar" } else { "Nueva Corporaci\u{00f3}n" }
            }
        }
        {
            if show_corp_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre de la corporaci\u{00f3}n:" }
                                input { class: "form-input", value: "{corp_name}", oninput: move |e| corp_name.set(e.value()), placeholder: "Ej: Corporaci\u{00f3}n Educacional ABC" }
                            }
                            div { class: "form-group",
                                label { "RUT:" }
                                input { class: "form-input", value: "{corp_rut}", oninput: move |e| corp_rut.set(e.value()), placeholder: "Ej: 12.345.678-9" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving_corp(), onclick: do_create_corp,
                                if saving_corp() { "Creando..." } else { "Crear Corporaci\u{00f3}n" }
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match corps() {
                Some(Ok(data)) => {
                    let list = data["corporations"].as_array().cloned().unwrap_or_default();
                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "RUT" } th { "Nombre" } th { "Colegios" } th { "Estado" } th { "Acciones" } } }
                            tbody { for corp in &list {
                                CorporationRow {
                                    key: corp["id"].as_str().unwrap_or(""),
                                    corp: corp.clone(),
                                    is_expanded: expanded_corp() == corp["id"].as_str().map(|s| s.to_string()),
                                    on_toggle: toggle_expand.clone(),
                                }
                            }}
                        }
                    }
                }
                Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
            }
        }
    }
}

#[component]
fn CorporationRow(corp: Value, is_expanded: bool, on_toggle: EventHandler<String>) -> Element {
    let id = corp["id"].as_str().unwrap_or("").to_string();
    let name = corp["name"].as_str().unwrap_or("").to_string();
    let rut = corp["rut"].as_str().unwrap_or("").to_string();
    let active = corp["active"].as_bool().unwrap_or(true);

    let id_toggle = id.clone();
    let id2 = id.clone();
    let reps_data = use_resource({
        let cid_reps = id.clone();
        move || {
            let cid = cid_reps.clone();
            async move { client::fetch_legal_reps(&cid).await }
        }
    });

    rsx! {
        tr {
            td { class: "cell-mono", "{rut}" }
            td { "{name}" }
            td {
                SchoolCount { corporation_id: id.clone() }
            }
            td {
                if active { span { class: "badge badge-success", "Activo" } }
                else { span { class: "badge badge-danger", "Inactivo" } }
            }
            td {
                button { class: "btn btn-sm", onclick: move |_| on_toggle.call(id_toggle.clone()),
                    if is_expanded { "Contraer" } else { "Ver Colegios" }
                }
            }
        }
        if is_expanded {
            tr { class: "expand-row",
                td { colspan: "5",
                    div { class: "expand-content",
                        SchoolSection { corporation_id: id2.clone() }
                        LegalRepSection { corporation_id: id.clone(), reps: reps_data() }
                    }
                }
            }
        }
    }
}

#[component]
fn LegalRepSection(corporation_id: String, reps: Option<Result<Value, String>>) -> Element {
    let mut creating = use_signal(|| None::<String>);
    match reps {
        Some(Ok(ref data)) => {
            let list: Vec<Value> = data["legal_representatives"].as_array().cloned().unwrap_or_default();
            if list.is_empty() { return rsx! {} }
            let cid = corporation_id;
            let rows: Vec<_> = list.into_iter().map(|r| {
                let rid = r["id"].as_str().unwrap_or("").to_string();
                let rut_v = r["rut"].as_str().unwrap_or("-").to_string();
                let first = r["first_name"].as_str().unwrap_or("").to_string();
                let last = r["last_name"].as_str().unwrap_or("").to_string();
                let email_v = r["email"].as_str().unwrap_or("-").to_string();
                let phone_v = r["phone"].as_str().unwrap_or("-").to_string();
                let rep_active = r["active"].as_bool().unwrap_or(true);
                let has_email = !email_v.is_empty() && rep_active;
                let rid_clone = rid.clone();
                let cid_clone = cid.clone();
                let r_val = r.clone();
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
                        td {
                            if has_email {
                                LegalRepCreateBtn {
                                    corporation_id: cid_clone,
                                    r_val: r_val,
                                    rid: rid_clone,
                                    creating: creating,
                                }
                            }
                        }
                    }
                }
            }).collect();
            rsx! {
                h4 { "Representantes Legales (Sostenedores)" }
                p { class: "text-muted", "Los representantes legales pueden crear un Admin Global para gestionar todos los colegios de la corporación." }
                table { class: "data-table mini-table",
                    thead { tr { th { "RUT" } th { "Nombre" } th { "Email" } th { "Teléfono" } th { "Estado" } th { "Acciones" } } }
                    tbody { {rows.into_iter()} }
                }
            }
        }
        _ => rsx! {}
    }
}

#[component]
fn LegalRepCreateBtn(corporation_id: String, r_val: Value, rid: String, creating: Signal<Option<String>>) -> Element {
    let is_creating = creating() == Some(rid.clone());
    rsx! {
        button {
            class: "btn btn-xs btn-primary",
            disabled: is_creating,
            onclick: move |_| {
                creating.set(Some(rid.clone()));
                let payload = serde_json::json!({
                    "rut": r_val["rut"],
                    "name": format!("{} {}", r_val["first_name"].as_str().unwrap_or(""), r_val["last_name"].as_str().unwrap_or("")),
                    "email": r_val["email"],
                    "password": r_val["rut"].as_str().unwrap_or(""),
                    "role": "Administrador",
                    "admin_type": "global",
                    "corporation_id": corporation_id,
                });
                spawn(async move {
                    let result = client::post_json("/api/auth/register", &payload).await;
                    if result.is_err() {
                        creating.set(None);
                    }
                });
            },
            { if is_creating { "Creando..." } else { "Crear Admin Global" } }
        }
    }
}

#[component]
fn SchoolCount(corporation_id: String) -> Element {
    let schools = use_resource(move || {
        let cid = corporation_id.clone();
        async move { client::fetch_schools(Some(&cid)).await }
    });

    match schools() {
        Some(Ok(ref d)) => {
            let count = d["schools"].as_array().map(|a| a.len()).unwrap_or(0);
            rsx! { span { "{count}" } }
        }
        _ => rsx! { span { "..." } },
    }
}

#[component]
fn SchoolSection(corporation_id: String) -> Element {
    let mut show_form = use_signal(|| false);
    let mut school_name = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let cid_for_resource = corporation_id.clone();
    let mut schools_res = use_resource(move || {
        let cid = cid_for_resource.clone();
        async move { client::fetch_schools(Some(&cid)).await }
    });

    let do_create = move |_| {
        let name = school_name();
        if name.trim().is_empty() {
            return;
        }
        saving.set(true);
        let cid = corporation_id.clone();
        spawn(async move {
            let _ = client::create_school(&json!({"corporation_id": cid, "name": name})).await;
            saving.set(false);
            show_form.set(false);
            school_name.set(String::new());
            schools_res.restart();
        });
    };

    rsx! {
        div { class: "school-subtable",
            div { class: "school-header",
                strong { "Colegios" }
                button { class: "btn btn-xs btn-primary", onclick: move |_| show_form.set(!show_form()),
                    if show_form() { "Cancelar" } else { "Agregar Colegio" }
                }
            }
            {
                if show_form() {
                    rsx! {
                        div { class: "form-row inline-form",
                            input { class: "form-input", value: "{school_name}", oninput: move |e| school_name.set(e.value()), placeholder: "Nombre del colegio" }
                            button { class: "btn btn-sm btn-primary", disabled: saving(), onclick: do_create,
                                if saving() { "Creando..." } else { "Guardar" }
                            }
                        }
                    }
                } else { rsx! {} }
            }
            match schools_res() {
                Some(Ok(ref data)) => {
                    let list = data["schools"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "No hay colegios registrados" } }
                    } else {
                        rsx! {
                            table { class: "data-table mini-table",
                                thead { tr { th { "Nombre" } th { "Direcci\u{00f3}n" } th { "Tel\u{00e9}fono" } th { "Estado" } th { "Acciones" } } }
                                tbody { for s in &list {
                                    SchoolRow { school: s.clone(), on_refresh: move || schools_res.restart() }
                                }}
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
fn SchoolRow(school: Value, on_refresh: EventHandler<()>) -> Element {
    let sid = school["id"].as_str().unwrap_or("").to_string();
    let name = school["name"].as_str().unwrap_or("").to_string();
    let addr = school["address"].as_str().unwrap_or("").to_string();
    let phone = school["phone"].as_str().unwrap_or("").to_string();
    let active = school["active"].as_bool().unwrap_or(true);

    let mut editing = use_signal(|| false);
    let mut edit_name = use_signal(|| name.clone());
    let mut edit_addr = use_signal(|| addr.clone());
    let mut edit_phone = use_signal(|| if phone == "-" { String::new() } else { phone.clone() });
    let mut saving_edit = use_signal(|| false);
    let mut toggling = use_signal(|| false);
    let toggle_class = if active { "btn btn-xs btn-warning" } else { "btn btn-xs btn-success" };
    let sid_save = sid.clone();
    let sid_toggle = sid.clone();

    let do_save = move |_| {
        saving_edit.set(true);
        let id = sid_save.clone();
        let payload = json!({
            "name": edit_name(),
            "address": edit_addr(),
            "phone": edit_phone(),
        });
        spawn(async move {
            let _ = client::update_school(&id, &payload).await;
            saving_edit.set(false);
            editing.set(false);
            on_refresh.call(());
        });
    };

    let do_toggle = move |_| {
        toggling.set(true);
        let id = sid_toggle.clone();
        spawn(async move {
            let _ = client::toggle_school(&id).await;
            toggling.set(false);
            on_refresh.call(());
        });
    };

    rsx! {
        tr {
            td { "{name}" }
            td { "{addr}" }
            td { "{phone}" }
            td {
                if active {
                    span { class: "badge badge-success", "Activo" }
                } else {
                    span { class: "badge badge-danger", "Inactivo" }
                }
            }
            td {
                button { class: "btn btn-xs", onclick: move |_| {
                    editing.set(!editing());
                    if !editing() {
                        edit_name.set(name.clone());
                        edit_addr.set(addr.clone());
                        edit_phone.set(if phone == "-" { String::new() } else { phone.clone() });
                    }
                },
                    { if editing() { "Cancelar" } else { "Editar" } }
                }
                button {
                    class: "{toggle_class}",
                    disabled: toggling(),
                    onclick: do_toggle,
                    { if toggling() { "..." } else if active { "Deshabilitar" } else { "Habilitar" } }
                }
            }
        }
        if editing() {
            tr { class: "expand-row",
                td { colspan: "5",
                    div { class: "expand-content",
                        div { class: "form-row inline-form",
                            div { class: "form-group",
                                label { "Nombre:" }
                                input { class: "form-input", value: "{edit_name}", oninput: move |e| edit_name.set(e.value()) }
                            }
                            div { class: "form-group",
                                label { "Dirección:" }
                                input { class: "form-input", value: "{edit_addr}", oninput: move |e| edit_addr.set(e.value()) }
                            }
                            div { class: "form-group",
                                label { "Teléfono:" }
                                input { class: "form-input", value: "{edit_phone}", oninput: move |e| edit_phone.set(e.value()) }
                            }
                            button { class: "btn btn-sm btn-primary", disabled: saving_edit(), onclick: do_save,
                                { if saving_edit() { "Guardando..." } else { "Guardar" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

