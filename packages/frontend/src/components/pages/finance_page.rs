use dioxus::prelude::*;

use crate::api::client;

#[derive(PartialEq, Clone)]
enum FinanceTab {
    Fees,
    Payments,
    Scholarships,
}

#[component]
pub fn FinancePage() -> Element {
    let mut active_tab = use_signal(|| FinanceTab::Fees);

    rsx! {
        div { class: "page-header",
            h1 { "Finanzas" }
            p { "Gestión de cuotas, pagos y becas" }
        }
        div { class: "tab-bar",
            button {
                class: if active_tab() == FinanceTab::Fees { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Fees),
                "Cuotas"
            }
            button {
                class: if active_tab() == FinanceTab::Payments { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Payments),
                "Pagos"
            }
            button {
                class: if active_tab() == FinanceTab::Scholarships { "tab active" } else { "tab" },
                onclick: move |_| active_tab.set(FinanceTab::Scholarships),
                "Becas"
            }
        }
        div { class: "tab-content",
            {
                match active_tab() {
                    FinanceTab::Fees => rsx! { FeesTab {} },
                    FinanceTab::Payments => rsx! { PaymentsTab {} },
                    FinanceTab::Scholarships => rsx! { ScholarshipsTab {} },
                }
            }
        }
    }
}

#[component]
fn FeesTab() -> Element {
    let mut fees = use_resource(|| client::fetch_all_fees());
    let mut show_form = use_signal(|| false);
    let mut student_search = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut description = use_signal(|| String::new());
    let mut amount = use_signal(|| String::new());
    let mut due_date = use_signal(|| String::new());
    let mut saving = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = student_search();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let mut reset_form = move || {
        student_search.set(String::new());
        selected_student.set(None);
        description.set(String::new());
        amount.set(String::new());
        due_date.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if selected_student().is_none() || description().is_empty() || amount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "student_id": selected_student().unwrap()["id"].as_str().unwrap_or(""),
            "description": description(),
            "amount": amount().parse::<f64>().unwrap_or(0.0),
            "due_date": due_date(),
        });
        spawn(async move {
            let _ = client::create_fee(&payload).await;
            saving.set(false);
            reset_form();
            fees.restart();
        });
    };

    let do_mark_paid = move |fee_id: String| {
        spawn(async move {
            let _ = client::mark_fee_paid(&fee_id).await;
            fees.restart();
        });
    };

    let do_delete = move |fee_id: String| {
        spawn(async move {
            let _ = client::delete_fee(&fee_id).await;
            fees.restart();
        });
    };

    let do_online_payment = move |fee_id: String| {
        spawn(async move {
            match client::init_online_payment(&fee_id).await {
                Ok(data) => {
                    if let Some(url) = data["url"].as_str() {
                        let _ = web_sys::window().and_then(|w| w.location().assign(url).ok());
                    }
                }
                Err(e) => {
                    web_sys::window()
                        .and_then(|w| w.alert_with_message(&format!("Error: {e}")).ok());
                }
            }
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nueva Cuota" } }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "student-selector", style: "width: 100%;",
                                label { "Estudiante:" }
                                {
                                    match selected_student() {
                                        Some(ref s) => {
                                            let sname = format!("{} {}",
                                                s["first_name"].as_str().unwrap_or(""),
                                                s["last_name"].as_str().unwrap_or("")
                                            );
                                            rsx! {
                                                div { class: "selected-student",
                                                    span { "{sname}" }
                                                    button { class: "btn-icon", onclick: move |_| selected_student.set(None), "✕" }
                                                }
                                            }
                                        }
                                        None => rsx! {
                                            input { class: "search-input", value: "{student_search}", oninput: move |evt| student_search.set(evt.value()), placeholder: "Buscar estudiante..." }
                                        }
                                    }
                                }
                                {
                                    match search_results() {
                                        Some(Ok(j)) => {
                                            let list = j["students"].as_array().cloned().unwrap_or_default();
                                            if !list.is_empty() && student_search().len() >= 2 && selected_student().is_none() {
                                                let items: Vec<Element> = list.iter().map(|s| {
                                                    let sid = s["id"].as_str().unwrap_or("").to_string();
                                                    let sname = format!("{} {}",
                                                        s["first_name"].as_str().unwrap_or(""),
                                                        s["last_name"].as_str().unwrap_or("")
                                                    );
                                                    rsx! {
                                                        div {
                                                            class: "search-result-item",
                                                            onclick: move |_| {
                                                                selected_student.set(Some(serde_json::json!({"id": sid.clone(), "first_name": sname.clone()})));
                                                                student_search.set(String::new());
                                                            },
                                                            span { "{sname}" }
                                                        }
                                                    }
                                                }).collect();
                                                rsx! { div { class: "search-results", { items.into_iter() } } }
                                            } else { rsx! {} }
                                        }
                                        _ => rsx! {},
                                    }
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Descripción:" }
                                input { class: "form-input", value: "{description}", oninput: move |evt| description.set(evt.value()), placeholder: "Ej: Matrícula 2025" }
                            }
                            div { class: "form-group",
                                label { "Monto:" }
                                input { class: "form-input", value: "{amount}", oninput: move |evt| amount.set(evt.value()), type: "number", step: "1000", placeholder: "0" }
                            }
                            div { class: "form-group",
                                label { "Vencimiento:" }
                                input { class: "form-input", value: "{due_date}", oninput: move |evt| due_date.set(evt.value()), type: "date" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            {
                match fees() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String, bool, String)> = j["fees"].as_array().map(|arr| {
                            arr.iter().map(|fee| {
                                let sid = fee["student_id"].as_str().unwrap_or("-").to_string();
                                let desc = fee["description"].as_str().unwrap_or("").to_string();
                                let fid = fee["id"].as_str().unwrap_or("").to_string();
                                let monto_display = {
                                    let m = fee["amount"].as_f64().unwrap_or(0.0);
                                    format!("${:.0}", m)
                                };
                                let due = fee["due_date"].as_str().unwrap_or("").to_string();
                                let paid = fee["paid"].as_bool().unwrap_or(false);
                                let paid_display = if paid {
                                    let amt = fee["paid_amount"].as_f64().unwrap_or(0.0);
                                    format!("${:.0}", amt)
                                } else { "-".to_string() };
                                (fid, sid, desc, monto_display, due, paid, paid_display)
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Descripción" }
                                    th { "Monto" }
                                    th { "Vencimiento" }
                                    th { "Estado" }
                                    th { "Pagado" }
                                    th { "Acciones" }
                                }}
                                tbody { for (fid, sid, desc, monto_display, due, paid, paid_display) in &list {
                                    tr {
                                        td { "{sid}" }
                                        td { "{desc}" }
                                        td { "{monto_display}" }
                                        td { "{due}" }
                                        td { if *paid { span { class: "grade-good", "Pagado" } } else { span { class: "grade-bad", "Pendiente" } } }
                                        td { "{paid_display}" }
                                        td {
                                            if !paid {
                                                button { class: "btn btn-sm btn-success", onclick: { let id = fid.clone(); move |_| do_mark_paid(id.clone()) }, "Pagar" }
                                                button { class: "btn btn-sm btn-info", style: "margin-left: 4px;", onclick: { let id = fid.clone(); move |_| do_online_payment(id.clone()) }, "Online" }
                                            }
                                            button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let id = fid.clone(); move |_| do_delete(id.clone()) }, "Eliminar" }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
                }
            }
        }
    }
}

#[component]
fn PaymentsTab() -> Element {
    let mut payments = use_resource(|| client::fetch_all_payments());
    let mut show_form = use_signal(|| false);
    let mut fee_id = use_signal(|| String::new());
    let mut student_search = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut amount = use_signal(|| String::new());
    let mut payment_method = use_signal(|| "Efectivo".to_string());
    let mut reference = use_signal(|| String::new());
    let mut saving = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = student_search();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let mut reset_form = move || {
        fee_id.set(String::new());
        student_search.set(String::new());
        selected_student.set(None);
        amount.set(String::new());
        payment_method.set("Efectivo".to_string());
        reference.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if selected_student().is_none() || amount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "fee_id": fee_id(),
            "student_id": selected_student().unwrap()["id"].as_str().unwrap_or(""),
            "amount": amount().parse::<f64>().unwrap_or(0.0),
            "payment_method": payment_method(),
            "reference": reference(),
        });
        spawn(async move {
            let _ = client::create_payment(&payload).await;
            saving.set(false);
            reset_form();
            payments.restart();
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nuevo Pago" } }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "student-selector", style: "width: 100%;",
                                label { "Estudiante:" }
                                {
                                    match selected_student() {
                                        Some(ref s) => {
                                            let sname = format!("{} {}",
                                                s["first_name"].as_str().unwrap_or(""),
                                                s["last_name"].as_str().unwrap_or("")
                                            );
                                            rsx! {
                                                div { class: "selected-student",
                                                    span { "{sname}" }
                                                    button { class: "btn-icon", onclick: move |_| selected_student.set(None), "✕" }
                                                }
                                            }
                                        }
                                        None => rsx! {
                                            input { class: "search-input", value: "{student_search}", oninput: move |evt| student_search.set(evt.value()), placeholder: "Buscar estudiante..." }
                                        }
                                    }
                                }
                                {
                                    match search_results() {
                                        Some(Ok(j)) => {
                                            let list = j["students"].as_array().cloned().unwrap_or_default();
                                            if !list.is_empty() && student_search().len() >= 2 && selected_student().is_none() {
                                                let items: Vec<Element> = list.iter().map(|s| {
                                                    let sid = s["id"].as_str().unwrap_or("").to_string();
                                                    let sname = format!("{} {}",
                                                        s["first_name"].as_str().unwrap_or(""),
                                                        s["last_name"].as_str().unwrap_or("")
                                                    );
                                                    rsx! {
                                                        div {
                                                            class: "search-result-item",
                                                            onclick: move |_| {
                                                                selected_student.set(Some(serde_json::json!({"id": sid.clone(), "first_name": sname.clone()})));
                                                                student_search.set(String::new());
                                                            },
                                                            span { "{sname}" }
                                                        }
                                                    }
                                                }).collect();
                                                rsx! { div { class: "search-results", { items.into_iter() } } }
                                            } else { rsx! {} }
                                        }
                                        _ => rsx! {},
                                    }
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "ID Cuota (opcional):" }
                                input { class: "form-input", value: "{fee_id}", oninput: move |evt| fee_id.set(evt.value()), placeholder: "UUID de la cuota" }
                            }
                            div { class: "form-group",
                                label { "Monto:" }
                                input { class: "form-input", value: "{amount}", oninput: move |evt| amount.set(evt.value()), type: "number", step: "1000", placeholder: "0" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Método:" }
                                select { class: "form-input", value: "{payment_method}", onchange: move |evt| payment_method.set(evt.value()),
                                    option { value: "Efectivo", "Efectivo" }
                                    option { value: "Transferencia", "Transferencia" }
                                    option { value: "Tarjeta", "Tarjeta" }
                                    option { value: "Cheque", "Cheque" }
                                }
                            }
                            div { class: "form-group",
                                label { "Referencia:" }
                                input { class: "form-input", value: "{reference}", oninput: move |evt| reference.set(evt.value()), placeholder: "Nº transferencia, cheque..." }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            {
                match payments() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String)> = j["payments"].as_array().map(|arr| {
                            arr.iter().map(|p| {
                                let amt = p["amount"].as_f64().unwrap_or(0.0);
                                let amt_display = format!("${:.0}", amt);
                                (
                                    p["student_id"].as_str().unwrap_or("-").to_string(),
                                    amt_display,
                                    p["payment_date"].as_str().unwrap_or("").to_string(),
                                    p["payment_method"].as_str().unwrap_or("").to_string(),
                                    p["reference"].as_str().unwrap_or("-").to_string(),
                                )
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Monto" }
                                    th { "Fecha" }
                                    th { "Método" }
                                    th { "Referencia" }
                                }}
                                tbody { for (sid, amt_display, date, method, ref_text) in &list {
                                    tr {
                                        td { "{sid}" }
                                        td { "{amt_display}" }
                                        td { "{date}" }
                                        td { "{method}" }
                                        td { "{ref_text}" }
                                    }
                                }}
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
                }
            }
        }
    }
}

#[component]
fn ScholarshipsTab() -> Element {
    let mut scholarships = use_resource(|| client::fetch_all_scholarships());
    let mut show_form = use_signal(|| false);
    let mut student_search = use_signal(String::new);
    let mut selected_student = use_signal(|| None::<serde_json::Value>);
    let mut name = use_signal(|| String::new());
    let mut discount = use_signal(|| String::new());
    let mut valid_from = use_signal(|| String::new());
    let mut valid_until = use_signal(|| String::new());
    let mut saving = use_signal(|| false);
    let search_results = use_resource(move || {
        let q = student_search();
        async move {
            if q.len() < 2 {
                Ok(serde_json::json!({"students": []}))
            } else {
                client::search_students(&q).await
            }
        }
    });

    let mut reset_form = move || {
        student_search.set(String::new());
        selected_student.set(None);
        name.set(String::new());
        discount.set(String::new());
        valid_from.set(String::new());
        valid_until.set(String::new());
        show_form.set(false);
    };

    let do_save = move |_| {
        if selected_student().is_none() || name().is_empty() || discount().is_empty() {
            return;
        }
        saving.set(true);
        let payload = serde_json::json!({
            "student_id": selected_student().unwrap()["id"].as_str().unwrap_or(""),
            "name": name(),
            "discount_percentage": discount().parse::<f64>().unwrap_or(0.0),
            "valid_from": valid_from(),
            "valid_until": valid_until(),
        });
        spawn(async move {
            let _ = client::create_scholarship(&payload).await;
            saving.set(false);
            reset_form();
            scholarships.restart();
        });
    };

    let do_approve = move |sid: String| {
        spawn(async move {
            let _ = client::approve_scholarship(&sid).await;
            scholarships.restart();
        });
    };

    let do_delete = move |sid: String| {
        spawn(async move {
            let _ = client::delete_scholarship(&sid).await;
            scholarships.restart();
        });
    };

    rsx! {
        div { class: "toolbar-row",
            button { class: "btn btn-primary", onclick: move |_| show_form.set(!show_form()), if show_form() { "Cancelar" } else { "Nueva Beca" } }
        }
        {
            if show_form() {
                rsx! {
                    div { class: "form-card",
                        div { class: "form-row",
                            div { class: "student-selector", style: "width: 100%;",
                                label { "Estudiante:" }
                                {
                                    match selected_student() {
                                        Some(ref s) => {
                                            let sname = format!("{} {}",
                                                s["first_name"].as_str().unwrap_or(""),
                                                s["last_name"].as_str().unwrap_or("")
                                            );
                                            rsx! {
                                                div { class: "selected-student",
                                                    span { "{sname}" }
                                                    button { class: "btn-icon", onclick: move |_| selected_student.set(None), "✕" }
                                                }
                                            }
                                        }
                                        None => rsx! {
                                            input { class: "search-input", value: "{student_search}", oninput: move |evt| student_search.set(evt.value()), placeholder: "Buscar estudiante..." }
                                        }
                                    }
                                }
                                {
                                    match search_results() {
                                        Some(Ok(j)) => {
                                            let list = j["students"].as_array().cloned().unwrap_or_default();
                                            if !list.is_empty() && student_search().len() >= 2 && selected_student().is_none() {
                                                let items: Vec<Element> = list.iter().map(|s| {
                                                    let sid = s["id"].as_str().unwrap_or("").to_string();
                                                    let sname = format!("{} {}",
                                                        s["first_name"].as_str().unwrap_or(""),
                                                        s["last_name"].as_str().unwrap_or("")
                                                    );
                                                    rsx! {
                                                        div {
                                                            class: "search-result-item",
                                                            onclick: move |_| {
                                                                selected_student.set(Some(serde_json::json!({"id": sid.clone(), "first_name": sname.clone()})));
                                                                student_search.set(String::new());
                                                            },
                                                            span { "{sname}" }
                                                        }
                                                    }
                                                }).collect();
                                                rsx! { div { class: "search-results", { items.into_iter() } } }
                                            } else { rsx! {} }
                                        }
                                        _ => rsx! {},
                                    }
                                }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Nombre Beca:" }
                                input { class: "form-input", value: "{name}", oninput: move |evt| name.set(evt.value()), placeholder: "Ej: Beca Excelencia" }
                            }
                            div { class: "form-group",
                                label { "% Descuento:" }
                                input { class: "form-input", value: "{discount}", oninput: move |evt| discount.set(evt.value()), type: "number", min: "1", max: "100", placeholder: "0" }
                            }
                        }
                        div { class: "form-row",
                            div { class: "form-group",
                                label { "Vigencia desde:" }
                                input { class: "form-input", value: "{valid_from}", oninput: move |evt| valid_from.set(evt.value()), type: "date" }
                            }
                            div { class: "form-group",
                                label { "Vigencia hasta:" }
                                input { class: "form-input", value: "{valid_until}", oninput: move |evt| valid_until.set(evt.value()), type: "date" }
                            }
                        }
                        div { class: "form-actions",
                            button { class: "btn btn-primary", disabled: saving(), onclick: do_save, if saving() { "Guardando..." } else { "Guardar" } }
                            button { class: "btn", onclick: move |_| reset_form(), "Cancelar" }
                        }
                    }
                }
            } else { rsx! {} }
        }
        div { class: "data-table-container",
            {
                match scholarships() {
                    Some(Ok(j)) => {
                        let list: Vec<(String, String, String, String, String, String, bool)> = j["scholarships"].as_array().map(|arr| {
                            arr.iter().map(|s| {
                                let disc = s["discount_percentage"].as_f64().unwrap_or(0.0);
                                let disc_display = format!("{:.0}%", disc);
                                (
                                    s["id"].as_str().unwrap_or("").to_string(),
                                    s["student_id"].as_str().unwrap_or("-").to_string(),
                                    s["name"].as_str().unwrap_or("").to_string(),
                                    disc_display,
                                    s["valid_from"].as_str().unwrap_or("").to_string(),
                                    s["valid_until"].as_str().unwrap_or("").to_string(),
                                    s["approved"].as_bool().unwrap_or(false),
                                )
                            }).collect()
                        }).unwrap_or_default();
                        rsx! {
                            table { class: "data-table",
                                thead { tr {
                                    th { "Estudiante" }
                                    th { "Nombre" }
                                    th { "% Descuento" }
                                    th { "Vigencia" }
                                    th { "Estado" }
                                    th { "Acciones" }
                                }}
                                tbody { for (sid, stu_id, sname, disc_display, vfrom, vuntil, approved) in &list {
                                    tr {
                                        td { "{stu_id}" }
                                        td { "{sname}" }
                                        td { "{disc_display}" }
                                        td { "{vfrom} - {vuntil}" }
                                        td { if *approved { span { class: "grade-good", "Aprobada" } } else { span { class: "grade-bad", "Pendiente" } } }
                                        td {
                                            if !approved {
                                                button { class: "btn btn-sm btn-success", onclick: { let id = sid.clone(); move |_| do_approve(id.clone()) }, "Aprobar" }
                                            }
                                            button { class: "btn btn-sm btn-danger", style: "margin-left: 4px;", onclick: { let id = sid.clone(); move |_| do_delete(id.clone()) }, "Eliminar" }
                                        }
                                    }
                                }}
                            }
                        }
                    }
                    Some(Err(e)) => rsx! { div { class: "empty-state", "Error: {e}" } },
                    None => rsx! { div { class: "empty-state", div { class: "loading-spinner", "Cargando..." } } },
                }
            }
        }
    }
}
