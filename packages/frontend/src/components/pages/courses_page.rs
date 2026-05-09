use dioxus::prelude::*;
use serde_json::Value;

use crate::api::client;
use crate::components::widgets::searchable_select::SearchableSelect;

fn needs_plan(level: &str) -> bool {
    level == "3° Medio" || level == "4° Medio"
}

#[component]
pub fn CoursesPage() -> Element {
    let mut courses = use_resource(|| client::fetch_json("/api/courses"));
    let mut name = use_signal(|| "".to_string());
    let mut subject = use_signal(|| "".to_string());
    let mut grade_level = use_signal(|| "".to_string());
    let mut plan = use_signal(|| "".to_string());
    let mut section = use_signal(|| "".to_string());
    let mut teacher_id = use_signal(|| "".to_string());
    let mut editing_id = use_signal(|| None::<String>);
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    let mut reset_form = move || {
        name.set("".to_string());
        subject.set("".to_string());
        grade_level.set("".to_string());
        plan.set("".to_string());
        section.set("".to_string());
        teacher_id.set("".to_string());
        editing_id.set(None);
        show_form.set(false);
    };

    let do_save = move |_| {
        saving.set(true);
        let mut payload = serde_json::json!({
            "name": name(),
            "subject": subject(),
            "grade_level": grade_level(),
            "section": section(),
            "teacher_id": teacher_id(),
        });
        if needs_plan(&grade_level()) && !plan().is_empty() {
            payload["plan"] = serde_json::json!(plan());
        }
        let is_edit = editing_id().is_some();
        let endpoint = if let Some(ref id) = editing_id() {
            format!("/api/courses/{}", id)
        } else {
            "/api/courses".to_string()
        };
        spawn(async move {
            if is_edit {
                let _ = client::put_json(&endpoint, &payload).await;
            } else {
                let _ = client::post_json(&endpoint, &payload).await;
            }
            saving.set(false);
            reset_form();
            courses.restart();
        });
    };

    let do_delete = move |id: String| {
        spawn(async move {
            let _ = client::post_json(&format!("/api/courses/{}", id), &serde_json::json!({})).await;
            courses.restart();
        });
    };

    let do_edit = move |c: Value| {
        name.set(c["name"].as_str().unwrap_or("").to_string());
        subject.set(c["subject"].as_str().unwrap_or("").to_string());
        grade_level.set(c["grade_level"].as_str().unwrap_or("").to_string());
        plan.set(c["plan"].as_str().unwrap_or("").to_string());
        section.set(c["section"].as_str().unwrap_or("").to_string());
        teacher_id.set(c["teacher_id"].as_str().unwrap_or("").to_string());
        editing_id.set(c["id"].as_str().map(|s| s.to_string()));
        show_form.set(true);
    };

    rsx! {
        div { class: "page-header",
            h1 { "Cursos" }
            p { "Gestión de cursos y asignación de profesores jefe" }
        }
        div { class: "page-toolbar",
            button { class: "btn btn-primary", onclick: move |_| { reset_form(); show_form.set(true); },
                "Nuevo Curso"
            }
        }
        if show_form() {
            div { class: "card form-card",
                h3 { if editing_id().is_some() { "Editar Curso" } else { "Nuevo Curso" } }
                div { class: "form-grid",
                    div { class: "field",
                        label { "Nombre del Curso" }
                        input { class: "form-input", placeholder: "1° Medio A", value: "{name}",
                            oninput: move |e| name.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "Asignatura Principal" }
                        input { class: "form-input", placeholder: "Lenguaje", value: "{subject}",
                            oninput: move |e| subject.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "Nivel" }
                        select { class: "form-input", value: "{grade_level}",
                            oninput: move |e| {
                                grade_level.set(e.value());
                                plan.set(String::new());
                            },
                            option { value: "", "Seleccionar..." }
                            option { value: "Sala Cuna", "Sala Cuna" }
                            option { value: "Medio Menor", "Medio Menor" }
                            option { value: "Medio Mayor", "Medio Mayor" }
                            option { value: "Pre-kinder", "Pre-kinder" }
                            option { value: "Kinder", "Kinder" }
                            option { value: "1° Básico", "1° Básico" }
                            option { value: "2° Básico", "2° Básico" }
                            option { value: "3° Básico", "3° Básico" }
                            option { value: "4° Básico", "4° Básico" }
                            option { value: "5° Básico", "5° Básico" }
                            option { value: "6° Básico", "6° Básico" }
                            option { value: "7° Básico", "7° Básico" }
                            option { value: "8° Básico", "8° Básico" }
                            option { value: "1° Medio", "1° Medio" }
                            option { value: "2° Medio", "2° Medio" }
                            option { value: "3° Medio", "3° Medio" }
                            option { value: "4° Medio", "4° Medio" }
                        }
                    }
                    {
                        if needs_plan(&grade_level()) {
                            rsx! {
                                div { class: "field",
                                    label { "Plan" }
                                    select { class: "form-input", value: "{plan}",
                                        oninput: move |e| plan.set(e.value()),
                                        option { value: "", "Seleccionar..." }
                                        option { value: "HC", "Científico-Humanista" }
                                        option { value: "TP", "Técnico-Profesional" }
                                        option { value: "Artístico", "Artístico" }
                                    }
                                }
                            }
                        } else { rsx! {} }
                    }
                    div { class: "field",
                        label { "Sección" }
                        input { class: "form-input", placeholder: "A", value: "{section}",
                            oninput: move |e| section.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "Profesor Jefe" }
                        SearchableSelect {
                            fetch_url: "/api/auth/users".to_string(),
                            results_key: "users".to_string(),
                            label_key: "name".to_string(),
                            value_key: "id".to_string(),
                            placeholder: "Buscar profesor...",
                            on_select: move |id| teacher_id.set(id),
                        }
                    }
                }
                div { class: "form-actions",
                    button { class: "btn-secondary", onclick: move |_| reset_form(), "Cancelar" }
                    button { class: "btn-primary", onclick: do_save, disabled: saving(),
                        if saving() { "Guardando..." } else { "Guardar" }
                    }
                }
            }
        }
        div { class: "data-table-container",
            table { class: "data-table",
                thead {
                    tr {
                        th { "Nombre" }
                        th { "Asignatura" }
                        th { "Nivel" }
                        th { "Plan" }
                        th { "Sección" }
                        th { "Profesor ID" }
                        th { "Acciones" }
                    }
                }
                tbody {
                    match courses() {
                        Some(Ok(data)) => {
                            let list = data["courses"].as_array().cloned().unwrap_or_default();
                            if list.is_empty() {
                                rsx! { tr { td { colspan: "7", class: "empty-state", "No hay cursos configurados" } } }
                            } else {
                                rsx! {
                                    for c in list {
                                        CourseRow { course: c, on_edit: do_edit.clone(), on_delete: do_delete.clone() }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => rsx! { tr { td { colspan: "7", class: "empty-state", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "7", class: "empty-state", div { class: "loading-spinner", "Cargando..." } } } },
                    }
                }
            }
        }
    }
}

#[component]
fn CourseRow(course: Value, on_edit: EventHandler<Value>, on_delete: EventHandler<String>) -> Element {
    let id = course["id"].as_str().unwrap_or("").to_string();
    let name = course["name"].as_str().unwrap_or("").to_string();
    let subject = course["subject"].as_str().unwrap_or("").to_string();
    let grade_level = course["grade_level"].as_str().unwrap_or("").to_string();
    let plan = course["plan"].as_str().unwrap_or("").to_string();
    let section = course["section"].as_str().unwrap_or("").to_string();
    let teacher_id = course["teacher_id"].as_str().unwrap_or("").to_string();
    let plan_display = if plan.is_empty() { "-".to_string() } else { plan.clone() };

    rsx! {
        tr {
            td { class: "cell-name", "{name}" }
            td { "{subject}" }
            td { "{grade_level}" }
            td { "{plan_display}" }
            td { "{section}" }
            td { class: "cell-mono", "{&teacher_id[..8]}..." }
            td { class: "cell-actions",
                button { class: "btn-icon", onclick: move |_| on_edit.call(course.clone()),
                    svg { role: "presentation", view_box: "0 0 24 24", width: "16", height: "16",
                        path { d: "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" }
                    }
                }
                button { class: "btn-icon btn-icon-danger", onclick: move |_| on_delete.call(id.clone()),
                    svg { role: "presentation", view_box: "0 0 24 24", width: "16", height: "16",
                        path { d: "M3 6h18" }
                        path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
                    }
                }
            }
        }
    }
}
