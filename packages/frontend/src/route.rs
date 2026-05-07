use dioxus::prelude::*;

use crate::components::layout::dashboard_grid::DashboardGrid;
use crate::components::pages::students_page::StudentsPage;
use crate::components::pages::attendance_page::AttendancePage;
use crate::components::pages::grades_page::GradesPage;
use crate::components::pages::student_detail_page::StudentDetailPage;
use crate::components::pages::notifications_page::NotificationsPage;
use crate::components::pages::agenda_page::AgendaPage;
use crate::components::pages::reports_page::ReportsPage;
use crate::components::pages::finance_page::FinancePage;
use crate::components::pages::users_page::UsersPage;
use crate::components::pages::module_manager::ModuleManager;
use crate::components::pages::login_page::LoginPage;

pub fn has_token() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("jwt_token").ok())
        .flatten()
        .map(|t| !t.is_empty())
        .unwrap_or(false)
}

fn require_auth() {
    if !has_token() {
        let nav = navigator();
        nav.push("/login");
    }
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login")]
    Login {},
    #[route("/")]
    ModuleManagerRoot {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/students")]
    Students {},
    #[route("/students/{student_id}")]
    StudentDetailPage { student_id: String },
    #[route("/attendance")]
    Attendance {},
    #[route("/grades")]
    Grades {},
    #[route("/notifications")]
    Notifications {},
    #[route("/agenda")]
    Agenda {},
    #[route("/reports")]
    Reports {},
    #[route("/finance")]
    Finance {},
    #[route("/users")]
    Users {},
}

#[component]
pub fn Login() -> Element {
    rsx! { LoginPage {} }
}

#[component]
pub fn ModuleManagerRoot() -> Element {
    require_auth();
    rsx! { ModuleManager {} }
}

#[component]
pub fn Dashboard() -> Element {
    require_auth();
    rsx! { DashboardGrid {} }
}

#[component]
pub fn Students() -> Element {
    require_auth();
    rsx! { StudentsPage {} }
}

#[component]
pub fn Attendance() -> Element {
    require_auth();
    rsx! { AttendancePage {} }
}

#[component]
pub fn Grades() -> Element {
    require_auth();
    rsx! { GradesPage {} }
}

#[component]
pub fn Notifications() -> Element {
    require_auth();
    rsx! { NotificationsPage {} }
}

#[component]
pub fn Agenda() -> Element {
    require_auth();
    rsx! { AgendaPage {} }
}

#[component]
pub fn Reports() -> Element {
    require_auth();
    rsx! { ReportsPage {} }
}

#[component]
pub fn Finance() -> Element {
    require_auth();
    rsx! { FinancePage {} }
}

#[component]
pub fn Users() -> Element {
    require_auth();
    rsx! { UsersPage {} }
}
