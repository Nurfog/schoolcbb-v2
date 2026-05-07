use dioxus::prelude::*;

#[component]
pub fn ReportsPage() -> Element {
    rsx! {
        div { class: "page-header",
            h1 { "Reportes" }
            p { "Certificados, concentraciones de notas y actas" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                label { "Tipo:" }
                select { value: "certificate",
                    option { value: "certificate", "Certificado Alumno Regular" }
                    option { value: "concentration", "Concentración de Notas" }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select { value: "2025",
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
        }
        div { class: "info-card",
            div { class: "info-icon",
                svg { role: "presentation", view_box: "0 0 24 24", width: "32", height: "32",
                    path { d: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" }
                    polyline { points: "14 2 14 8 20 8" }
                    line { x1: "16", y1: "13", x2: "8", y2: "13" }
                    line { x1: "16", y1: "17", x2: "8", y2: "17" }
                }
            }
            div { class: "info-text",
                h3 { "Reportes Oficiales" }
                p { "Use la búsqueda global (Ctrl+K) para encontrar un estudiante, luego seleccione el tipo de reporte. Los datos se pueden imprimir como PDF desde el navegador." }
            }
        }
    }
}
