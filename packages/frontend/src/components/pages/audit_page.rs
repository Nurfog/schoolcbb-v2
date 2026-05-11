use crate::api::client;
use dioxus::prelude::*;

#[component]
pub fn AuditPage() -> Element {
    let logs = use_resource(|| client::fetch_audit_logs());

    rsx! {
        div { class: "page-header", h1 { "Auditoría" } p { "Registro de cambios en el sistema" } }
        div { class: "data-table-container",
            match logs() {
                Some(Ok(j)) => {
                    let rows: Vec<(String, String, String, String, String)> = j["audit_logs"].as_array().map(|arr| arr.iter().map(|l| {
                        (l["entity_type"].as_str().unwrap_or("").to_string(),
                         l["action"].as_str().unwrap_or("").to_string(),
                         l["entity_id"].as_str().unwrap_or("").to_string(),
                         l["user_id"].as_str().unwrap_or("-").to_string(),
                         l["created_at"].as_str().unwrap_or("").to_string())
                    }).collect()).unwrap_or_default();
                    rsx! {
                        table { class: "data-table",
                            thead { tr { th { "Entidad" } th { "Acción" } th { "ID" } th { "Usuario" } th { "Fecha" } } }
                            tbody { for (et, act, eid, uid, date) in &rows {
                                tr {
                                    td { "{et}" }
                                    td { "{act}" }
                                    td { class: "cell-mono", "{&eid[..8]}..." }
                                    td { class: "cell-mono", "{&uid[..8]}..." }
                                    td { "{date}" }
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
