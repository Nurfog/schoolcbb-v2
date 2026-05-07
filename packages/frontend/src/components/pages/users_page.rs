use dioxus::prelude::*;

use crate::api::client;

fn get_users(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data["users"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn UsersPage() -> Element {
    let users = use_resource(|| client::fetch_json("/api/auth/users"));

    rsx! {
        div { class: "page-header",
            h1 { "Usuarios y Perfiles" }
            p { "Gestión de roles, permisos y estado de cuentas" }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Nombre" }
                        th { "Email" }
                        th { "RUT" }
                        th { "Rol" }
                        th { "Estado" }
                    }
                }
                tbody {
                    match users() {
                        Some(Ok(data)) => {
                            let list = get_users(&data);
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "5", class: "empty-state", "No hay usuarios" } } }
                            } else {
                                rsx! {
                                    for user in list {
                                        UserRow { user: user }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "5", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "5", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

#[component]
fn UserRow(user: serde_json::Value) -> Element {
    let name = user["name"].as_str().unwrap_or("").to_string();
    let email = user["email"].as_str().unwrap_or("").to_string();
    let rut = user["rut"].as_str().unwrap_or("").to_string();
    let role = user["role"].as_str().unwrap_or("").to_string();
    let active = user["active"].as_bool().unwrap_or(true);

    rsx! {
        tr {
            td { class: "cell-name", "{name}" }
            td { "{email}" }
            td { span { class: "rut-badge", "{rut}" } }
            td { span { class: "role-badge", "{role}" } }
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
