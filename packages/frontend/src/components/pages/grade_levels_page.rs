use crate::api::client;
use dioxus::prelude::*;

#[component]
pub fn GradeLevelsPage() -> Element {
    let mut levels = use_resource(|| client::fetch_grade_levels());
    let mut code = use_signal(String::new);
    let mut name = use_signal(String::new);
    let mut plan = use_signal(String::new);
    let mut sort_order = use_signal(|| 0i32);
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut reset_form = move || {
        code.set(String::new());
        name.set(String::new());
        plan.set(String::new());
        sort_order.set(0);
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let payload = serde_json::json!({ "code": code(), "name": name(), "plan": if plan().is_empty() { serde_json::Value::Null } else { serde_json::json!(plan()) }, "sort_order": sort_order() });
        let is_edit = editing_id().is_some();
        let ep = if let Some(ref id) = editing_id() {
            format!("/api/academic/grade-levels/{}", id)
        } else {
            "/api/academic/grade-levels".to_string()
        };
        spawn(async move {
            if is_edit {
                let _ = client::update_grade_level(
                    &ep.replace("/api/academic/grade-levels/", ""),
                    &payload,
                )
                .await;
            } else {
                let _ = client::create_grade_level(&payload).await;
            }
            saving.set(false);
            reset_form();
            levels.restart();
        });
    };

    let do_delete = move |id: String| {
        spawn(async move {
            let _ = client::delete_grade_level(&id).await;
            levels.restart();
        });
    };

    rsx! {
        div { class: "page-header", h1 { "Niveles" } p { "Gestión de niveles educativos y planes" } }
        div { class: "page-toolbar", button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); }, "Nuevo Nivel" } }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group", label { "Código:" } input { class: "form-input", value: "{code}", oninput: move |e| code.set(e.value()) } }
                            div { class: "form-group", label { "Nombre:" } input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()) } }
                        }
                        div { class: "form-row",
                            div { class: "form-group", label { "Plan:" } select { class: "form-input", value: "{plan}", oninput: move |e| plan.set(e.value()),
                                option { value: "", "Plan Común" } option { value: "HC", "Científico-Humanista" } option { value: "TP", "Técnico-Profesional" } option { value: "Artístico", "Artístico" }
                            } }
                            div { class: "form-group", label { "Orden:" } input { class: "form-input", value: "{sort_order}", oninput: move |e| { if let Ok(v) = e.value().parse() { sort_order.set(v); } }, type: "number" } }
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
            match levels() {
                Some(Ok(j)) => {
                    let rows: Vec<(String, String, String, String, i32)> = j["levels"].as_array().map(|arr| arr.iter().map(|l| {
                        (l["id"].as_str().unwrap_or("").to_string(), l["code"].as_str().unwrap_or("").to_string(), l["name"].as_str().unwrap_or("").to_string(), l["plan"].as_str().unwrap_or("").to_string(), l["sort_order"].as_i64().unwrap_or(0) as i32)
                    }).collect()).unwrap_or_default();
                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "Código" } th { "Nombre" } th { "Plan" } th { "Orden" } th { "Acciones" } } }
                            tbody { for (id, c, n, p, o) in &rows {
                                tr {
                                    td { class: "cell-mono", "{c}" }
                                    td { "{n}" }
                                    td { if p.is_empty() { "-" } else { "{p}" } }
                                    td { "{o}" }
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
