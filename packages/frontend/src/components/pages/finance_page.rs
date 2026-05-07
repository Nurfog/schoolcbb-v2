use dioxus::prelude::*;

#[component]
pub fn FinancePage() -> Element {
    rsx! {
        div { class: "page-header",
            h1 { "Finanzas" }
            p { "Gestión de cuotas, pagos y becas" }
        }
        div { class: "info-card",
            div { class: "info-icon",
                svg { role: "presentation", view_box: "0 0 24 24", width: "32", height: "32",
                    line { x1: "12", y1: "1", x2: "12", y2: "23" }
                    path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" }
                }
            }
            div { class: "info-text",
                h3 { "Situación Financiera" }
                p { "Seleccione un estudiante desde la búsqueda global (Ctrl+K) o desde la lista de Alumnos para ver su situación financiera. Allí encontrará cuotas, pagos y becas registradas." }
            }
        }
    }
}
