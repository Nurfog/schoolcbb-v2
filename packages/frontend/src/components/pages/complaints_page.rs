use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn ComplaintsPage() -> Element {
    let mut tab = use_signal(|| "submit".to_string());
    let complaints = use_resource(|| async move {
        client::fetch_json("/api/hr/complaints").await
    });

    rsx! {
        div { class: "page-header",
            h1 { "Ley Karin — Canal de Denuncias" }
            p { "Sistema de denuncias conforme a la Ley 21.643 (Ley Karin)" }
        }
        div { class: "tabs-container",
            div { class: "tabs-header",
                button { class: if tab() == "submit" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("submit".to_string()), "Denunciar" }
                button { class: if tab() == "list" { "tab-active" } else { "tab" }, onclick: move |_| tab.set("list".to_string()), "Gestionar Denuncias" }
            }
            div { class: "tab-content",
                match tab() {
                s if s == "submit" => rsx! { SubmitComplaint {} },
                s if s == "list" => rsx! { ComplaintsList { data: complaints } },
                _ => rsx! {}
                }
            }
        }
    }
}

#[component]
fn SubmitComplaint() -> Element {
    let mut anonymous = use_signal(|| true);
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut accused_rut = use_signal(String::new);
    let mut complaint_type = use_signal(|| "Acoso Laboral".to_string());
    let mut description = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut done = use_signal(|| false);
    let mut error = use_signal(|| String::new());

    let do_submit = move |_| {
        if description().trim().is_empty() { return; }
        submitting.set(true);
        error.set(String::new());
        let payload = serde_json::json!({
            "complainant_name": if anonymous() { serde_json::Value::Null } else { serde_json::Value::String(name()) },
            "complainant_email": if anonymous() { serde_json::Value::Null } else { serde_json::Value::String(email()) },
            "accused_rut": accused_rut(),
            "complaint_type": complaint_type(),
            "description": description(),
        });
        spawn(async move {
            match client::post_json("/api/hr/complaints/submit", &payload).await {
                Ok(_) => { done.set(true); }
                Err(e) => { error.set(e); }
            }
            submitting.set(false);
        });
    };

    if done() {
        return rsx! {
            div { class: "empty-state",
                h2 { "Denuncia recibida" }
                p { "Tu denuncia ha sido registrada. El equipo de RRHH la revisará a la brevedad." }
                button { class: "btn btn-primary", onclick: move |_| {
                    done.set(false);
                    description.set(String::new());
                    name.set(String::new());
                    email.set(String::new());
                    accused_rut.set(String::new());
                }, "Nueva denuncia" }
            }
        };
    }

    rsx! {
        div { class: "form-card",
            div { class: "form-row",
                div { class: "form-group",
                    label { "Tipo de denuncia:" }
                    select { class: "form-input", value: "{complaint_type}", onchange: move |e| complaint_type.set(e.value()),
                        option { value: "Acoso Laboral", "Acoso Laboral" }
                        option { value: "Acoso Sexual", "Acoso Sexual" }
                        option { value: "Violencia en el Trabajo", "Violencia en el Trabajo" }
                        option { value: "Discriminacion", "Discriminacion" }
                        option { value: "Incumplimiento Normativo", "Incumplimiento Normativo" }
                        option { value: "Otro", "Otro" }
                    }
                }
                div { class: "form-group",
                    label { "RUT del denunciado (opcional):" }
                    input { class: "form-input", value: "{accused_rut}", oninput: move |e| accused_rut.set(e.value()), placeholder: "12.345.678-9" }
                }
            }
            div { class: "form-group",
                div { class: "checkbox-group",
                    input { "type": "checkbox", checked: "{anonymous}", onchange: move |_| anonymous.set(!anonymous()) }
                    label { "Denuncia anonima" }
                }
            }
            { if !anonymous() {
                rsx! {
                    div { class: "form-row",
                        div { class: "form-group",
                            label { "Nombre:" }
                            input { class: "form-input", value: "{name}", oninput: move |e| name.set(e.value()) }
                        }
                        div { class: "form-group",
                            label { "Email:" }
                            input { class: "form-input", value: "{email}", oninput: move |e| email.set(e.value()), placeholder: "correo@ejemplo.cl" }
                        }
                    }
                }
            } else { rsx! {} }}
            div { class: "form-group",
                label { "Descripcion de los hechos:" }
                textarea { class: "form-input", rows: "6", value: "{description}", oninput: move |e| description.set(e.value()),
                    placeholder: "Describe los hechos con el maximo detalle posible..." }
            }
            { if !error().is_empty() {
                rsx! { div { class: "alert alert-error", "{error}" } }
            } else { rsx! {} }}
            div { class: "form-actions",
                button { class: "btn btn-primary", disabled: submitting(), onclick: do_submit,
                    if submitting() { "Enviando..." } else { "Enviar Denuncia" }
                }
            }
        }
    }
}

#[component]
fn ComplaintsList(data: Resource<Result<serde_json::Value, String>>) -> Element {
    rsx! {
        div { class: "data-table-container",
            match data() {
                Some(Ok(json)) => {
                    let list = json["complaints"].as_array().cloned().unwrap_or_default();
                    if list.is_empty() {
                        rsx! { div { class: "empty-state", "Sin denuncias registradas" } }
                    } else {
                        let rows: Vec<Element> = list.iter().map(|c| {
                            let ctype = c["complaint_type"].as_str().unwrap_or("").to_string();
                            let desc = c["description"].as_str().unwrap_or("").to_string();
                            let status = c["status"].as_str().unwrap_or("Pendiente").to_string();
                            let date = c["created_at"].as_str().unwrap_or("").to_string();
                            let is_anon = c["complainant_name"].is_null();
                            let complainant = if is_anon { "Anonimo".to_string() } else { c["complainant_name"].as_str().unwrap_or("").to_string() };
                            rsx! {
                                tr {
                                    td { span { class: "role-badge", "{ctype}" } }
                                    td { "{complainant}" }
                                    td { "{desc.chars().take(60).collect::<String>()}..." }
                                    td {
                                        if status == "Pendiente" { span { class: "status-pending", "Pendiente" } }
                                        else if status == "Resuelto" { span { class: "status-active", "Resuelto" } }
                                        else { span { class: "status-inactive", "{status}" } }
                                    }
                                    td { "{date}" }
                                }
                            }
                        }).collect();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Tipo" }
                                    th { "Denunciante" }
                                    th { "Descripcion" }
                                    th { "Estado" }
                                    th { "Fecha" }
                                }}
                                tbody { { rows.into_iter() } }
                            }
                        }
                    }
                }
                _ => rsx! { div { class: "empty-state", "Cargando..." } },
            }
        }
    }
}
