use dioxus::prelude::*;

use crate::api::client;
use crate::components::pages::admission_page::AdmissionPage;
use crate::components::pages::attendance_page::AttendancePage;
use crate::components::pages::config_page::ConfigPage;
use crate::components::pages::corporations_page::CorporationsPage;
use crate::components::pages::courses_page::CoursesPage;
use crate::components::pages::csv_import_page::CsvImportPage;
use crate::components::pages::enrollments_page::EnrollmentsPage;
use crate::components::pages::finance_page::FinancePage;
use crate::components::pages::grades_page::GradesPage;
use crate::components::pages::hr_detail_page::HrDetailPage;
use crate::components::pages::hr_page::HrPage;
use crate::components::pages::login_page::LoginPage;
use crate::components::pages::module_manager::ModuleManager;
use crate::components::pages::notifications_page::NotificationsPage;
use crate::components::pages::reports_page::ReportsPage;
use crate::components::pages::student_detail_page::StudentDetailPage;
use crate::components::pages::students_page::StudentsPage;
use crate::components::pages::subjects_page::SubjectsPage;
use crate::components::pages::users_page::UsersPage;

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
    #[route("/reports")]
    Reports {},
    #[route("/finance")]
    Finance {},
    #[route("/users")]
    Users {},
    #[route("/courses")]
    Courses {},
    #[route("/enrollments")]
    Enrollments {},
    #[route("/subjects")]
    Subjects {},
    #[route("/config")]
    Config {},
    #[route("/admission")]
    Admission {},
    #[route("/hr")]
    Hr {},
    #[route("/hr/{employee_id}")]
    HrDetail { employee_id: String },
    #[route("/import")]
    Import {},
    #[route("/corporations")]
    Corporations {},
}

#[component]
pub fn Login() -> Element {
    rsx! { LoginPage {} }
}

#[component]
pub fn ModuleManagerRoot() -> Element {
    require_auth();
    let prefs = use_resource(|| async move { client::fetch_json("/api/user/preferences").await });
    let mut navigated = use_signal(|| false);

    match prefs() {
        Some(Ok(data)) => {
            let show = data["show_module_manager"].as_bool().unwrap_or(true);
            if !show && !navigated() {
                navigated.set(true);
                let nav = navigator();
                nav.replace("/dashboard");
            }
            if show {
                rsx! { ModuleManager {} }
            } else {
                rsx! { div { class: "loading-spinner", "Redirigiendo..." } }
            }
        }
        _ => rsx! { div { class: "loading-spinner", "Cargando..." } },
    }
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

#[component]
pub fn Courses() -> Element {
    require_auth();
    rsx! { CoursesPage {} }
}

#[component]
pub fn Enrollments() -> Element {
    require_auth();
    rsx! { EnrollmentsPage {} }
}

#[component]
pub fn Subjects() -> Element {
    require_auth();
    rsx! { SubjectsPage {} }
}

#[component]
pub fn Config() -> Element {
    require_auth();
    rsx! { ConfigPage {} }
}

#[component]
pub fn Admission() -> Element {
    require_auth();
    rsx! { AdmissionPage {} }
}

#[component]
pub fn Hr() -> Element {
    require_auth();
    rsx! { HrPage {} }
}

#[component]
pub fn HrDetail(employee_id: String) -> Element {
    require_auth();
    rsx! { HrDetailPage { employee_id: employee_id } }
}

#[component]
pub fn Import() -> Element {
    use dioxus::prelude::*;
    let mut entity_type = use_signal(|| "employees".to_string());
    rsx! {
        div { class: "page-toolbar",
            button { class: if entity_type() == "employees" { "btn btn-primary" } else { "btn" }, onclick: move |_| entity_type.set("employees".to_string()), "Empleados" }
            button { class: if entity_type() == "students" { "btn btn-primary" } else { "btn" }, onclick: move |_| entity_type.set("students".to_string()), "Alumnos" }
        }
        CsvImportPage { entity_type: entity_type() }
    }
}

#[component]
pub fn Corporations() -> Element {
    require_auth();
    rsx! { CorporationsPage {} }
}
