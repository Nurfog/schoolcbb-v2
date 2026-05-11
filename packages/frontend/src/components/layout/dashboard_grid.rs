use dioxus::prelude::*;

use crate::components::widgets::{
    academic_performance::AcademicPerformanceWidget, agenda::AgendaWidget,
    attendance_today::AttendanceTodayWidget, student_alerts::StudentAlertsWidget,
};

#[component]
pub fn DashboardGrid() -> Element {
    rsx! {
        div { class: "dashboard-header",
            h1 { "Panel de Control" }
            p { "Resumen general del sistema escolar" }
        }
        div { class: "dashboard-grid",
            AttendanceTodayWidget {}
            AcademicPerformanceWidget {}
            StudentAlertsWidget {}
            AgendaWidget {}
        }
    }
}
