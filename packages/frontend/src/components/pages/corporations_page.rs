use dioxus::prelude::*;
use serde_json::{json, Value};

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
        if name.trim().is_empty() || rut.trim().is_empty() { return; }
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
                    }
                }
            }
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
        if name.trim().is_empty() { return; }
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
                                thead { tr { th { "Nombre" } th { "Direcci\u{00f3}n" } th { "Tel\u{00e9}fono" } th { "Estado" } } }
                                tbody { for s in &list {
                                    SchoolRow { school: s.clone() }
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
fn SchoolRow(school: Value) -> Element {
    let name = school["name"].as_str().unwrap_or("").to_string();
    let addr = school["address"].as_str().unwrap_or("-").to_string();
    let phone = school["phone"].as_str().unwrap_or("-").to_string();
    let active = school["active"].as_bool().unwrap_or(true);

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
        }
    }
}
