use dioxus::prelude::*;

use crate::api::client;
use crate::seo::use_page_title;
use crate::components::pages::academic_years_page::AcademicYearsPage;
use crate::components::pages::admission_page::AdmissionPage;
use crate::components::pages::admin_contracts_page::AdminContractsPage;
use crate::components::pages::admin_payments_page::AdminPaymentsPage;
use crate::components::pages::admin_plans_page::AdminPlansPage;
use crate::components::pages::admin_system_page::AdminSystemPage;
use crate::components::pages::agenda_page::AgendaPage;
use crate::components::pages::attendance_page::AttendancePage;
use crate::components::pages::audit_page::AuditPage;
use crate::components::pages::classrooms_page::ClassroomsPage;
use crate::components::pages::client_portal_page::ClientPortalPage;
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
use crate::components::pages::curriculum_agent::CurriculumAgent;
use crate::components::pages::dashboard_mosaicos_page::DashboardMosaicosPage;
use crate::components::pages::root_page::RootDashboard;
use crate::components::pages::sostenedor_page::SostenedorPage;
use crate::components::pages::users_page::UsersPage;

pub fn has_token() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };
    let doc = match window.document() {
        Some(d) => d,
        None => return false,
    };
    let cookie = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("cookie"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    cookie.split(';').any(|c| {
        let c = c.trim();
        c.starts_with("jwt_token=") && c.len() > "jwt_token=".len()
    })
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
    #[route("/session-login")]
    SessionLogin {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/sostenedor")]
    SostenedorPortal {},
    #[route("/root")]
    RootDashboard {},
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
    #[route("/license-portal")]
    ClientPortal {},
    #[route("/my-portal")]
    EmployeePortal {},
    #[route("/sige")]
    Sige {},
    #[route("/complaints")]
    Complaints {},
    #[route("/curriculum")]
    Curriculum {},
    #[route("/admin/plans")]
    AdminPlans {},
    #[route("/admin/contracts")]
    AdminContracts {},
    #[route("/admin/payments")]
    AdminPayments {},
    #[route("/admin/system")]
    AdminSystem {},
}

#[component]
pub fn Login() -> Element {
    use_page_title("Iniciar Sesion");
    rsx! { LoginPage {} }
}

#[component]
pub fn SessionLogin() -> Element {
    use_effect(|| {
        let window = web_sys::window();
        if let Some(w) = window {
            let params = w.location().search().ok().unwrap_or_default();
            let code = params
                .trim_start_matches('?')
                .split('&')
                .find_map(|p| p.strip_prefix("code="))
                .map(|v| urlencoding_decode(v))
                .unwrap_or_default();
            if !code.is_empty() {
                spawn(async move {
                    match crate::api::client::exchange_code(&code).await {
                        Ok(Some(data)) => {
                            if let Some(token) = data.get("token").and_then(|v| v.as_str()) {
                                if let Some(doc) = w.document() {
                                    let cookie = format!("jwt_token={}; Path=/; SameSite=Lax; Max-Age=43200", token);
                                    let _ = js_sys::Reflect::set(&doc, &wasm_bindgen::JsValue::from_str("cookie"), &wasm_bindgen::JsValue::from_str(&cookie));
                                }
                                let origin = w.location().origin().ok().unwrap_or_default();
                                let _ = w.location().set_href(&origin);
                            }
                        }
                        _ => {
                            let _ = w.location().set_href("/login?error=Ocurrió un error al iniciar sesión");
                        }
                    }
                });
            }
        }
    });
    use_page_title("Iniciando sesión...");
    rsx! { div { class: "loading-spinner", "Iniciando sesión..." } }
}

fn urlencoding_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                }
            }
            '+' => result.push(' '),
            other => result.push(other),
        }
    }
    result
}

#[component]
pub fn Dashboard() -> Element {
    require_auth();
    use_page_title("Dashboard");
    rsx! { DashboardMosaicosPage {} }
}

#[component]
pub fn RootDashboardRoute() -> Element {
    require_auth();
    use_page_title("Panel Root");
    rsx! { RootDashboard {} }
}

#[component]
pub fn ModuleManagerRoot() -> Element {
    require_auth();
    use_page_title("Inicio");
    let prefs = use_resource(|| async move { client::fetch_json("/api/user/preferences").await });
    let mut redirected = use_signal(|| false);

    match prefs() {
        Some(Ok(data)) => {
            let show = data["show_module_manager"].as_bool().unwrap_or(true);
            if show {
                rsx! { ModuleManager {} }
            } else if !*redirected.peek() {
                redirected.set(true);
                let nav = navigator();
                nav.replace("/dashboard");
                rsx! { div { class: "loading-spinner", "Redirigiendo..." } }
            } else {
                rsx! { div { class: "loading-spinner", "Redirigiendo..." } }
            }
        }
        _ => {
            rsx! { div { class: "loading-spinner", "Cargando..." } }
        }
    }
}

#[component]
pub fn SostenedorPortal() -> Element {
    require_auth();
    use_page_title("Panel del Sostenedor");
    rsx! { SostenedorPage {} }
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
    use_page_title("Detalle Empleado");
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
pub fn ClientPortal() -> Element {
    require_auth();
    use_page_title("Portal de Licencia");
    rsx! { ClientPortalPage {} }
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

#[component]
pub fn Curriculum() -> Element {
    require_auth();
    use_page_title("Currículum Nacional");
    rsx! { CurriculumAgent {} }
}

#[component]
pub fn AdminPlans() -> Element {
    require_auth();
    use_page_title("Planes - Root");
    rsx! { AdminPlansPage {} }
}

#[component]
pub fn AdminContracts() -> Element {
    require_auth();
    use_page_title("Contratos - Root");
    rsx! { AdminContractsPage {} }
}

#[component]
pub fn AdminPayments() -> Element {
    require_auth();
    use_page_title("Pagos - Root");
    rsx! { AdminPaymentsPage {} }
}

#[component]
pub fn AdminSystem() -> Element {
    require_auth();
    use_page_title("Sistema - Root");
    rsx! { AdminSystemPage {} }
}
