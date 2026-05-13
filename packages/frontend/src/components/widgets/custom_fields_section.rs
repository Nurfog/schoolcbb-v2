use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;

#[component]
pub fn CustomFieldsSection(entity_id: String, entity_type: String) -> Element {
    let entity_id_for_values = entity_id.clone();

    let definitions = use_resource(move || {
        let et = entity_type.clone();
        async move { client::fetch_custom_field_definitions(&et).await }
    });

    let values = use_resource(move || {
        let eid = entity_id_for_values.clone();
        async move { client::fetch_custom_field_values(&eid).await }
    });

    let mut field_values: Signal<Vec<(String, String)>> = use_signal(|| Vec::new());
    let mut saved = use_signal(|| false);

    use_effect(move || {
        if let (Some(Ok(v)), Some(Ok(d))) = (values(), definitions()) {
            let existing: std::collections::HashMap<String, String> = v["values"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let fid = item["field_definition_id"].as_str()?.to_string();
                            let val = item["value"].as_str().unwrap_or("").to_string();
                            Some((fid, val))
                        })
                        .collect()
                })
                .unwrap_or_default();

            let defs = d["definitions"].as_array().cloned().unwrap_or_default();
            let new_vals: Vec<(String, String)> = defs
                .iter()
                .map(|d| {
                    let fid = d["id"].as_str().unwrap_or("").to_string();
                    let val = existing.get(&fid).cloned().unwrap_or_default();
                    (fid, val)
                })
                .collect();
            field_values.set(new_vals);
        }
    });

    let on_save = move |_| {
        saved.set(false);
        let vals: Vec<Value> = field_values
            .read()
            .iter()
            .map(|(fid, val)| serde_json::json!({"field_definition_id": fid, "value": val}))
            .collect();
        let payload = serde_json::json!({"values": vals});
        let eid = entity_id.clone();
        spawn(async move {
            let _ = client::save_custom_field_values(&eid, &payload).await;
            saved.set(true);
        });
    };

    rsx! {
        div { class: "detail-section",
            h4 { "Campos Personalizados" }
            {
                match definitions() {
                    Some(Ok(d)) => {
                        let defs = d["definitions"].as_array().cloned().unwrap_or_default();
                        if defs.is_empty() {
                            rsx! { p { style: "color: #999; font-size: 12px;", "No hay campos personalizados definidos" } }
                        } else {
                            rsx! {
                                RenderCustomFields { defs: defs.clone(), field_values: field_values }
                                button { class: "btn btn-primary btn-sm", style: "margin-top: 8px;", onclick: on_save,
                                    if saved() { "Guardado!" } else { "Guardar Campos" }
                                }
                            }
                        }
                    }
                    _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
                }
            }
        }
    }
}

#[component]
fn RenderCustomFields(defs: Vec<Value>, field_values: Signal<Vec<(String, String)>>) -> Element {
    let mut rows: Vec<Element> = Vec::new();
    for (i, def) in defs.iter().enumerate() {
        let fname = def["field_name"].as_str().unwrap_or("").to_string();
        let ftype = def["field_type"].as_str().unwrap_or("text").to_string();
        let options = def["options"].as_array().cloned().unwrap_or_default();

        let input_element: Element = if ftype == "select" && !options.is_empty() {
            let opt_elements: Vec<Element> = options
                .iter()
                .map(|opt| {
                    let opt_val = opt.as_str().unwrap_or("").to_string();
                    rsx! { option { value: "{opt_val}", "{opt_val}" } }
                })
                .collect();
            rsx! {
                select {
                    class: "form-input",
                    value: "{field_values.read().get(i).map(|(_, v)| v.clone()).unwrap_or_default()}",
                    onchange: move |e| {
                        let mut fv = field_values.write();
                        if let Some(item) = fv.get_mut(i) {
                            item.1 = e.value();
                        }
                    },
                    option { value: "", "Seleccionar..." }
                    { opt_elements.into_iter() }
                }
            }
        } else {
            rsx! {
                input {
                    class: "form-input",
                    value: "{field_values.read().get(i).map(|(_, v)| v.clone()).unwrap_or_default()}",
                    oninput: move |e| {
                        let mut fv = field_values.write();
                        if let Some(item) = fv.get_mut(i) {
                            item.1 = e.value();
                        }
                    },
                    placeholder: "{fname}",
                }
            }
        };

        rows.push(rsx! {
            div { class: "form-group",
                label { "{fname}" }
                { input_element }
            }
        });
    }

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 8px;", { rows.into_iter() } }
    }
}
