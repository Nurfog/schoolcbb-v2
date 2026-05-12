use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Crear,
    Leer,
    Actualizar,
    Eliminar,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Crear => "create",
            Action::Leer => "read",
            Action::Actualizar => "update",
            Action::Eliminar => "delete",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Module {
    Students,
    Courses,
    Enrollments,
    Subjects,
    GradeLevels,
    AcademicYears,
    Classrooms,
    Attendance,
    Grades,
    HR,
    Payroll,
    MyPortal,
    Finance,
    Admission,
    Reports,
    Notifications,
    Agenda,
    Users,
    Roles,
    Config,
    Corporations,
    Audit,
    SIGE,
    Complaints,
}

impl Module {
    pub fn as_str(&self) -> &'static str {
        match self {
            Module::Students => "students",
            Module::Courses => "courses",
            Module::Enrollments => "enrollments",
            Module::Subjects => "subjects",
            Module::GradeLevels => "grade-levels",
            Module::AcademicYears => "academic-years",
            Module::Classrooms => "classrooms",
            Module::Attendance => "attendance",
            Module::Grades => "grades",
            Module::HR => "hr",
            Module::Payroll => "payroll",
            Module::MyPortal => "my-portal",
            Module::Finance => "finance",
            Module::Admission => "admission",
            Module::Reports => "reports",
            Module::Notifications => "notifications",
            Module::Agenda => "agenda",
            Module::Users => "users",
            Module::Roles => "roles",
            Module::Config => "config",
            Module::Corporations => "corporations",
            Module::Audit => "audit",
            Module::SIGE => "sige",
            Module::Complaints => "complaints",
        }
    }

    pub fn all() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("students", vec!["view", "create", "edit", "delete", "import", "export"]),
            ("courses", vec!["view", "create", "edit", "delete"]),
            ("enrollments", vec!["view", "create", "edit", "delete", "manage"]),
            ("subjects", vec!["view", "create", "edit", "delete"]),
            ("grade-levels", vec!["view", "create", "edit", "delete"]),
            ("academic-years", vec!["view", "create", "edit", "delete", "activate"]),
            ("classrooms", vec!["view", "create", "edit", "delete"]),
            ("attendance", vec!["records", "reports", "alerts", "modify"]),
            ("grades", vec!["view", "create", "edit", "delete", "periods", "categories", "reports"]),
            ("hr", vec!["employees", "contracts", "documents", "leaves"]),
            ("payroll", vec!["view", "calculate", "export"]),
            ("my-portal", vec!["view"]),
            ("finance", vec!["fees", "payments", "scholarships"]),
            ("admission", vec!["prospects", "stages", "documents", "activities", "classrooms", "metrics"]),
            ("reports", vec!["certificates", "concentrations", "final-records", "sige"]),
            ("notifications", vec!["send", "view", "manage"]),
            ("agenda", vec!["events", "view", "manage"]),
            ("users", vec!["view", "create", "edit", "delete"]),
            ("roles", vec!["view", "create", "edit", "delete", "assign"]),
            ("config", vec!["branding", "preferences", "general"]),
            ("corporations", vec!["view", "create", "edit", "toggle"]),
            ("audit", vec!["view", "export"]),
            ("sige", vec!["export"]),
            ("complaints", vec!["view", "manage", "resolve"]),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct PermissionDef {
    pub id: Uuid,
    pub module: String,
    pub resource: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub permissions: Vec<ResourcePermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub permission_id: Uuid,
    pub module: String,
    pub resource: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionsPayload {
    pub permissions: Vec<PermissionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub permission_id: Uuid,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRolePayload {
    pub role_id: Uuid,
}

#[cfg(feature = "db")]
pub async fn check_permission(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    required_module: &str,
    required_action: Action,
) -> Result<bool, sqlx::Error> {
    let roles = sqlx::query_as::<_, (Uuid,)>("SELECT role_id FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    if roles.is_empty() {
        return Ok(false);
    }

    for (role_id,) in &roles {
        let permissions = sqlx::query_as::<_, (String, String, String)>(
            r#"SELECT pd.module, pd.resource, pd.action
               FROM role_permissions rp
               JOIN permission_definitions pd ON rp.permission_id = pd.id
               WHERE rp.role_id = $1 AND pd.module = $2"#,
        )
        .bind(role_id)
        .bind(required_module)
        .fetch_all(pool)
        .await?;

        for (_mod, _resource, action) in &permissions {
            if action == required_action.as_str() {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
