use dioxus::prelude::*;
use serde_json::json;

use crate::api::client;

#[component]
pub fn SostenedorPage() -> Element {
    let profile = use_resource(|| async move { client::fetch_json("/api/auth/me").await });
    let summary = use_resource(|| async move { client::fetch_json("/api/dashboard/summary").await });
    let license = use_resource(|| async move { client::fetch_json("/api/client/license").await });

    rsx! {
        div { class: "page-header",
            h1 { "Panel del Sostenedor" }
            p { "Visión general de tu institución" }
        }
        div { class: "dashboard-grid",
            match profile() {
                Some(Ok(data)) => {
                    let profile_data = data["user"].as_object().map(|_| &data["user"]).unwrap_or(&data);
                    let name = profile_data["name"].as_str().unwrap_or("Usuario");
                    let role = profile_data["role"].as_str().unwrap_or("");
                    let school_id = profile_data["school_id"].as_str().and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
                    let corporation_id = profile_data["corporation_id"].as_str().and_then(|s| if s.is_empty() { None } else { Some(s.to_string()) });
                    let is_individual = school_id.is_some() && corporation_id.is_none();
                    let school_name = profile_data["school_name"].as_str().unwrap_or("");

                    rsx! {
                        div { class: "widget-card",
                            h2 { "Bienvenido, {name}" }
                            p { "Rol: {role}" }
                            if is_individual {
                                p { "Colegio Individual" }
                            }
                            if !school_name.is_empty() { p { "{school_name}" } }
                        }
                        if is_individual {
                            IndividualSchoolPanel { school_id: school_id.clone().unwrap_or_default() }
                        }
                    }
                }
                _ => rsx! {}
            }
            match license() {
                Some(Ok(data)) => {
                    let plan = data["plan_name"].as_str().unwrap_or("-");
                    let status = data["status"].as_str().unwrap_or("-");
                    let days = data["days_remaining"].as_i64().unwrap_or(0);
                    let sc = if status == "active" { "badge-success" } else { "badge-warning" };
                    let modules = data["modules"].as_array().cloned().unwrap_or_default();
                    let enabled_count = modules.iter().filter(|m| m["included"].as_bool().unwrap_or(false)).count();
                    rsx! {
                        div { class: "widget-card",
                            h2 { "Plan y Licencia" }
                            p { "Plan: {plan}" }
                            p { "Estado: ", span { class: "badge {sc}", "{status}" } }
                            p { "Días restantes: {days}" }
                            p { "Módulos habilitados: {enabled_count}/{modules.len()}" }
                        }
                    }
                }
                _ => rsx! {}
            }
            match summary() {
                Some(Ok(data)) => {
                    let students = data["total_students"].as_i64().unwrap_or(0);
                    let teachers = data["total_teachers"].as_i64().unwrap_or(0);
                    let attendance = data["attendance_percentage"].as_f64().unwrap_or(0.0);
                    rsx! {
                        div { class: "widget-card",
                            h2 { "Resumen" }
                            p { "Alumnos: {students}" }
                            p { "Docentes: {teachers}" }
                            p { "Asistencia: {attendance:.1}%" }
                        }
                    }
                }
                _ => rsx! {}
            }
            div { class: "widget-card",
                h2 { "Acciones Rápidas" }
                ul {
                    li { a { href: "/users", "Gestionar Usuarios" } }
                    li { a { href: "/corporations", "Mi Corporación" } }
                    li { a { href: "/students", "Ver Alumnos" } }
                    li { a { href: "/reports", "Reportes" } }
                }
            }
        }
    }
}

#[component]
fn IndividualSchoolPanel(school_id: String) -> Element {
    let mut show_form = use_signal(|| false);
    let mut admin_name = use_signal(String::new);
    let mut admin_email = use_signal(String::new);
    let mut admin_rut = use_signal(String::new);
    let mut saving = use_signal(|| false);

    let do_create = move |_| {
        let name = admin_name();
        let email = admin_email();
        let rut = admin_rut();
        if name.trim().is_empty() || rut.trim().is_empty() {
            return;
        }
        saving.set(true);
        let sid = school_id.clone();
        spawn(async move {
            let payload = json!({
                "name": name,
                "email": email,
                "rut": rut,
                "password": rut,
                "role": "Administrador",
                "admin_type": "school",
                "school_id": sid,
            });
            let _ = client::post_json("/api/auth/register", &payload).await;
            saving.set(false);
            show_form.set(false);
            admin_name.set(String::new());
            admin_email.set(String::new());
            admin_rut.set(String::new());
        });
    };

    rsx! {
        div { class: "widget-card",
            h2 { "Colegio Individual" }
            p { "Gestionas un colegio de forma independiente." }
            hr {}
            h3 { "Crear Administrador del Colegio" }
            p { "Crea un usuario administrador que gestione este colegio en tu nombre." }
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()),
                if show_form() { "Cancelar" } else { "Crear Administrador del Colegio" }
            }
            if show_form() {
                div { class: "form-card", style: "margin-top: 12px;",
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "RUT:" }
                            input { class: "form-input", value: "{admin_rut}", oninput: move |e| admin_rut.set(e.value()), placeholder: "11.111.111-1" }
                        }
                        div { class: "form-group",
                            label { "Nombre:" }
                            input { class: "form-input", value: "{admin_name}", oninput: move |e| admin_name.set(e.value()), placeholder: "Nombre completo" }
                        }
                    }
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Email:" }
                            input { class: "form-input", r#type: "email", value: "{admin_email}", oninput: move |e| admin_email.set(e.value()), placeholder: "admin@colegio.cl" }
                        }
                        div { class: "form-group",
                            label { "Contraseña inicial:" }
                            input { class: "form-input", r#type: "password", value: "{admin_rut}", disabled: true, placeholder: "Se usará el RUT" }
                        }
                    }
                    div { class: "form-actions",
                        button { class: "btn btn-primary", disabled: saving(), onclick: do_create,
                            if saving() { "Creando..." } else { "Crear Administrador" }
                        }
                    }
                }
            }
        }
    }
}
