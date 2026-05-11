use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn CsvMapperPage() -> Element {
    let mut csv_content = use_signal(String::new);
    let mut csv_headers = use_signal(|| Vec::<String>::new());
    let mut mappings = use_signal(|| std::collections::HashMap::<String, String>::new());
    let mut system_fields = use_signal(|| Vec::<String>::new());
    let mut preview_lines = use_signal(|| Vec::<String>::new());
    let mut mapped_csv = use_signal(String::new);
    let mut entity_type = use_signal(|| "employees".to_string());
    let mut result_msg = use_signal(String::new);
    let mut importing = use_signal(|| false);

    let system_fields_for = |entity: &str| -> Vec<String> {
        match entity {
            "students" => vec![
                "rut".into(),
                "first_name".into(),
                "last_name".into(),
                "email".into(),
                "phone".into(),
                "grade_level".into(),
                "section".into(),
                "condicion".into(),
                "prioritario".into(),
                "nee".into(),
            ],
            _ => vec![
                "rut".into(),
                "first_name".into(),
                "last_name".into(),
                "email".into(),
                "phone".into(),
                "position".into(),
                "category".into(),
            ],
        }
    };

    let parse_csv = move |_| {
        let content = csv_content();
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return;
        }

        let headers: Vec<String> = lines[0].split(',').map(|s| s.trim().to_string()).collect();
        csv_headers.set(headers.clone());

        let data_lines: Vec<String> = lines[1..].iter().map(|l| l.to_string()).collect();
        preview_lines.set(data_lines.clone());

        let fields = system_fields_for(&entity_type());
        system_fields.set(fields.clone());

        let mut auto_map = std::collections::HashMap::<String, String>::new();
        for h in &headers {
            let h_lower = h.to_lowercase();
            let matched = fields.iter().find(|f| {
                f.as_str() == h_lower.as_str()
                    || f.as_str() == h_lower.trim_start_matches('"').trim_end_matches('"')
            });
            if let Some(m) = matched {
                auto_map.insert(h.clone(), m.clone());
            }
        }
        mappings.set(auto_map);
        mapped_csv.set(String::new());
        result_msg.set(String::new());
    };

    let do_import = move |_| {
        if csv_headers().is_empty() {
            return;
        }
        importing.set(true);
        result_msg.set(String::new());

        let headers = csv_headers();
        let data = preview_lines();
        let map = mappings();
        let entity = entity_type();

        let mut new_headers: Vec<String> = Vec::new();
        for h in &headers {
            let mapped = map.get(h).map(|s| s.clone()).unwrap_or_else(|| h.clone());
            new_headers.push(mapped);
        }

        let mut csv_lines: Vec<String> = vec![new_headers.join(",")];
        for line in &data {
            csv_lines.push(line.clone());
        }
        let final_csv = csv_lines.join("\n");
        mapped_csv.set(final_csv.clone());

        let url = if entity == "students" {
            "/api/students/import/csv".to_string()
        } else {
            "/api/hr/employees/import/csv".to_string()
        };

        spawn(async move {
            let payload = serde_json::json!({ "csv_content": final_csv });
            match client::post_json(&url, &payload).await {
                Ok(data) => {
                    let msg = data["message"].as_str().unwrap_or("Importado").to_string();
                    result_msg.set(msg);
                }
                Err(e) => {
                    result_msg.set(format!("Error: {}", e));
                }
            }
            importing.set(false);
        });
    };

    let mut set_mapping = move |csv_col: String, sys_field: String| {
        let mut m = mappings();
        m.insert(csv_col, sys_field);
        mappings.set(m);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Mapper de CSV" }
            p { "Arrastra y mapea las columnas de tu archivo a los campos del sistema" }
        }
        div { class: "form-card",
            div { class: "form-row",
                div { class: "form-group",
                    label { "Tipo de importacion:" }
                    select { class: "form-input", value: "{entity_type}", onchange: move |e| entity_type.set(e.value()),
                        option { value: "employees", "Empleados" }
                        option { value: "students", "Alumnos" }
                    }
                }
            }
            div { class: "form-group",
                label { "Pega el contenido CSV (incluyendo encabezados):" }
                textarea {
                    class: "form-input csv-textarea",
                    rows: "6",
                    placeholder: "rut,nombre,apellido,email,cargo\n12.345.678-9,Juan,Perez,juan@c.cl,Profesor",
                    oninput: move |e| csv_content.set(e.value()),
                }
            }
            div { class: "form-actions",
                button { class: "btn btn-primary", onclick: parse_csv, "Analizar CSV" }
            }
        }

        { if !csv_headers().is_empty() {
            let headers = csv_headers();
            let mapping = mappings();
            let sys_fields = system_fields();
            let mapper_rows: Vec<Element> = headers.iter().map(|h| {
                let col = h.clone();
                let current = mapping.get(h).cloned().unwrap_or_default();
                let opts: Vec<Element> = sys_fields.iter().map(|f| {
                    let selected = f.as_str() == current.as_str();
                    rsx! { option { selected: "{selected}", value: "{f}", "{f}" } }
                }).collect();
                rsx! {
                    div { class: "mapper-row",
                        div { class: "mapper-source", "{col}" }
                        div { class: "mapper-arrow", "→" }
                        select {
                            class: "form-input",
                            value: "{current}",
                            onchange: move |e| set_mapping(col.clone(), e.value()),
                            option { value: "", "Ignorar columna" }
                            { opts.into_iter() }
                        }
                    }
                }
            }).collect();
            rsx! {
                div { class: "form-card",
                    h3 { "Mapeo de Columnas" }
                    p { "Para cada columna del CSV, selecciona el campo del sistema correspondiente:" }
                    div { class: "mapper-grid", { mapper_rows.into_iter() } }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: importing(), onclick: do_import,
                            if importing() { "Importando..." } else { "Importar con este mapeo" }
                        }
                    }
                }
            }
        } else { rsx! {} }}

        { if !result_msg().is_empty() {
            rsx! { div { class: "alert alert-info", "{result_msg}" } }
        } else { rsx! {} }}

        { if !mapped_csv().is_empty() {
            rsx! {
                div { class: "form-card",
                    h3 { "Vista previa del CSV generado" }
                    pre { class: "csv-preview", "{mapped_csv}" }
                }
            }
        } else { rsx! {} }}
    }
}
