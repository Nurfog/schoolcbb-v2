use dioxus::prelude::*;

use crate::api::client;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    let do_login = move |_| {
        if email().is_empty() || password().is_empty() {
            error.set(Some("Email y contraseña son obligatorios".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);
        let e = email();
        let p = password();
        let nav = navigator();
        spawn(async move {
            match client::login(&e, &p).await {
                Ok(resp) => {
                    if resp.get("token").is_some() {
                        nav.replace("/");
                    } else {
                        let msg = resp.get("error").and_then(|v| v.as_str()).unwrap_or("Credenciales inválidas");
                        loading.set(false);
                        error.set(Some(msg.to_string()));
                    }
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e));
                }
            }
        });
    };

    rsx! {
        div { class: "login-container",
            div { class: "login-card",
                div { class: "login-header",
                    div { class: "login-logo", "SC" }
                    h1 { "SchoolCBB" }
                    p { "Plataforma Escolar" }
                }
                div { class: "login-form",
                    if let Some(ref msg) = error() {
                        div { class: "login-error", "{msg}" }
                    }
                    div { class: "field",
                        label { "Email" }
                        input {
                            class: "login-input",
                            "type": "email",
                            placeholder: "email@colegio.cl",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value()),
                        }
                    }
                    div { class: "field",
                        label { "Contraseña" }
                        input {
                            class: "login-input",
                            "type": "password",
                            placeholder: "contraseña",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value()),
                        }
                    }
                    button {
                        class: "login-btn",
                        onclick: do_login,
                        disabled: loading(),
                        if loading() { "Ingresando..." } else { "Iniciar Sesión" }
                    }
                }
            }
        }
    }
}
