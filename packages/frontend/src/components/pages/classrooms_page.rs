use dioxus::prelude::*;
use crate::api::client;

#[component]
pub fn ClassroomsPage() -> Element {
    let mut rooms = use_resource(|| client::fetch_classrooms());
    let mut name = use_signal(String::new);
    let mut capacity = use_signal(|| 30i32);
    let mut location = use_signal(String::new);
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut reset_form = move || { name.set(String::new()); capacity.set(30); location.set(String::new()); editing_id.set(None); show_form.set(false); };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({ "name": name(), "capacity": capacity(), "location": if location().is_empty() { serde_json::Value::Null } else { serde_json::json!(location()) } });
        let is_edit = editing_id().is_some();
        let ep = if let Some(ref id) = editing_id() { format!("/api/admission/classrooms/{}", id) } else { "/api/admission/classrooms".to_string() };
        spawn(async move {
            if is_edit { let _ = client::update_classroom(&editing_id().unwrap_or_default(), &payload).await; }
            else { let _ = client::create_classroom(&payload).await; }
            saving.set(false); reset_form(); rooms.restart();
        });
    };

    let do_delete = move |id: String| { spawn(async move { let _ = client::delete_classroom(&id).await; rooms.restart(); }); };

    rsx! {
        div { class: "page-header", h1 { "Salas" } p { "Gestión de salas y aforo máximo" } }
        div { class: "page-toolbar", button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); }, "Nueva Sala" } }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group", label { "Nombre:" } input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()), placeholder: "Sala 101" } }
                            div { class: "form-group", label { "Capacidad:" } input { class: "form-input", value: "{capacity}", oninput: move |e| { if let Ok(v) = e.value().parse() { capacity.set(v); } }, type: "number", min: "1" } }
                        }
                        div { class: "form-row",
                            div { class: "form-group", label { "Ubicación:" } input { class: "form-input", value: "{location}", oninput: move |e| location.set(e.value()), placeholder: "Piso 1, Edificio A" } }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            match rooms() {
                Some(Ok(j)) => {
                    let rows: Vec<(String, String, i32, String)> = j["classrooms"].as_array().map(|arr| arr.iter().map(|r| {
                        (r["id"].as_str().unwrap_or("").to_string(), r["name"].as_str().unwrap_or("").to_string(), r["capacity"].as_i64().unwrap_or(0) as i32, r["location"].as_str().unwrap_or("").to_string())
                    }).collect()).unwrap_or_default();
                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "Nombre" } th { "Capacidad" } th { "Ubicación" } th { "Acciones" } } }
                            tbody { for (id, n, cap, loc) in &rows {
                                tr {
                                    td { "{n}" }
                                    td { "{cap}" }
                                    td { if loc.is_empty() { "-" } else { "{loc}" } }
                                    td { button { class: "btn btn-sm btn-danger", onclick: { let i = id.clone(); move |_| do_delete(i.clone()) }, "Eliminar" } }
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
