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
    Dashboard,
    Academic,
    Attendance,
    SIS,
    Finance,
    Communications,
    Admission,
    Reports,
    Config,
    Users,
}

impl Module {
    pub fn as_str(&self) -> &'static str {
        match self {
            Module::Dashboard => "Dashboard",
            Module::Academic => "Academic",
            Module::Attendance => "Attendance",
            Module::SIS => "SIS",
            Module::Finance => "Finance",
            Module::Communications => "Communications",
            Module::Admission => "Admission",
            Module::Reports => "Reports",
            Module::Config => "Config",
            Module::Users => "Users",
        }
    }

    pub fn all() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("Dashboard", vec!["panel", "kpis"]),
            ("Academic", vec!["subjects", "grades", "periods", "categories", "reports", "academic_years", "grade_levels"]),
            ("Attendance", vec!["records", "alerts", "reports"]),
            ("SIS", vec!["students", "courses", "enrollments", "guardians"]),
            ("Finance", vec!["fees", "payments", "scholarships"]),
            ("Communications", vec!["messages", "interviews", "notifications"]),
            ("Admission", vec!["prospects", "stages", "documents", "activities", "classrooms"]),
            ("Reports", vec!["certificates", "concentrations", "final_records", "sige"]),
            ("Config", vec!["branding", "preferences"]),
            ("Users", vec!["users", "roles", "permissions"]),
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
    let roles = sqlx::query_as::<_, (Uuid,)>(
        "SELECT role_id FROM user_roles WHERE user_id = $1",
    )
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
