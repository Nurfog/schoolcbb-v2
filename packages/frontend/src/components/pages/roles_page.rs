use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::api::client;

#[component]
pub fn RolesPage() -> Element {
    let mut roles = use_resource(|| client::fetch_roles());
    let defs = use_resource(|| client::fetch_permission_definitions());
    let mut selected_role_id = use_signal(|| Option::<String>::None);
    let mut show_create = use_signal(|| false);
    let mut new_name = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let do_create = move |_| {
        let name = new_name();
        if name.trim().is_empty() {
            return;
        }
        saving.set(true);
        spawn({
            let name = name.clone();
            async move {
                let _ = client::create_role(&json!({"name": name, "description": ""})).await;
                saving.set(false);
                show_create.set(false);
                new_name.set(String::new());
                roles.restart();
            }
        });
    };

    let do_delete_role = move |id: String| {
        if !web_sys::window().unwrap().confirm_with_message("¿Estás seguro?").unwrap_or(false) {
            return;
        }
        spawn(async move {
            let _ = client::delete_role(&id).await;
            selected_role_id.set(None);
            roles.restart();
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Roles y Permisos" }
            p { "Gestión de roles y permisos del sistema" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| show_create.set(!show_create()),
                if show_create() { "Cancelar" } else { "Nuevo Rol" }
            }
        }
        {
            if show_create() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre del nuevo rol:" }
                                input { class: "form-input", value: "{new_name}", oninput: move |e| new_name.set(e.value()), placeholder: "Ej: Inspector" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_create,
                                if saving() { "Creando..." } else { "Crear Rol" }
                            }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "roles-layout",
            match roles() {
                Some(Ok(data)) => {
                    let list = data["roles"].as_array().cloned().unwrap_or_default();
                    let sel_id = selected_role_id.read().clone();
                    let role_items: Vec<Element> = list.iter().map(|r| {
                        let rid = r["id"].as_str().unwrap_or("").to_string();
                        let is_sel = sel_id.as_deref() == Some(&rid);
                        let mut sel_signal = selected_role_id.clone();
                        rsx! {
                            RoleCard {
                                key: "{rid}",
                                role: r.clone(),
                                is_selected: is_sel,
                                on_select: move |id: String| sel_signal.set(Some(id)),
                                on_delete: do_delete_role.clone(),
                            }
                        }
                    }).collect();

                    rsx! {
                        div { class: "roles-list", { role_items.into_iter() } }
                        div { class: "perm-panel",
                            match selected_role_id() {
                                Some(ref rid) => {
                                    let role_data = list.iter().find(|r| r["id"].as_str() == Some(rid)).cloned();
                                    match role_data {
                                        Some(r) => rsx! { PermTreeView { role: r.clone(), definitions: defs() } },
                                        None => rsx! { div { class: "empty-state", "Cargando..." } },
                                    }
                                }
                                None => rsx! { div { class: "empty-state", "Seleccione un rol para configurar sus accesos" } },
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
fn RoleCard(
    role: Value,
    is_selected: bool,
    on_select: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    let name = role["name"].as_str().unwrap_or("").to_string();
    let desc = role["description"].as_str().unwrap_or("").to_string();
    let is_system = role["is_system"].as_bool().unwrap_or(false);
    let rid = role["id"].as_str().unwrap_or("").to_string();
    let sel_class = if is_selected {
        "role-item selected"
    } else {
        "role-item"
    };

    let rid1 = rid.clone();
    let rid2 = rid.clone();
    rsx! {
        div { class: "{sel_class}",
            onclick: move |_| on_select.call(rid1.clone()),
            div { class: "role-name", "{name}" }
            div { class: "role-desc", "{desc}" }
            div { class: "role-actions",
                if is_system {
                    span { class: "role-badge", "Sistema" }
                } else {
                    button { class: "btn-icon btn-sm", onclick: move |e| { e.stop_propagation(); on_delete.call(rid2.clone()); }, title: "Eliminar rol",
                        span { "🗑" }
                    }
                }
            }
        }
    }
}

#[component]
fn PermTreeView(role: Value, definitions: Option<Result<Value, String>>) -> Element {
    let rname = role["name"].as_str().unwrap_or("").to_string();
    let role_id = role["id"].as_str().unwrap_or("").to_string();
    let is_system = role["is_system"].as_bool().unwrap_or(false);
    let existing_perms = role["permissions"].as_array().cloned().unwrap_or_default();

    let mut perms_state = use_signal(|| build_perms_state(&definitions, &existing_perms));
    let modules = compute_modules(&definitions);
    let mut saving = use_signal(|| false);
    let mut saved = use_signal(|| false);

    use_effect(move || {
        let _ = definitions;
        let _ = existing_perms;
        perms_state.set(build_perms_state(&definitions, &existing_perms));
    });

    let do_save = move |_| {
        saving.set(true);
        saved.set(false);
        let payload = json!({ "permissions": perms_state.read().clone() });
        let rid = role_id.clone();
        spawn(async move {
            let _ = client::update_role_permissions(&rid, &payload).await;
            saving.set(false);
            saved.set(true);
        });
    };

    let module_elements: Vec<Element> = modules.iter().enumerate().map(|(mod_idx, (mod_name, resource_idxs))| {
        let resource_elements: Vec<Element> = resource_idxs.iter().map(|i| {
            let p = perms_state.read().get(*i).cloned().unwrap_or_default();
            let res = p["resource"].as_str().unwrap_or("").to_string();
            let cr = p["can_create"].as_bool().unwrap_or(false);
            let rd = p["can_read"].as_bool().unwrap_or(false);
            let up = p["can_update"].as_bool().unwrap_or(false);
            let dl = p["can_delete"].as_bool().unwrap_or(false);
            let idx = *i;
            rsx! {
                div { class: "perm-resource", key: "{idx}",
                    span { class: "perm-resource-name", "{res}" }
                    div { class: "perm-checks",
                        PermCheck2 { label: "Leer", checked: rd, on_toggle: move |v| { let mut s = perms_state.clone(); s.write()[idx]["can_read"] = json!(v); } }
                        PermCheck2 { label: "Crear", checked: cr, on_toggle: move |v| { let mut s = perms_state.clone(); s.write()[idx]["can_create"] = json!(v); if v { s.write()[idx]["can_read"] = json!(true); } } }
                        PermCheck2 { label: "Actualizar", checked: up, on_toggle: move |v| { let mut s = perms_state.clone(); s.write()[idx]["can_update"] = json!(v); } }
                        PermCheck2 { label: "Eliminar", checked: dl, on_toggle: move |v| { let mut s = perms_state.clone(); s.write()[idx]["can_delete"] = json!(v); } }
                    }
                }
            }
        }).collect();

        rsx! {
            div { class: "perm-module", key: "{mod_idx}",
                div { class: "perm-module-header",
                    strong { "{mod_name}" }
                }
                { resource_elements.into_iter() }
            }
        }
    }).collect();

    rsx! {
        div { class: "perm-panel-header",
            h3 { "Permisos: {rname}" }
            if is_system { span { class: "role-badge", "Sistema" } }
            div { style: "margin-left: auto; display: flex; gap: 8px;",
                button { class: "btn btn-primary btn-sm", disabled: saving(), onclick: do_save,
                    if saving() { "Guardando..." } else if saved() { "✓ Guardado" } else { "Guardar Todos" }
                }
            }
        }
        div { class: "perm-tree", { module_elements.into_iter() } }
    }
}

#[component]
fn PermCheck2(label: String, checked: bool, on_toggle: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "perm-check",
            input {
                "type": "checkbox",
                checked: checked,
                onchange: move |e| on_toggle.call(e.value() == "true" || e.value() == "on"),
            }
            span { "{label}" }
        }
    }
}

fn build_perms_state(
    definitions: &Option<Result<Value, String>>,
    existing_perms: &[Value],
) -> Vec<Value> {
    let defs = match definitions {
        Some(Ok(d)) => d,
        _ => return vec![],
    };

    let existing_map: std::collections::HashMap<String, &Value> = existing_perms
        .iter()
        .map(|p| (p["permission_id"].as_str().unwrap_or("").to_string(), p))
        .collect();

    defs["definitions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|d| {
                    let pid = d["id"].as_str().unwrap_or("").to_string();
                    let ep = existing_map.get(&pid);
                    json!({
                        "permission_id": pid,
                        "module": d["module"],
                        "resource": d["resource"],
                        "can_create": ep.and_then(|p| p["can_create"].as_bool()).unwrap_or(false),
                        "can_read": ep.and_then(|p| p["can_read"].as_bool()).unwrap_or(true),
                        "can_update": ep.and_then(|p| p["can_update"].as_bool()).unwrap_or(false),
                        "can_delete": ep.and_then(|p| p["can_delete"].as_bool()).unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn compute_modules(definitions: &Option<Result<Value, String>>) -> Vec<(String, Vec<usize>)> {
    let defs = match definitions {
        Some(Ok(d)) => d,
        _ => return vec![],
    };
    let mut modules: Vec<(String, Vec<usize>)> = Vec::new();
    if let Some(arr) = defs["definitions"].as_array() {
        for (i, d) in arr.iter().enumerate() {
            let module = d["module"].as_str().unwrap_or("").to_string();
            if let Some(existing) = modules.iter_mut().find(|(m, _)| *m == module) {
                existing.1.push(i);
            } else {
                modules.push((module, vec![i]));
            }
        }
    }
    modules
}
