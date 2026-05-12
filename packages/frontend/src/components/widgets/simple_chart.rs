use dioxus::prelude::*;
use serde_json::Value;

#[component]
pub fn BarChart(
    data: Vec<Value>,
    label_key: String,
    value_key: String,
    height: Option<u32>,
    color: Option<String>,
) -> Element {
    let h = height.unwrap_or(200) as f64;
    let bar_color = color.unwrap_or_else(|| "#4fc3f7".to_string());
    let max_val = data
        .iter()
        .filter_map(|d| d[&value_key].as_f64())
        .fold(0.0_f64, f64::max);
    let max = if max_val == 0.0 { 1.0 } else { max_val };
    let count = data.len();
    let width = (count * 60).max(100);

    let bars: Vec<Element> = data
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let val = d[&value_key].as_f64().unwrap_or(0.0);
            let bar_h = (val / max * (h - 20.0)) as u32;
            let x = (i * 60 + 10) as u32;
            let label = d[&label_key].as_str().unwrap_or("").to_string();
            let label_y = (h - 5.0) as u32;
            let y = (h - bar_h as f64 - 10.0) as u32;
            rsx! {
                rect { key: "bar{i}", x: "{x}", y: "{y}", width: "40", height: "{bar_h}", fill: "{bar_color}", rx: "4" }
                text { key: "lb{i}", x: "{x + 20}", y: "{label_y}", text_anchor: "middle", font_size: "10", fill: "#546e7a", "{label}" }
            }
        })
        .collect();

    rsx! {
        div { style: "width: 100%; overflow-x: auto;",
            svg {
                width: "{width}",
                height: "{h}",
                view_box: "0 0 {width} {h}",
                {bars.into_iter()}
            }
        }
    }
}

#[component]
pub fn DoughnutChart(
    data: Vec<Value>,
    label_key: String,
    value_key: String,
    size: Option<u32>,
) -> Element {
    let s = size.unwrap_or(180) as f64;
    let cx = s / 2.0;
    let cy = s / 2.0;
    let r = s / 2.5;
    let total: f64 = data.iter().filter_map(|d| d[&value_key].as_f64()).sum();
    let total = if total == 0.0 { 1.0 } else { total };
    let colors = [
        "#4fc3f7", "#ff7043", "#66bb6a", "#ffa726", "#ab47bc", "#26c6da",
    ];
    let circumference = 2.0 * std::f64::consts::PI * r;

    let mut offset = 0.0;
    let mut segments: Vec<Element> = Vec::new();
    let mut legend: Vec<Element> = Vec::new();

    for (i, d) in data.iter().enumerate() {
        let val = d[&value_key].as_f64().unwrap_or(0.0);
        let ratio = val / total;
        let dash_len = ratio * circumference;
        let color = colors[i % colors.len()];
        let label = d[&label_key].as_str().unwrap_or("").to_string();

        segments.push(rsx! {
            circle {
                key: "seg{i}",
                cx: "{cx}", cy: "{cy}", r: "{r}",
                fill: "none", stroke: "{color}", stroke_width: "20",
                stroke_dasharray: "{dash_len} {circumference - dash_len}",
                stroke_dashoffset: "{-offset}",
                transform: "rotate(-90 {cx} {cy})",
            }
        });

        legend.push(rsx! {
            div { key: "leg{i}", class: "legend-item",
                span { class: "legend-dot", style: "background: {color}" }
                span { class: "legend-label", "{label}" }
                span { class: "legend-value", "{val}" }
            }
        });

        offset += dash_len;
    }

    rsx! {
        div { style: "display: flex; align-items: center; gap: 1rem;",
            svg { width: "{s}", height: "{s}", view_box: "0 0 {s} {s}",
                circle { cx: "{cx}", cy: "{cy}", r: "{r}", fill: "none", stroke: "#eceff1", stroke_width: "20" }
                {segments.into_iter()}
            }
            div { class: "chart-legend", {legend.into_iter()} }
        }
    }
}
