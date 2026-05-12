use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn ClientPortalPage() -> Element {
    let license = use_resource(|| client::fetch_json("/api/client/license"));
    let payments = use_resource(|| client::fetch_json("/api/client/payments"));
    let billing = use_resource(|| client::fetch_json("/api/client/billing-info"));

    let mut children: Vec<Element> = Vec::new();

    children.push(rsx! {
        div { class: "page-header",
            h1 { "Portal de Licencia" }
            p { "Gestiona tu licencia, pagos y datos de facturación" }
        }
    });

    match license() {
        Some(Ok(data)) => {
            let lic = &data["license"];
            let days = lic["days_remaining"].as_i64().unwrap_or(0);
            let status = lic["status"].as_str().unwrap_or("");
            let plan = lic["plan_name"].as_str().unwrap_or("");
            let status_class = if status == "active" { "badge-success" } else { "badge-warning" };
            let days_class = if days < 30 { "text-warning" } else { "text-success" };
            let modules = data["modules"].as_array().cloned().unwrap_or_default();
            let module_items: Vec<Element> = modules.into_iter().map(|m| {
                let name = m["name"].as_str().unwrap_or("").to_string();
                let included = m["included"].as_bool().unwrap_or(false);
                let cls = if included { "module-included" } else { "module-excluded" };
                let icon = if included { "✅" } else { "❌" };
                rsx! {
                    div { key: "{name}", class: "{cls}",
                        span { "{icon}" }
                        span { "{name}" }
                    }
                }
            }).collect();
            children.push(rsx! {
                div { class: "card",
                    h3 { "Plan Actual" }
                    div { class: "plan-badge", "{plan}" }
                    p { class: "{days_class}", "Días restantes: {days}" }
                    p { "Estado: " span { class: "badge {status_class}", "{status}" } }
                    div { class: "module-list",
                        h4 { "Módulos incluidos:" }
                        {module_items.into_iter()}
                    }
                }
            });
        }
        _ => children.push(rsx! { div { class: "loading-spinner", "Cargando licencia..." } }),
    }

    match payments() {
        Some(Ok(data)) => {
            let list = data["payments"].as_array().cloned().unwrap_or_default();
            let payment_rows: Vec<Element> = list.into_iter().map(|p| {
                let id = p["id"].as_str().unwrap_or("").to_string();
                let amount = p["amount"].as_f64().unwrap_or(0.0);
                let amount_str = format!("${:.0}", amount);
                let method = p["payment_method"].as_str().unwrap_or("").to_string();
                let tx = p["transaction_id"].as_str().unwrap_or("-").to_string();
                let paid = p["paid_at"].as_str().unwrap_or("").to_string();
                rsx! {
                    tr { key: "{id}",
                        td { "{paid}" }
                        td { "{amount_str}" }
                        td { "{method}" }
                        td { "{tx}" }
                    }
                }
            }).collect();
            children.push(rsx! {
                div { class: "card", style: "margin-top: 1rem;",
                    h3 { "Historial de Pagos" }
                    if payment_rows.is_empty() {
                        p { "Sin pagos registrados" }
                    } else {
                        table { class: "data-table",
                            thead { tr {
                                th { "Fecha" } th { "Monto" } th { "Método" } th { "Transacción" }
                            } }
                            tbody { {payment_rows.into_iter()} }
                        }
                    }
                }
            });
        }
        _ => children.push(rsx! { div { class: "loading-spinner", "Cargando pagos..." } }),
    }

    match billing() {
        Some(Ok(data)) => {
            let rut = data["rut"].as_str().unwrap_or("").to_string();
            let name = data["business_name"].as_str().unwrap_or("").to_string();
            children.push(rsx! {
                div { class: "card", style: "margin-top: 1rem;",
                    h3 { "Datos de Facturación" }
                    p { "RUT: {rut}" }
                    p { "Razón Social: {name}" }
                }
            });
        }
        _ => children.push(rsx! { div { class: "loading-spinner", "Cargando datos..." } }),
    }

    rsx! { {children.into_iter()} }
}
