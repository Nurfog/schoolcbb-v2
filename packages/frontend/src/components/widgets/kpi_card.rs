use dioxus::prelude::*;

#[component]
pub fn KpiCard(
    label: String,
    value: String,
    color: Option<String>,
    icon: Option<String>,
) -> Element {
    let css_class = match color.as_deref() {
        Some("#66bb6a") => "kpi-value success",
        Some("#ff7043" | "#ffa726") => "kpi-value warning",
        Some("#ab47bc" | "#26c6da") => "kpi-value info",
        _ => "kpi-value primary",
    };
    rsx! {
        div { class: "kpi-item",
            if let Some(ic) = icon {
                div { style: "font-size: 1.5rem; margin-bottom: 4px;", "{ic}" }
            }
            div { class: "{css_class}", "{value}" }
            div { class: "kpi-label", "{label}" }
        }
    }
}
