use std::rc::Rc;

use dioxus::prelude::*;

#[component]
pub fn InlineEdit(
    value: String,
    field: String,
    entity_id: String,
    api_url: String,
    input_type: Option<String>,
    options: Option<Vec<String>>,
) -> Element {
    let mut editing = use_signal(|| false);
    let mut edit_value = use_signal(|| value.clone());
    let mut saving = use_signal(|| false);
    let display = value.clone();

    let value_rc = Rc::new(value);
    let start_edit = {
        let v = value_rc.clone();
        move |_: Event<MouseData>| {
            edit_value.set((*v).clone());
            editing.set(true);
        }
    };

    let v2 = display.clone();
    let f2 = Rc::new(field.clone());
    let u2 = Rc::new(api_url.clone());
    let mut do_save = {
        let v = v2;
        let f = f2.clone();
        let u = u2.clone();
        move || {
            if edit_value() == v {
                editing.set(false);
                return;
            }
            saving.set(true);
            let payload = serde_json::json!({ (*f).clone(): edit_value() });
            let url = (*u).clone();
            spawn(async move {
                let _ = crate::api::client::put_json(&url, &payload).await;
                saving.set(false);
                editing.set(false);
            });
        }
    };

    let on_select_change = {
        let u = u2.clone();
        let f = f2.clone();
        move |e: Event<FormData>| {
            let url = (*u).clone();
            let field_name = (*f).clone();
            let new_val = e.value();
            saving.set(true);
            spawn(async move {
                let payload = serde_json::json!({ field_name: new_val });
                let _ = crate::api::client::put_json(&url, &payload).await;
                saving.set(false);
                editing.set(false);
            });
        }
    };

    let on_text_change = move |e: Event<FormData>| {
        edit_value.set(e.value());
    };

    let on_blur = move |_: Event<FocusData>| {
        do_save();
    };

    if editing() && saving() {
        return rsx! { span { class: "inline-edit-saving", "Guardando..." } };
    }

    if editing() {
        if input_type.as_deref() == Some("select") {
            if let Some(ref opts) = options {
                let val = edit_value();
                let opt_elements: Vec<Element> = opts.iter().map(|opt| {
                    let selected = opt == &val;
                    rsx! { option { selected: "{selected}", value: "{opt}", "{opt}" } }
                }).collect();
                return rsx! {
                    select {
                        class: "inline-edit-input",
                        value: "{val}",
                        onchange: on_select_change,
                        { opt_elements.into_iter() }
                    }
                };
            }
        } else {
            let val = edit_value();
            return rsx! {
                input {
                    class: "inline-edit-input",
                    value: "{val}",
                    oninput: on_text_change,
                    onblur: on_blur,
                    autofocus: true,
                }
            };
        }
    }

    rsx! {
        span {
            class: "inline-edit-value",
            ondoubleclick: start_edit,
            title: "Doble clic para editar",
            "{display}"
        }
    }
}
