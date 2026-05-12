use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

const MODULE_DEFS: &[(&str, &str)] = &[
    ("dashboard", "Dashboard"),
    ("students", "Gestión de Alumnos"),
    ("courses", "Cursos"),
    ("enrollments", "Matrículas"),
    ("subjects", "Asignaturas"),
    ("grade-levels", "Niveles"),
    ("academic-years", "Años Académicos"),
    ("classrooms", "Salas"),
    ("attendance", "Asistencia"),
    ("grades", "Calificaciones"),
    ("hr", "Recursos Humanos"),
    ("payroll", "Remuneraciones"),
    ("my-portal", "Portal Auto-consulta"),
    ("finance", "Finanzas"),
    ("admission", "Admisión CRM"),
    ("reports", "Reportes"),
    ("notifications", "Centro de Mensajería"),
    ("agenda", "Agenda Escolar"),
    ("sige", "SIGE / MINEDUC"),
    ("corporations", "Multi-colegio"),
    ("complaints", "Ley Karin — Denuncias"),
    ("users", "Usuarios y Perfiles"),
    ("roles", "Roles y Permisos"),
    ("config", "Configuración"),
];

#[component]
pub fn AdminPlansPage() -> Element {
    let mut plans = use_resource(|| client::admin_list_plans());
    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut desc = use_signal(String::new);
    let mut price_m = use_signal(|| "".to_string());
    let mut price_y = use_signal(|| "".to_string());
    let mut featured = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut mods = use_signal(|| vec![false; MODULE_DEFS.len()]);

    let mut open_edit = move |p: Value| {
        edit_id.set(p["id"].as_str().map(|s| s.to_string()));
        name.set(p["name"].as_str().unwrap_or("").to_string());
        desc.set(p["description"].as_str().unwrap_or("").to_string());
        price_m.set(p["price_monthly"].as_f64().unwrap_or(0.0).to_string());
        price_y.set(p["price_yearly"].as_f64().unwrap_or(0.0).to_string());
        featured.set(p["featured"].as_bool().unwrap_or(false));
        show_form.set(true);
    };

    let do_save = move |_| {
        let n = name();
        let d = desc();
        let pm = price_m();
        let py = price_y();
        let f = featured();
        let eid = edit_id();
        let mvals = mods();
        if n.trim().is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let payload = json!({
                "name": n, "description": d,
                "price_monthly": pm.parse::<f64>().unwrap_or(0.0),
                "price_yearly": py.parse::<f64>().unwrap_or(0.0),
                "featured": f,
            });
            let result = match &eid {
                Some(id) => client::admin_update_plan(id, &payload).await,
                None => client::admin_create_plan(&payload).await,
            };
            if let Ok(data) = result {
                let pid = eid.as_deref().unwrap_or_else(|| data["id"].as_str().unwrap_or(""));
                if !pid.is_empty() {
                    let enabled: Vec<Value> = MODULE_DEFS.iter().enumerate()
                        .filter(|(i, _)| mvals[*i])
                        .map(|(_, (k, n))| json!({"module_key": k, "module_name": n}))
                        .collect();
                    let _ = client::admin_set_plan_modules(pid, &json!({"modules": enabled})).await;
                }
            }
            saving.set(false);
            show_form.set(false);
            edit_id.set(None);
            name.set(String::new());
            desc.set(String::new());
            price_m.set("".to_string());
            price_y.set("".to_string());
            featured.set(false);
            plans.restart();
        });
    };

    let do_delete = move |id: String| {
        spawn(async move {
            let _ = client::admin_delete_plan(&id).await;
            plans.restart();
        });
    };

    let mut toggle_mod = move |idx: usize| {
        mods.with_mut(|m: &mut Vec<bool>| m[idx] = !m[idx]);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Planes" }
            p { "Configuración de planes de precios y sus módulos" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { show_form.set(!show_form()); },
                if show_form() { "Cancelar" } else { "Nuevo Plan" }
            }
        }
        {if show_form() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Nombre:" }
                            input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Ej: Básico" }
                        }
                        div { class: "form-group",
                            label { "Descripción:" }
                            input { class: "form-input", value: "{desc}", oninput: move |e| desc.set(e.value()) }
                        }
                    }
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Precio mensual:" }
                            input { class: "form-input", r#type: "number", step: "any", value: "{price_m}", oninput: move |e| price_m.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Precio anual:" }
                            input { class: "form-input", r#type: "number", step: "any", value: "{price_y}", oninput: move |e| price_y.set(e.value()) }
                        }
                    }
                    div { class: "form-group",
                        label { class: "checkbox-label",
                            input { r#type: "checkbox", checked: featured, oninput: move |_| featured.set(!featured()) }
                            " Plan destacado"
                        }
                    }
                    div { class: "form-group",
                        label { "Módulos incluidos:" }
                        div { class: "checkbox-grid",
                            {{let enabled = mods();
                            let items: Vec<_> = MODULE_DEFS.iter().enumerate().map(|(i, (key, label))| {
                                let checked = enabled[i];
                                rsx! {
                                    label { key: "{key}", class: "checkbox-label",
                                        input { r#type: "checkbox", checked: checked, oninput: move |_| toggle_mod(i) }
                                        " {label}"
                                    }
                                }
                            }).collect();
                            items.into_iter()}}
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving(), onclick: do_save,
                            if saving() { "Guardando..." } else { "Guardar Plan" }
                        }
                    }
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match plans() {
                Some(Ok(data)) => {
                    let list = data["plans"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "No hay planes configurados" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Nombre" } th { "Descripción" } th { "Mensual" } th { "Anual" }
                                        th { "Destacado" } th { "Activo" } th { "Acciones" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|p| {
                                        let pid = p["id"].as_str().unwrap_or("").to_string();
                                        let pname = p["name"].as_str().unwrap_or("").to_string();
                                        let pdesc = p["description"].as_str().unwrap_or("").to_string();
                                        let pmonthly = p["price_monthly"].as_f64().unwrap_or(0.0);
                                        let pyearly = p["price_yearly"].as_f64().unwrap_or(0.0);
                                        let pfeat = p["featured"].as_bool().unwrap_or(false);
                                        let pact = p["active"].as_bool().unwrap_or(true);
                                        rsx! {
                                            tr { key: "{pid}",
                                                td { "{pname}" } td { "{pdesc}" }
                                                td { "${pmonthly:.0}" } td { "${pyearly:.0}" }
                                                td { if pfeat { "⭐" } else { "—" } }
                                                td { if pact { span { class: "badge badge-success", "Activo" } } else { span { class: "badge badge-warning", "Inactivo" } } }
                                                td {
                                                    button { class: "btn btn-sm", onclick: { let p = p.clone(); move |_| open_edit(p.clone()) }, "Editar" }
                                                    button { class: "btn btn-sm btn-danger", onclick: { let pid = pid.clone(); move |_| do_delete(pid.clone()) }, "Eliminar" }
                                                }
                                            }
                                        }
                                    })}
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { p { class: "error-state", "Error: {e}" } },
                None => rsx! { div { class: "loading-spinner", "Cargando..." } },
            }
        }
    }
}
