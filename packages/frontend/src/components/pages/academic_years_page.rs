use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn AcademicYearsPage() -> Element {
    let mut years = use_resource(|| client::fetch_academic_years());
    let mut show_form = use_signal(|| false);
    let mut year = use_signal(|| 2026i32);
    let mut name = use_signal(|| String::new());
    let mut is_active = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut clone_from = use_signal(|| 2025i32);
    let mut clone_to = use_signal(|| 2026i32);
    let mut clone_msg = use_signal(|| None::<String>);
    let mut show_clone = use_signal(|| false);

    let mut reset_form = move || {
        year.set(2026);
        name.set(String::new());
        is_active.set(false);
        show_form.set(false);
    };

    let do_create = move |_| {
        saving.set(true);
        let payload = serde_json::json!({
            "year": year(),
            "name": name(),
            "is_active": is_active(),
        });
        spawn(async move {
            let _ = client::create_academic_year(&payload).await;
            saving.set(false);
            reset_form();
            years.restart();
        });
    };

    let do_activate = move |id: String| {
        spawn(async move {
            let _ = client::activate_academic_year(&id).await;
            years.restart();
        });
    };

    let do_delete = move |id: String| {
        spawn(async move {
            let _ = client::delete_academic_year(&id).await;
            years.restart();
        });
    };

    let do_clone = move |_| {
        saving.set(true);
        clone_msg.set(None);
        let payload = serde_json::json!({
            "from_year": clone_from(),
            "to_year": clone_to(),
        });
        spawn(async move {
            let res = client::clone_academic_year(&payload).await;
            saving.set(false);
            clone_msg.set(Some(match res {
                Ok(j) => j["message"].as_str().unwrap_or("OK").to_string(),
                Err(e) => format!("Error: {}", e),
            }));
            years.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Años Académicos" }
            p { "Gestión de años escolares y clonación de estructura académica" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); }, "Nuevo Año" }
            button { class: "btn", style: "margin-left: 8px;", onclick: move |_| show_clone.set(!show_clone()), "Clonar Estructura" }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Año:" }
                                input { class: "form-input", value: "{year}", oninput: move |e| { if let Ok(v) = e.value().parse() { year.set(v); } }, type: "number", min: "2020", max: "2035" }
                            }
                            div { class: "form-group",
                                label { "Nombre:" }
                                input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Año Escolar 2026" }
                            }
                            div { class: "form-group",
                                label { "Activo:" }
                                input { class: "form-input", r#type: "checkbox", checked: is_active, oninput: move |e| is_active.set(e.value() == "true") }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_create, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        {
            if show_clone() {
                rsx! {
                    div { class: "form-card",
                        h3 { "Clonar Estructura Académica" }
                        p { "Copia las asignaturas de un año a otro para agilizar la configuración anual." }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Desde año:" }
                                input { class: "form-input", value: "{clone_from}", oninput: move |e| { if let Ok(v) = e.value().parse() { clone_from.set(v); } }, type: "number" }
                            }
                            div { class: "form-group",
                                label { "Hacia año:" }
                                input { class: "form-input", value: "{clone_to}", oninput: move |e| { if let Ok(v) = e.value().parse() { clone_to.set(v); } }, type: "number" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_clone, if saving() { "Clonando..." } else { "Clonar" } }
                        }
                        {
                            match clone_msg() {
                                Some(msg) => rsx! { div { class: "info-card", p { "{msg}" } } },
                                None => rsx! {},
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            {
                match years() {
                    Some(Ok(j)) => {
                        let rows: Vec<(String, i64, String, bool)> = j["years"].as_array().map(|arr| {
                            arr.iter().map(|y| {
                                let id = y["id"].as_str().unwrap_or("").to_string();
                                let year_num = y["year"].as_i64().unwrap_or(0);
                                let name = y["name"].as_str().unwrap_or("").to_string();
                                let active = y["is_active"].as_bool().unwrap_or(false);
                                (id, year_num, name, active)
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Año" }
                                    th { "Nombre" }
                                    th { "Estado" }
                                    th { "Acciones" }
                                }}
                                tbody { for (yid, ynum, yname, active) in &rows {
                                    tr {
                                        td { style: "font-weight: bold;", "{ynum}" }
                                        td { "{yname}" }
                                        td { if *active { span { class: "status-active", "Activo" } } else { span { class: "status-inactive", "Inactivo" } } }
                                        td {
                                            if !active {
                                                button { class: "btn btn-sm btn-success", onclick: { let id = yid.clone(); move |_| do_activate(id.clone()) }, "Activar" }
                                                button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let id = yid.clone(); move |_| do_delete(id.clone()) }, "Eliminar" }
                                            }
                                        }
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
}
