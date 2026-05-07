use dioxus::prelude::*;

use crate::components::widgets::{
    attendance_today::AttendanceTodayWidget,
    student_alerts::StudentAlertsWidget,
    agenda::AgendaWidget,
    academic_performance::AcademicPerformanceWidget,
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
