use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn HrPage() -> Element {
    let mut employees = use_resource(|| client::fetch_json("/api/hr/employees"));
    let mut search = use_signal(String::new);
    let mut show_form = use_signal(|| false);
    let mut rut = use_signal(String::new);
    let mut first_name = use_signal(String::new);
    let mut last_name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut category = use_signal(|| "Docente".to_string());
    let mut hire_date = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let do_create = move |_| {
        if rut().trim().is_empty()
            || first_name().trim().is_empty()
            || last_name().trim().is_empty()
        {
            return;
        }
        saving.set(true);
        let payload = json!({
            "rut": rut(),
            "first_name": first_name(),
            "last_name": last_name(),
            "email": email(),
            "phone": phone(),
            "position": position(),
            "category": category(),
            "hire_date": hire_date(),
        });
        spawn(async move {
            let _ = client::post_json("/api/hr/employees", &payload).await;
            saving.set(false);
            show_form.set(false);
            rut.set(String::new());
            first_name.set(String::new());
            last_name.set(String::new());
            email.set(String::new());
            phone.set(String::new());
            position.set(String::new());
            hire_date.set(String::new());
            employees.restart();
        });
    };

    let do_search = move |e: FormEvent| {
        let q = e.value();
        search.set(q.clone());
        if q.len() >= 2 || q.is_empty() {
            employees.restart();
        }
    };

    rsx! {
        div { class: "page-header",
            h1 { "Recursos Humanos" }
            p { "Gestión de empleados, contratos y documentación" }
        }
        div { class: "page-toolbar",
            input { class: "search-input", value: "{search}", oninput: do_search, placeholder: "Buscar por RUT, nombre..." }
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Nuevo Empleado" }
            }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "RUT:" }
                                input { class: "form-input", value: "{rut}", oninput: move |e| rut.set(e.value()), placeholder: "12.345.678-9" }
                            }
                            div { class: "form-group",
                                label { "Categoría:" }
                                select { class: "form-input", value: "{category}", onchange: move |e| category.set(e.value()),
                                    option { value: "Docente", "Docente" }
                                    option { value: "Directivo", "Directivo" }
                                    option { value: "Administrativo", "Administrativo" }
                                    option { value: "Asistente", "Asistente" }
                                    option { value: "Enfermeria", "Enfermería" }
                                    option { value: "Psicologia", "Psicología" }
                                    option { value: "Psicopedagogia", "Psicopedagogía" }
                                    option { value: "Auxiliar", "Auxiliar" }
                                    option { value: "Otro", "Otro" }
                                }
                            }
                        }
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
                                label { "Email:" }
                                input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "juan@colegio.cl" }
                            }
                            div { class: "form-group",
                                label { "Teléfono:" }
                                input { class: "form-input", value: "{phone}", oninput: move |e| phone.set(e.value()), placeholder: "+56 9 1234 5678" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Cargo:" }
                                input { class: "form-input", value: "{position}", oninput: move |e| position.set(e.value()), placeholder: "Profesor de Matemáticas" }
                            }
                            div { class: "form-group",
                                label { "Fecha de Contratación:" }
                                input { class: "form-input", value: "{hire_date}", oninput: move |e| hire_date.set(e.value()), type: "date" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_create,
                                if saving() { "Guardando..." } else { "Crear Empleado" }
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match employees() {
                Some(Ok(data)) => {
                    let list = data["employees"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "No hay empleados registrados" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|emp| {
                            let id = emp["id"].as_str().unwrap_or("").to_string();
                            let rut = emp["rut"].as_str().unwrap_or("").to_string();
                            let name = format!("{} {}",
                                emp["first_name"].as_str().unwrap_or(""),
                                emp["last_name"].as_str().unwrap_or("")
                            );
                            let cat = emp["category"].as_str().unwrap_or("—").to_string();
                            let pos = emp["position"].as_str().unwrap_or("—").to_string();
                            let active = emp["active"].as_bool().unwrap_or(true);
                            let avatar = first_letter(&name);
                            rsx! {
                                tr { class: "clickable-row", onclick: move |_| {
                                    let nav = navigator();
                                    nav.push(format!("/hr/{}", id));
                                },
                                    td { div { class: "employee-cell",
                                        div { class: "emp-avatar-small", "{avatar}" }
                                        span { class: "rut-badge", "{rut}" }
                                    }}
                                    td { "{name}" }
                                    td { span { class: "role-badge", "{cat}" } }
                                    td { "{pos}" }
                                    td {
                                        if active {
                                            span { class: "status-active", "Activo" }
                                        } else {
                                            span { class: "status-inactive", "Inactivo" }
                                        }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "RUT" }
                                    th { "Nombre" }
                                    th { "Categoría" }
                                    th { "Cargo" }
                                    th { "Estado" }
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
