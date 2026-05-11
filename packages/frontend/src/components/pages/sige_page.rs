use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn SigePage() -> Element {
    let mut tab = use_signal(|| "students".to_string());
    let students =
        use_resource(|| async move { client::fetch_json("/api/reports/sige/students").await });
    let mut year = use_signal(|| 2026);
    let mut month = use_signal(|| 5);
    let attendance = use_resource(move || async move {
        client::fetch_json(&format!(
            "/api/reports/sige/attendance/{}/{}",
            year(),
            month()
        ))
        .await
    });

    rsx! {
        div { class: "page-header",
            h1 { "SIGE — Exportacion MINEDUC" }
            p { "Datos formateados para plataforma SIGE del Ministerio de Educacion" }
        }
        div { class: "tabs-container",
            div { class: "tabs-header",
                button { class: if tab() == "students" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("students".to_string()), "Estudiantes" }
                button { class: if tab() == "attendance" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("attendance".to_string()), "Asistencia" }
            }
            div { class: "tab-content",
                match tab() {
                s if s == "students" => rsx! {
                    div { class: "data-table-container",
                        match students() {
                            Some(Ok(data)) => {
                                let csv = data["csv"].as_str().unwrap_or("Sin datos").to_string();
                                rsx! {
                                    div { class: "form-actions",
                                        button { class: "btn btn-primary", onclick: move |_| {
                                            let nav = navigator();
                                            nav.push("/api/reports/sige/students");
                                        }, "Descargar CSV" }
                                    }
                                    pre { class: "csv-preview", "{csv}" }
                                }
                            }
                            _ => rsx! { div { class: "empty-state", "Cargando..." } },
                        }
                    }
                },
                s if s == "attendance" => rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Anio:" }
                                input { class: "form-input", value: "{year}", oninput: move |e| year.set(e.value().parse().unwrap_or(2026)), type: "number" }
                            }
                            div { class: "form-group",
                                label { "Mes:" }
                                select { class: "form-input", value: "{month}", onchange: move |e| month.set(e.value().parse().unwrap_or(5)),
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
                        }
                    }
                    div { class: "data-table-container",
                        match attendance() {
                            Some(Ok(data)) => {
                                let csv = data["csv"].as_str().unwrap_or("Sin datos").to_string();
                                rsx! {
                                    pre { class: "csv-preview", "{csv}" }
                                }
                            }
                            _ => rsx! { div { class: "empty-state", "Cargando..." } },
                        }
                    }
                },
                _ => rsx! {}
                }
            }
        }
    }
}
