use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn CsvImportPage(entity_type: String) -> Element {
    let mut csv_content = use_signal(String::new);
    let mut importing = use_signal(|| false);
    let mut result = use_signal(|| None::<serde_json::Value>);

    let api_url = if entity_type == "students" {
        "/api/students/import/csv".to_string()
    } else {
        "/api/hr/employees/import/csv".to_string()
    };

    let title = if entity_type == "students" {
        "Importar Alumnos"
    } else {
        "Importar Empleados"
    };

    let description = if entity_type == "students" {
        "Sube un archivo CSV con columnas: rut, first_name, last_name, grade_level, section, email, phone, condicion, prioritario, nee"
    } else {
        "Sube un archivo CSV con columnas: rut, first_name, last_name, email, phone, position, category"
    };

    let template = if entity_type == "students" {
        "rut,first_name,last_name,grade_level,section,email,phone\n12.345.678-9,Juan,Perez,1 Basico,A,juan@c.cl,+56912345678"
    } else {
        "rut,first_name,last_name,email,phone,position,category\n12.345.678-9,Juan,Perez,juan@c.cl,+56912345678,Profesor,Docente"
    };

    let do_import = move |_| {
        if csv_content().trim().is_empty() {
            return;
        }
        importing.set(true);
        result.set(None);
        let content = csv_content();
        let url = api_url.clone();
        spawn(async move {
            let payload = serde_json::json!({ "csv_content": content });
            let resp = client::post_json(&url, &payload).await;
            importing.set(false);
            match resp {
                Ok(data) => {
                    result.set(Some(data));
                }
                Err(e) => {
                    result.set(Some(serde_json::json!({
                        "message": format!("Error: {}", e),
                        "imported": 0, "errors": [e], "total": 0
                    })));
                }
            }
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "{title}" }
            p { "{description}" }
        }
        div { class: "csv-import-container",
            div { class: "form-card",
                h3 { "Pega el contenido CSV" }
                div { class: "form-group",
                    label { "Contenido CSV (incluyendo encabezados):" }
                    textarea {
                        class: "form-input csv-textarea",
                        rows: "8",
                        placeholder: "{template}",
                        oninput: move |e| csv_content.set(e.value()),
                    }
                }
                div { class: "form-actions",
                    button { class: "btn btn-primary", disabled: importing(), onclick: do_import,
                        if importing() { "Importando..." } else { "Importar CSV" }
                    }
                }
            }
            { match result() {
                Some(res) => {
                    let _imported = res["imported"].as_i64().unwrap_or(0);
                    let _total = res["total"].as_i64().unwrap_or(0);
                    let errors = res["errors"].as_array().cloned().unwrap_or_default();
                    let msg = res["message"].as_str().unwrap_or("").to_string();
                    let error_items: Vec<Element> = errors.iter().map(|e| {
                        let text = e.as_str().unwrap_or("").to_string();
                        rsx! { div { class: "error-item", "{text}" } }
                    }).collect();
                    rsx! {
                        div { class: "import-result-card",
                            p { class: "import-summary", "{msg}" }
                            { if !error_items.is_empty() {
                                rsx! {
                                    div { class: "import-errors",
                                        h4 { "Errores ({errors.len()})" }
                                        div { class: "error-list", { error_items.into_iter() } }
                                    }
                                }
                            } else { rsx! {} }}
                            div { class: "form-actions",
                                button { class: "btn btn-secondary", onclick: move |_| result.set(None),
                                    "Limpiar resultado"
                                }
                            }
                        }
                    }
                }
                None => rsx! {}
            }}
        }
    }
}
