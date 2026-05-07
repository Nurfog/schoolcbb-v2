use dioxus::prelude::*;

use crate::api::client;

fn get_subjects(data: &serde_json::Value) -> Vec<serde_json::Value> {
    data["subjects"].as_array().cloned().unwrap_or_default()
}

#[component]
pub fn GradesPage() -> Element {
    let mut selected_subject = use_signal(String::new);
    let selected_year = use_signal(|| 2025i32);
    let subjects = use_resource(|| client::fetch_subjects());

    let on_subject_change = move |evt: Event<FormData>| {
        selected_subject.set(evt.value());
    };

    rsx! {
        div { class: "page-header",
            h1 { "Calificaciones" }
            p { "Consulta de notas por asignatura" }
        }
        div { class: "page-toolbar",
            div { class: "filter-group",
                label { "Asignatura:" }
                select { value: "{selected_subject}", onchange: on_subject_change,
                    option { value: "", "Seleccione una asignatura..." }
                    {
                        match subjects() {
                            Some(Ok(data)) => {
                                let list = get_subjects(&data);
                                rsx! {
                                    for subject in list {
                                        SubjectOption { subject: subject }
                                    }
                                }
                            }
                            _ => rsx! {},
                        }
                    }
                }
            }
            div { class: "filter-group",
                label { "Año:" }
                select { value: "{selected_year}",
                    option { value: "2025", "2025" }
                    option { value: "2024", "2024" }
                }
            }
        }
        div { class: "info-card",
            div { class: "info-icon",
                svg { role: "presentation", view_box: "0 0 24 24", width: "32", height: "32",
                    circle { cx: "12", cy: "12", r: "10" }
                    line { x1: "12", y1: "16", x2: "12", y2: "12" }
                    line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
                }
            }
            div { class: "info-text",
                h3 { "Vista de Calificaciones" }
                p { "Seleccione una asignatura para ver las calificaciones de los estudiantes. Para reportes detallados use la Vista 360° desde Alumnos." }
            }
        }
    }
}

#[component]
fn SubjectOption(subject: serde_json::Value) -> Element {
    let code = subject["code"].as_str().unwrap_or("").to_string();
    let name = subject["name"].as_str().unwrap_or("").to_string();
    let display = format!("{} ({})", name, code);
    rsx! {
        option { value: "{code}", "{display}" }
    }
}
