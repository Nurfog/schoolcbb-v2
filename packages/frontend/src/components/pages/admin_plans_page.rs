use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

const MODULE_DEFS: &[(&str, &str, &[&str])] = &[
    ("dashboard", "Dashboard", &[]),
    ("students", "Gestión de Alumnos", &["view", "create", "edit", "delete", "import", "export"]),
    ("attendance", "Asistencia", &["records", "reports", "alerts", "modify"]),
    ("grades", "Calificaciones", &["view", "create", "edit", "delete", "periods", "categories", "reports"]),
    ("hr", "Recursos Humanos", &["employees", "contracts", "documents", "leaves"]),
    ("payroll", "Remuneraciones", &["view", "calculate", "export"]),
    ("finance", "Finanzas", &["fees", "payments", "scholarships"]),
    ("admission", "Admisión CRM", &["prospects", "stages", "documents", "activities", "classrooms", "metrics"]),
    ("reports", "Reportes", &["certificates", "concentrations", "final-records", "sige"]),
    ("notifications", "Centro de Mensajería", &["send", "view", "manage"]),
    ("sige", "SIGE / MINEDUC", &["export"]),
    ("complaints", "Ley Karin - Denuncias", &["view", "manage", "resolve"]),
    ("users", "Usuarios y Perfiles", &["view", "create", "edit", "delete"]),
    ("roles", "Roles y Permisos", &["view", "create", "edit", "delete", "assign"]),
    ("config", "Configuración", &["branding", "preferences", "general"]),
    ("corporations", "Multi-colegio", &["view", "create", "edit", "toggle"]),
    ("courses", "Cursos", &["view", "create", "edit", "delete"]),
    ("enrollments", "Matrículas", &["view", "create", "edit", "delete", "manage"]),
    ("subjects", "Asignaturas", &["view", "create", "edit", "delete"]),
    ("grade-levels", "Niveles", &["view", "create", "edit", "delete"]),
    ("academic-years", "Años Académicos", &["view", "create", "edit", "delete", "activate"]),
    ("classrooms", "Salas", &["view", "create", "edit", "delete"]),
    ("agenda", "Agenda Escolar", &["events", "view", "manage"]),
    ("audit", "Auditoría", &["view", "export"]),
    ("my-portal", "Portal Auto-consulta", &["view"]),
];

#[component]
pub fn AdminPlansPage() -> Element {
    let mut active_tab = use_signal(|| "plans".to_string());
    let mut plans = use_resource(|| client::admin_list_plans());
    let mut show_form = use_signal(|| false);
    let mut edit_id = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut desc = use_signal(String::new);
    let mut price_m = use_signal(|| "".to_string());
    let mut price_y = use_signal(|| "".to_string());
    let mut featured = use_signal(|| false);
    let mut is_custom = use_signal(|| false);
    let mut show_in_portal = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut mods = use_signal(|| vec![false; MODULE_DEFS.len()]);
    let mut sub_mods = use_signal(|| {
        MODULE_DEFS.iter().map(|(_, _, subs)| vec![false; subs.len()]).collect::<Vec<_>>()
    });

    let mut open_edit = move |p: Value| {
        edit_id.set(p["id"].as_str().map(|s| s.to_string()));
        name.set(p["name"].as_str().unwrap_or("").to_string());
        desc.set(p["description"].as_str().unwrap_or("").to_string());
        price_m.set(p["price_monthly"].as_f64().unwrap_or(0.0).to_string());
        price_y.set(p["price_yearly"].as_f64().unwrap_or(0.0).to_string());
        featured.set(p["featured"].as_bool().unwrap_or(false));
        is_custom.set(p["is_custom"].as_bool().unwrap_or(false));
        show_in_portal.set(p["show_in_portal"].as_bool().unwrap_or(true));

        let mut new_mods = vec![false; MODULE_DEFS.len()];
        let mut new_sub_mods: Vec<Vec<bool>> = MODULE_DEFS.iter().map(|(_, _, subs)| vec![false; subs.len()]).collect();
        if let Some(modules) = p["modules"].as_array() {
            for m in modules {
                if let Some(mk) = m["module_key"].as_str() {
                    if let Some(idx) = MODULE_DEFS.iter().position(|(k, _, _)| *k == mk) {
                        new_mods[idx] = true;
                        if let Some(subs) = m["sub_modules"].as_array() {
                            for sv in subs {
                                if let Some(sk) = sv.as_str() {
                                    if let Some(si) = MODULE_DEFS[idx].2.iter().position(|s| *s == sk) {
                                        new_sub_mods[idx][si] = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        mods.set(new_mods);
        sub_mods.set(new_sub_mods);
        show_form.set(true);
    };

    let do_save = move |_| {
        let n = name();
        let d = desc();
        let pm = price_m();
        let py = price_y();
        let f = featured();
        let ic = is_custom();
        let sp = show_in_portal();
        let eid = edit_id();
        let mvals = mods();
        let smvals = sub_mods();
        if n.trim().is_empty() { return; }
        saving.set(true);
        spawn(async move {
            let payload = json!({
                "name": n, "description": d,
                "price_monthly": pm.parse::<f64>().unwrap_or(0.0),
                "price_yearly": py.parse::<f64>().unwrap_or(0.0),
                "featured": f,
                "is_custom": ic,
                "show_in_portal": sp,
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
                        .map(|(i, (k, n, subs))| {
                            let selected: Vec<&str> = subs.iter().enumerate()
                                .filter(|(si, _)| smvals[i].get(*si).copied().unwrap_or(false))
                                .map(|(_, s)| *s)
                                .collect();
                            let mut obj = json!({"module_key": k, "module_name": n});
                            if !selected.is_empty() {
                                obj["sub_modules"] = json!(selected);
                            }
                            obj
                        })
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
            is_custom.set(false);
            show_in_portal.set(true);
            plans.restart();
        });
    };

    let do_delete = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::admin_delete_plan(&id).await;
            plans.restart();
        });
    };

    let mut toggle_mod = move |idx: usize| {
        let new_val = !mods()[idx];
        mods.with_mut(|m| m[idx] = new_val);
        if !new_val {
            sub_mods.with_mut(|sm| {
                if idx < sm.len() {
                    sm[idx].iter_mut().for_each(|s| *s = false);
                }
            });
        }
    };

    let mut toggle_sub_mod = move |mod_idx: usize, sub_idx: usize| {
        sub_mods.with_mut(|sm| {
            if mod_idx < sm.len() && sub_idx < sm[mod_idx].len() {
                sm[mod_idx][sub_idx] = !sm[mod_idx][sub_idx];
            }
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Planes y Licencias" }
            p { "Configuración de planes de precios, módulos y licencias asignadas" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == "plans" { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set("plans".to_string()),
                "Planes"
            }
            button {
                class: if active_tab() == "licenses" { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set("licenses".to_string()),
                "Licencias"
            }
        }
        div { class: "tab-content",
            match active_tab().as_str() {
                "plans" => rsx! {
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
                                div { class: "form-row",
                                    div { class: "form-group",
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: featured, oninput: move |_| featured.set(!featured()) }
                                            " Plan destacado"
                                        }
                                    }
                                    div { class: "form-group",
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: is_custom, oninput: move |_| {
                                                let new_val = !is_custom();
                                                is_custom.set(new_val);
                                                if new_val { show_in_portal.set(false); }
                                            }}
                                            " Plan Custom"
                                        }
                                    }
                                    div { class: "form-group",
                                        label { class: "checkbox-label",
                                            input { r#type: "checkbox", checked: show_in_portal, oninput: move |_| show_in_portal.set(!show_in_portal()) }
                                            " Mostrar en Portal"
                                        }
                                    }
                                }
                                div { class: "form-group",
                                    label { "Módulos incluidos:" }
                                    div { class: "checkbox-grid",
                                        {{let enabled = mods();
                                        let sm = sub_mods();
                                        let items: Vec<Element> = MODULE_DEFS.iter().enumerate().flat_map(|(i, (key, label, subs))| {
                                            let checked = enabled[i];
                                            let mut elements: Vec<Element> = Vec::new();
                                            elements.push(rsx! {
                                                label { key: "{key}", class: "checkbox-label",
                                                    input { r#type: "checkbox", checked: checked, oninput: move |_| toggle_mod(i) }
                                                    " {label}"
                                                }
                                            });
                                            if checked && !subs.is_empty() {
                                                for (j, sub) in subs.iter().enumerate() {
                                                    let j = j;
                                                    let sub_checked = sm[i].get(j).copied().unwrap_or(false);
                                                    elements.push(rsx! {
                                                        label { key: "{key}-{sub}", class: "checkbox-label sub-module",
                                                            input { r#type: "checkbox", checked: sub_checked, oninput: move |_| toggle_sub_mod(i, j) }
                                                            " {sub}"
                                                        }
                                                    });
                                                }
                                            }
                                            elements.into_iter()
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
                                                    th { "Destacado" } th { "Custom" } th { "Portal" } th { "Activo" } th { "Acciones" }
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
                                                    let pcust = p["is_custom"].as_bool().unwrap_or(false);
                                                    let pportal = p["show_in_portal"].as_bool().unwrap_or(true);
                                                    let pact = p["active"].as_bool().unwrap_or(true);
                                                    rsx! {
                                                        tr { key: "{pid}",
                                                            td { "{pname}" } td { "{pdesc}" }
                                                            td { "${pmonthly:.0}" } td { "${pyearly:.0}" }
                                                            td { if pfeat { "⭐" } else { "—" } }
                                                            td { if pcust { span { class: "badge badge-info", "Custom" } } else { "—" } }
                                                            td { if pportal { span { class: "badge badge-success", "Sí" } } else { span { class: "badge badge-warning", "No" } } }
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
                },
                "licenses" => rsx! { LicenseManager {} },
                _ => rsx! {},
            }
        }
    }
}

#[component]
fn LicenseManager() -> Element {
    let mut licenses = use_resource(|| client::admin_list_licenses());
    let mut toggling = use_signal(|| None::<String>);

    let mut do_toggle = move |id: String, current_active: bool| {
        let msg = if current_active {
            "¿Desactivar esta licencia?"
        } else {
            "¿Activar esta licencia?"
        };
        if !web_sys::window().unwrap().confirm_with_message(msg).unwrap_or(false) {
            return;
        }
        toggling.set(Some(id.clone()));
        spawn(async move {
            let _ = client::admin_update_license_status(&id, &serde_json::json!({ "active": !current_active })).await;
            toggling.set(None);
            licenses.restart();
        });
    };

    rsx! {
        div { class: "page-toolbar",
            button { class: "btn btn-secondary", onclick: move |_| licenses.restart(), "Recargar" }
        }
        div { class: "data-table-container",
            match licenses() {
                Some(Ok(data)) => {
                    let list = data["licenses"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { p { class: "empty-state", "No hay licencias asignadas" } }
                    } else {
                        rsx! {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Escuela" }
                                        th { "Plan" }
                                        th { "Vence" }
                                        th { "Máx. Alumnos" }
                                        th { "Estado" }
                                        th { "Acciones" }
                                    }
                                }
                                tbody {
                                    {list.into_iter().map(|lic| {
                                        let lid = lic["id"].as_str().unwrap_or("").to_string();
                                        let school = lic["school_name"].as_str().unwrap_or("—").to_string();
                                        let plan = lic["plan_name"].as_str().unwrap_or("—").to_string();
                                        let expires = lic["expires_at"].as_str().unwrap_or("—").to_string();
                                        let max_students = lic["max_students"].as_i64().unwrap_or(0);
                                        let active = lic["active"].as_bool().unwrap_or(false);
                                        let is_toggling = toggling() == Some(lid.clone());
                                        rsx! {
                                            tr { key: "{lid}",
                                                td { "{school}" }
                                                td { "{plan}" }
                                                td { "{expires}" }
                                                td { "{max_students}" }
                                                td {
                                                    if active {
                                                        span { class: "badge badge-success", "Activa" }
                                                    } else {
                                                        span { class: "badge badge-warning", "Inactiva" }
                                                    }
                                                }
                                                td {
                                                    button {
                                                        class: if active { "btn btn-sm btn-secondary" } else { "btn btn-sm btn-primary" },
                                                        disabled: is_toggling,
                                                        onclick: { let lid = lid.clone(); move |_| do_toggle(lid.clone(), active) },
                                                        if is_toggling { "..." } else if active { "Desactivar" } else { "Activar" }
                                                    }
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
