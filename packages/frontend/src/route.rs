use dioxus::prelude::*;

use crate::api::client;
use crate::seo::use_page_title;
use crate::components::pages::academic_years_page::AcademicYearsPage;
use crate::components::pages::admission_page::AdmissionPage;
use crate::components::pages::agenda_page::AgendaPage;
use crate::components::pages::attendance_page::AttendancePage;
use crate::components::pages::audit_page::AuditPage;
use crate::components::pages::classrooms_page::ClassroomsPage;
use crate::components::pages::complaints_page::ComplaintsPage;
use crate::components::pages::config_page::ConfigPage;
use crate::components::pages::corporations_page::CorporationsPage;
use crate::components::pages::courses_page::CoursesPage;
use crate::components::pages::csv_import_page::CsvImportPage;
use crate::components::pages::employee_portal_page::EmployeePortalPage;
use crate::components::pages::enrollments_page::EnrollmentsPage;
use crate::components::pages::finance_page::FinancePage;
use crate::components::pages::grade_levels_page::GradeLevelsPage;
use crate::components::pages::grades_page::GradesPage;
use crate::components::pages::hr_detail_page::HrDetailPage;
use crate::components::pages::hr_page::HrPage;
use crate::components::pages::login_page::LoginPage;
use crate::components::pages::module_manager::ModuleManager;
use crate::components::pages::notifications_page::NotificationsPage;
use crate::components::pages::payroll_page::PayrollPage;
use crate::components::pages::reports_page::ReportsPage;
use crate::components::pages::roles_page::RolesPage;
use crate::components::pages::sige_page::SigePage;
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
    #[route("/agenda")]
    Agenda {},
    #[route("/academic-years")]
    AcademicYears {},
    #[route("/audit")]
    Audit {},
    #[route("/grade-levels")]
    GradeLevels {},
    #[route("/roles")]
    Roles {},
    #[route("/classrooms")]
    Classrooms {},
    #[route("/payroll")]
    Payroll {},
    #[route("/my-portal")]
    EmployeePortal {},
    #[route("/sige")]
    Sige {},
    #[route("/complaints")]
    Complaints {},
}

#[component]
pub fn Login() -> Element {
    use_page_title("Iniciar Sesion");
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
    use_page_title("Gestion de Alumnos");
    rsx! { StudentsPage {} }
}

#[component]
pub fn Attendance() -> Element {
    require_auth();
    use_page_title("Asistencia");
    rsx! { AttendancePage {} }
}

#[component]
pub fn Grades() -> Element {
    require_auth();
    use_page_title("Calificaciones");
    rsx! { GradesPage {} }
}

#[component]
pub fn Notifications() -> Element {
    require_auth();
    use_page_title("Centro de Mensajeria");
    rsx! { NotificationsPage {} }
}

#[component]
pub fn Reports() -> Element {
    require_auth();
    use_page_title("Reportes");
    rsx! { ReportsPage {} }
}

#[component]
pub fn Finance() -> Element {
    require_auth();
    use_page_title("Finanzas");
    rsx! { FinancePage {} }
}

#[component]
pub fn Users() -> Element {
    require_auth();
    use_page_title("Usuarios y Perfiles");
    rsx! { UsersPage {} }
}

#[component]
pub fn Courses() -> Element {
    require_auth();
    use_page_title("Cursos");
    rsx! { CoursesPage {} }
}

#[component]
pub fn Enrollments() -> Element {
    require_auth();
    use_page_title("Matriculas");
    rsx! { EnrollmentsPage {} }
}

#[component]
pub fn Subjects() -> Element {
    require_auth();
    use_page_title("Asignaturas");
    rsx! { SubjectsPage {} }
}

#[component]
pub fn Config() -> Element {
    require_auth();
    use_page_title("Configuracion");
    rsx! { ConfigPage {} }
}

#[component]
pub fn Admission() -> Element {
    require_auth();
    use_page_title("Admisiones");
    rsx! { AdmissionPage {} }
}

#[component]
pub fn Hr() -> Element {
    require_auth();
    use_page_title("Recursos Humanos");
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
    use_page_title("Importacion CSV");
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
    use_page_title("Corporaciones y Colegios");
    rsx! { CorporationsPage {} }
}

#[component]
pub fn Agenda() -> Element {
    require_auth();
    use_page_title("Agenda Escolar");
    rsx! { AgendaPage {} }
}

#[component]
pub fn AcademicYears() -> Element {
    require_auth();
    use_page_title("Anos Academicos");
    rsx! { AcademicYearsPage {} }
}

#[component]
pub fn Audit() -> Element {
    require_auth();
    use_page_title("Auditoria");
    rsx! { AuditPage {} }
}

#[component]
pub fn GradeLevels() -> Element {
    require_auth();
    use_page_title("Niveles");
    rsx! { GradeLevelsPage {} }
}

#[component]
pub fn Roles() -> Element {
    require_auth();
    use_page_title("Roles y Permisos");
    rsx! { RolesPage {} }
}

#[component]
pub fn Classrooms() -> Element {
    require_auth();
    use_page_title("Salas");
    rsx! { ClassroomsPage {} }
}

#[component]
pub fn Payroll() -> Element {
    require_auth();
    use_page_title("Remuneraciones");
    rsx! { PayrollPage {} }
}

#[component]
pub fn EmployeePortal() -> Element {
    require_auth();
    use_page_title("Mi Portal");
    rsx! { EmployeePortalPage {} }
}

#[component]
pub fn Sige() -> Element {
    require_auth();
    use_page_title("SIGE - Exportacion MINEDUC");
    rsx! { SigePage {} }
}

#[component]
pub fn Complaints() -> Element {
    require_auth();
    use_page_title("Ley Karin - Canal de Denuncias");
    rsx! { ComplaintsPage {} }
}
