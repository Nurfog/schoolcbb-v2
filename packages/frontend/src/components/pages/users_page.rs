use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
fn RoleAssignRow(name: String, has_role: bool, on_assign: EventHandler<()>, on_remove: EventHandler<()>) -> Element {
    rsx! {
        div { class: "role-assign-row",
            span { "{name}" }
            if has_role {
                button { class: "btn btn-sm btn-danger", onclick: move |_| on_remove.call(()), "Quitar" }
            } else {
                button { class: "btn btn-sm btn-primary", onclick: move |_| on_assign.call(()), "Asignar" }
            }
        }
    }
}

fn get_users(data: &Value) -> Vec<Value> {
    data["users"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn UsersPage() -> Element {
    let users = use_resource(|| client::fetch_json("/api/auth/users"));
    let roles = use_resource(|| client::fetch_roles());
    let schools = use_resource(|| client::fetch_schools(None));
    let selected_user_id = use_signal(|| Option::<String>::None);
    let permissions = use_resource(|| client::fetch_my_permissions());
    let mut user_roles = use_resource(move || {
        let uid = selected_user_id();
        async move {
            match uid {
                Some(ref id) => client::fetch_user_roles(id).await,
                None => Err("none".into()),
            }
        }
    });

    let do_assign = move |uid: String, rid: String| {
        spawn({
            let uid = uid.clone();
            async move {
                let _ = client::assign_role(&uid, &rid).await;
                user_roles.restart();
            }
        });
    };

    let do_remove = move |uid: String, rid: String| {
        spawn({
            async move {
                let _ = client::remove_role(&uid, &rid).await;
                user_roles.restart();
            }
        });
    };

    let assignment_panel: Element = match permissions() {
        Some(Ok(p)) if p["can_assign_roles"].as_bool().unwrap_or(false) => {
            match selected_user_id() {
                Some(ref uid) => {
                    let school_options: Vec<Element> = match schools() {
                        Some(Ok(s)) => {
                            let list = s["schools"].as_array().cloned().unwrap_or_default();
                            list.iter().map(|sc| {
                                let sid = sc["id"].as_str().unwrap_or("").to_string();
                                let sname = sc["name"].as_str().unwrap_or("").to_string();
                                rsx! { option { value: "{sid}", "{sname}" } }
                            }).collect()
                        }
                        _ => vec![],
                    };

                    rsx! {
                        div { class: "role-assignment-panel",
                            div { class: "detail-section",
                                h4 { "Colegio" }
                                select { class: "form-input",
                                    option { value: "", "Sin asignar" }
                                    { school_options.into_iter() }
                                }
                            }
                            h4 { "Roles asignados" }
                            match user_roles() {
                                Some(Ok(data)) => {
                                    let assigned: Vec<String> = data["role_ids"].as_array()
                                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                        .unwrap_or_default();

                                    let all_roles: Vec<Value> = match roles() {
                                        Some(Ok(r)) => r["roles"].as_array().cloned().unwrap_or_default(),
                                        _ => vec![],
                                    };

                                    let role_rows: Vec<Element> = all_roles.iter().map(|r| {
                                        let rid = r["id"].as_str().unwrap_or("").to_string();
                                        let rname = r["name"].as_str().unwrap_or("").to_string();
                                        let has_role = assigned.contains(&rid);
                                        rsx! {
                                            RoleAssignRow {
                                                key: "{rid}",
                                                name: rname,
                                                has_role: has_role,
                                                on_assign: { let u = uid.clone(); let rd = rid.clone(); move |_| do_assign(u.clone(), rd.clone()) },
                                                on_remove: { let u = uid.clone(); let rd = rid.clone(); move |_| do_remove(u.clone(), rd.clone()) },
                                            }
                                        }
                                    }).collect();
                                    rsx! { { role_rows.into_iter() } }
                                }
                                _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                            }
                        }
                    }
                }
                None => rsx! { div { class: "role-assignment-panel", div { class: "empty-state", "Seleccione un usuario" } } },
            }
        }
        _ => rsx! { div { class: "role-assignment-panel", div { class: "empty-state", "Sin permisos para asignar roles" } } },
    };

    rsx! {
        div { class: "page-header",
            h1 { "Usuarios y Perfiles" }
            p { "Gestión de usuarios y asignación de roles" }
        }
        div { class: "users-layout",
            div { class: "users-table-wrap",
                table { class: "data-table",
                    thead { tr {
                        th { "Nombre" } th { "Email" } th { "RUT" } th { "Rol" } th { "Colegio" } th { "Estado" }
                    }}
                    tbody {
                        match users() {
                            Some(Ok(data)) => {
                                let list = get_users(&data);
                                if list.is_empty() {
                                    rsx! { tr { td { colspan: "6", class: "empty-state", "No hay usuarios" } } }
                                } else {
                                    let rows: Vec<Element> = list.iter().map(|user| {
                                        let uid = user["id"].as_str().unwrap_or("").to_string();
                                        let is_sel = selected_user_id.read().as_deref() == Some(&uid);
                                        let mut sel = selected_user_id.clone();
                                        rsx! {
                                            UserRow {
                                                key: "{uid}",
                                                user: user.clone(),
                                                is_selected: is_sel,
                                                on_select: move |u: String| sel.set(Some(u)),
                                            }
                                        }
                                    }).collect();
                                    rsx! { { rows.into_iter() } }
                                }
                            }
                            Some(Err(e)) => rsx! { tr { td { colspan: "5", "Error: {e}" } } },
                            None => rsx! { tr { td { colspan: "5", div { class: "loading-spinner", "Cargando..." } } } },
                        }
                    }
                }
            }
            { assignment_panel }
        }
    }
}

#[component]
fn UserRow(user: Value, is_selected: bool, on_select: EventHandler<String>) -> Element {
    let name = user["name"].as_str().unwrap_or("").to_string();
    let email = user["email"].as_str().unwrap_or("").to_string();
    let rut = user["rut"].as_str().unwrap_or("").to_string();
    let role = user["role"].as_str().unwrap_or("").to_string();
    let sid = user["school_id"].as_str().unwrap_or("").to_string();
    let school_display = if sid.is_empty() { "—".to_string() } else { sid.chars().take(8).collect::<String>() + "…" };
    let active = user["active"].as_bool().unwrap_or(true);
    let uid = user["id"].as_str().unwrap_or("").to_string();
    let row_class = if is_selected { "selected" } else { "" };

    rsx! {
        tr { class: "{row_class}",
            onclick: move |_| on_select.call(uid.clone()),
            td { class: "cell-name", "{name}" }
            td { "{email}" }
            td { span { class: "rut-badge", "{rut}" } }
            td { span { class: "role-badge", "{role}" } }
            td { span { style: "font-size: 11px; color: var(--text-secondary);", "{school_display}" } }
            td {
                if active {
                    span { class: "status-active", "Activo" }
                } else {
                    span { class: "status-inactive", "Inactivo" }
                }
            }
        }
    }
}
