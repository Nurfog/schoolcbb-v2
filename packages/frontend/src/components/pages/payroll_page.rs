use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn PayrollPage() -> Element {
    let now_year = 2026;
    let now_month = 5;
    let mut selected_month = use_signal(|| now_month);
    let mut selected_year = use_signal(|| now_year);
    let mut show_generate = use_signal(|| false);
    let mut gen_employee_id = use_signal(String::new);
    let mut gen_non_taxable = use_signal(|| "0".to_string());
    let mut gen_other_deductions = use_signal(|| "0".to_string());
    let mut generating = use_signal(|| false);
    let mut calc_result = use_signal(|| None::<serde_json::Value>);
    let mut exported = use_signal(|| String::new());

    let mut payrolls = use_resource(move || {
        let m = selected_month();
        let y = selected_year();
        async move { client::fetch_json(&format!("/api/hr/payroll?month={}&year={}", m, y)).await }
    });

    let employees = use_resource(|| async move { client::fetch_json("/api/hr/employees").await });

    let do_calculate = move |_| {
        if gen_employee_id().trim().is_empty() {
            return;
        }
        generating.set(true);
        let payload = json!({
            "employee_id": gen_employee_id(),
            "month": selected_month(),
            "year": selected_year(),
            "non_taxable_earnings": gen_non_taxable().parse::<f64>().unwrap_or(0.0),
            "other_deductions": gen_other_deductions().parse::<f64>().unwrap_or(0.0),
        });
        spawn(async move {
            let result = client::post_json("/api/hr/payroll/calculate", &payload).await;
            calc_result.set(result.ok());
            generating.set(false);
        });
    };

    let do_generate = move |_| {
        if gen_employee_id().trim().is_empty() {
            return;
        }
        generating.set(true);
        let payload = json!({
            "employee_id": gen_employee_id(),
            "month": selected_month(),
            "year": selected_year(),
            "non_taxable_earnings": gen_non_taxable().parse::<f64>().unwrap_or(0.0),
            "other_deductions": gen_other_deductions().parse::<f64>().unwrap_or(0.0),
        });
        spawn(async move {
            let result = client::post_json("/api/hr/payroll", &payload).await;
            generating.set(false);
            if result.is_ok() {
                show_generate.set(false);
                gen_employee_id.set(String::new());
                gen_non_taxable.set("0".to_string());
                gen_other_deductions.set("0".to_string());
                calc_result.set(None);
                payrolls.restart();
            }
        });
    };

    let do_export_lre = move |_| {
        let m = selected_month();
        let y = selected_year();
        spawn(async move {
            let result = client::fetch_json(&format!(
                "/api/hr/payroll/export/lre?month={}&year={}",
                m, y
            ))
            .await;
            match result {
                Ok(data) => {
                    exported.set(format!("LRE exportado - {} registros", data["count"]));
                    payrolls.restart();
                }
                Err(e) => {
                    exported.set(format!("Error: {}", e));
                }
            }
        });
    };

    let do_export_previred = move |_| {
        let m = selected_month();
        let y = selected_year();
        spawn(async move {
            let result = client::fetch_json(&format!(
                "/api/hr/payroll/export/previred?month={}&year={}",
                m, y
            ))
            .await;
            match result {
                Ok(data) => {
                    exported.set(format!("Previred exportado - {} registros", data["count"]));
                    payrolls.restart();
                }
                Err(e) => {
                    exported.set(format!("Error: {}", e));
                }
            }
        });
    };

    rsx! {
        div { class: "page-header",
            h1 { "Remuneraciones" }
            p { "Liquidaciones de sueldo, LRE y exportacion Previred" }
        }
        div { class: "page-toolbar",
            div { class: "form-row", style: "gap: 8px; align-items: end;",
                div { class: "form-group",
                    label { "Mes:" }
                    select { class: "form-input", value: "{selected_month}", onchange: move |e| selected_month.set(e.value().parse().unwrap_or(now_month)),
                        option { value: "1", "Enero" }
                        option { value: "2", "Febrero" }
                        option { value: "3", "Marzo" }
                        option { value: "4", "Abril" }
                        option { value: "5", "Mayo" }
                        option { value: "6", "Junio" }
                        option { value: "7", "Julio" }
                        option { value: "8", "Agosto" }
                        option { value: "9", "Septiembre" }
                        option { value: "10", "Octubre" }
                        option { value: "11", "Noviembre" }
                        option { value: "12", "Diciembre" }
                    }
                }
                div { class: "form-group",
                    label { "Anio:" }
                    input { class: "form-input", value: "{selected_year}", oninput: move |e| selected_year.set(e.value().parse().unwrap_or(now_year)), type: "number", style: "width: 100px;" }
                }
                button { class: "btn btn-primary", onclick: move |_| show_generate.set(!show_generate()),
                    if show_generate() { "Cancelar" } else { "Nueva Liquidacion" }
                }
                button { class: "btn btn-secondary", onclick: do_export_lre, "Exportar LRE" }
                button { class: "btn btn-secondary", onclick: do_export_previred, "Exportar Previred" }
            }
        }
        { if !exported().is_empty() {
            rsx! { div { class: "alert alert-success", "{exported}" } }
        } else { rsx! {} }}
        { if show_generate() {
            rsx! {
                div { class: "form-card",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Empleado:" }
                            select { class: "form-input", value: "{gen_employee_id}", onchange: move |e| gen_employee_id.set(e.value()),
                                option { value: "", "Seleccionar empleado..." }
                                { match employees() {
                                    Some(Ok(data)) => {
                                        let list = data["employees"].as_array().cloned().unwrap_or_default();
                                        let opts: Vec<Element> = list.iter().map(|emp| {
                                            let eid = emp["id"].as_str().unwrap_or("").to_string();
                                            let name = format!("{} {} - {}",
                                                emp["first_name"].as_str().unwrap_or(""),
                                                emp["last_name"].as_str().unwrap_or(""),
                                                emp["rut"].as_str().unwrap_or("")
                                            );
                                            rsx! { option { value: "{eid}", "{name}" } }
                                        }).collect();
                                        rsx! { { opts.into_iter() } }
                                    }
                                    _ => rsx! {}
                                }}
                            }
                        }
                        div { class: "form-group",
                            label { "Movilizacion/Colacion:" }
                            input { class: "form-input", value: "{gen_non_taxable}", oninput: move |e| gen_non_taxable.set(e.value()), type: "number", min: "0" }
                        }
                        div { class: "form-group",
                            label { "Otros Descuentos:" }
                            input { class: "form-input", value: "{gen_other_deductions}", oninput: move |e| gen_other_deductions.set(e.value()), type: "number", min: "0" }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-secondary", disabled: generating(), onclick: do_calculate, "Calcular Vista Previa" }
                        button { class: "btn btn-primary", disabled: generating(), onclick: do_generate,
                            if generating() { "Procesando..." } else { "Generar Liquidacion" }
                        }
                    }
                    { if let Some(ref calc) = calc_result() {
                        let net_salary = calc["net_salary"].as_f64().unwrap_or(0.0);
                        let breakdown_rows: Vec<Element> = calc["breakdown"].as_array().cloned().unwrap_or_default().iter().map(|item| {
                            let concept = item["concept"].as_str().unwrap_or("").to_string();
                            let amount = item["amount"].as_f64().unwrap_or(0.0);
                            let cat = item["category"].as_str().unwrap_or("").to_string();
                            let cls = if amount < 0.0 { "amount-negative" } else { "amount-positive" };
                            rsx! { tr {
                                td { "{concept}" }
                                td { class: "{cls}", "${amount:.0}" }
                                td { "{cat}" }
                            }}
                        }).collect();
                        rsx! {
                            div { class: "calc-result",
                                h3 { "Vista Previa" }
                                table { class: "data-table",
                                    thead { tr { th { "Concepto" } th { "Monto" } th { "Categoria" } } }
                                    tbody { { breakdown_rows.into_iter() } }
                                    tfoot { tr {
                                        td { strong { "Sueldo Liquido" } }
                                        td { strong { "${net_salary:.0}" } }
                                        td { "" }
                                    }}
                                }
                            }
                        }
                    } else { rsx! {} }}
                }
            }
        } else { rsx! {} }}
        div { class: "data-table-container",
            match payrolls() {
                Some(Ok(data)) => {
                    let list = data["payrolls"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin liquidaciones para este periodo" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|p| {
                            let name = p["employee_name"].as_str().unwrap_or("").to_string();
                            let rut = p["rut"].as_str().unwrap_or("").to_string();
                            let sb = p["salary_base"].as_f64().unwrap_or(0.0);
                            let taxable = p["taxable_income"].as_f64().unwrap_or(0.0);
                            let afp = p["afp_discount"].as_f64().unwrap_or(0.0);
                            let health = p["health_discount"].as_f64().unwrap_or(0.0);
                            let net = p["net_salary"].as_f64().unwrap_or(0.0);
                            let lre = p["lre_exported"].as_bool().unwrap_or(false);
                            let prev = p["previred_exported"].as_bool().unwrap_or(false);
                            let avatar = first_letter(&name);
                            rsx! {
                                tr {
                                    td { div { class: "employee-cell",
                                        div { class: "emp-avatar-small", "{avatar}" }
                                        span { "{name}" }
                                    }}
                                    td { "{rut}" }
                                    td { "${sb:.0}" }
                                    td { "${taxable:.0}" }
                                    td { "${afp:.0}" }
                                    td { "${health:.0}" }
                                    td { strong { "${net:.0}" } }
                                    td {
                                        if lre { span { class: "status-active", "LRE" } }
                                        else { span { class: "status-inactive", "-" } }
                                        if prev { span { class: "status-active", " PREV" } }
                                    }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Empleado" }
                                    th { "RUT" }
                                    th { "Sueldo Base" }
                                    th { "Imponible" }
                                    th { "AFP" }
                                    th { "Salud" }
                                    th { "Liquido" }
                                    th { "Exportado" }
                                }}
                                tbody { { rows.into_iter() } }
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
