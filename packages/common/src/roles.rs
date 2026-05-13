use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Acción CRUD sobre un recurso del sistema.
///
/// Se usa en conjunto con [`Module`] para definir permisos de acceso.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Action {
    /// Crear nuevos registros.
    Crear,
    /// Leer o visualizar registros existentes.
    Leer,
    /// Actualizar registros existentes.
    Actualizar,
    /// Eliminar registros.
    Eliminar,
}

impl Action {
    /// Retorna la representación en inglés de la acción (`"create"`, `"read"`, `"update"`, `"delete"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Crear => "create",
            Action::Leer => "read",
            Action::Actualizar => "update",
            Action::Eliminar => "delete",
        }
    }
}

/// Módulo funcional del sistema. Cada variante representa un área de la
/// aplicación sobre la cual se pueden otorgar permisos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Module {
    /// Gestión de estudiantes.
    Students,
    /// Gestión de cursos.
    Courses,
    /// Matrículas y asignación de estudiantes a cursos.
    Enrollments,
    /// Asignaturas o subsectores del plan de estudios.
    Subjects,
    /// Niveles de enseñanza (1° Básico, 2° Medio, etc.).
    GradeLevels,
    /// Años académicos y períodos escolares.
    AcademicYears,
    /// Salas de clases.
    Classrooms,
    /// Registro y reportes de asistencia.
    Attendance,
    /// Calificaciones y evaluación.
    Grades,
    /// Recursos humanos (empleados, contratos, documentos).
    HR,
    /// Cálculo y exportación de remuneraciones.
    Payroll,
    /// Portal de apoderados y alumnos.
    MyPortal,
    /// Módulo financiero (aranceles, pagos, becas).
    Finance,
    /// Proceso de admisión y prospectos.
    Admission,
    /// Reportes, certificados y concentraciones.
    Reports,
    /// Notificaciones y comunicaciones masivas.
    Notifications,
    /// Agenda de eventos y actividades.
    Agenda,
    /// Administración de usuarios del sistema.
    Users,
    /// Roles y permisos del sistema.
    Roles,
    /// Configuración general del establecimiento.
    Config,
    /// Corporaciones o sostenedores.
    Corporations,
    /// Registro de auditoría.
    Audit,
    /// Exportación de datos a plataforma SIGE.
    SIGE,
    /// Denuncias y reclamos (Ley Karin).
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

/// Definición de un permiso atómico registrado en base de datos.
///
/// Cada permiso vincula un módulo con un recurso específico (ej: módulo
/// `students`, recurso `view`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct PermissionDef {
    pub id: Uuid,
    pub module: String,
    pub resource: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

/// Fila de un rol almacenado en base de datos.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// Rol del sistema con su lista completa de permisos asociados.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub permissions: Vec<ResourcePermission>,
}

/// Permiso específico sobre un recurso, indicando qué acciones CRUD están habilitadas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub permission_id: Uuid,
    pub module: String,
    pub resource: String,
    /// Permite crear nuevos registros.
    pub can_create: bool,
    /// Permite leer o visualizar registros.
    pub can_read: bool,
    /// Permite modificar registros existentes.
    pub can_update: bool,
    /// Permite eliminar registros.
    pub can_delete: bool,
}

/// Payload para crear un nuevo rol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRolePayload {
    pub name: String,
    pub description: Option<String>,
}

/// Payload para actualizar los permisos de un rol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionsPayload {
    pub permissions: Vec<PermissionEntry>,
}

/// Entrada individual de permiso dentro de una actualización masiva.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub permission_id: Uuid,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}

/// Payload para asignar un rol a un usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRolePayload {
    pub role_id: Uuid,
}

/// Verifica si un usuario tiene un permiso específico sobre un módulo.
///
/// Consulta los roles del usuario en base de datos y revisa si alguno
/// tiene el permiso `required_action` sobre `required_module`.
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

    let action_column = match required_action {
        Action::Crear => "can_create",
        Action::Leer => "can_read",
        Action::Actualizar => "can_update",
        Action::Eliminar => "can_delete",
    };

    for (role_id,) in &roles {
        let has_perm = sqlx::query_scalar::<_, bool>(
            &format!(
                r#"SELECT EXISTS(
                    SELECT 1 FROM role_permissions rp
                    JOIN permission_definitions pd ON rp.permission_id = pd.id
                    WHERE rp.role_id = $1 AND pd.module = $2 AND rp.{action_column} = true
                )"#,
            ),
        )
        .bind(role_id)
        .bind(required_module)
        .fetch_one(pool)
        .await?;

        if has_perm {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Helper: dado un `corporation_id` opcional (del JWT), verifica que el módulo
/// esté incluido en la licencia activa. Retorna `Ok(())` si tiene acceso,
/// o un mensaje de error si no.
pub async fn require_licensed_module(
    pool: &sqlx::PgPool,
    corporation_id: Option<&str>,
    module_key: &str,
) -> Result<(), String> {
    let cid = corporation_id.and_then(|s| Uuid::parse_str(s).ok());
    let allowed = check_license_module(pool, cid, module_key)
        .await
        .map_err(|e| format!("Error al verificar licencia: {e}"))?;
    if allowed {
        Ok(())
    } else {
        Err(format!("Módulo '{module_key}' no está incluido en el plan contratado"))
    }
}

/// Verifica si el módulo está incluido en la licencia activa de una corporación.
/// Retorna `true` si:
/// - No hay `corporation_id` (escuela individual sin licencia corporativa)
/// - La corporación tiene una licencia activa con el módulo incluido
/// - O hay un override manual habilitando el módulo para la corporación
#[cfg(feature = "db")]
pub async fn check_license_module(
    pool: &sqlx::PgPool,
    corporation_id: Option<Uuid>,
    module_key: &str,
) -> Result<bool, sqlx::Error> {
    let Some(cid) = corporation_id else {
        return Ok(true);
    };

    let has_override: Option<bool> = sqlx::query_scalar(
        "SELECT enabled FROM corporation_module_overrides WHERE corporation_id = $1 AND module_key = $2",
    )
    .bind(cid)
    .bind(module_key)
    .fetch_optional(pool)
    .await?;

    if let Some(enabled) = has_override {
        return Ok(enabled);
    }

    let included: Option<bool> = sqlx::query_scalar(
        r#"SELECT pm.included FROM corporation_licenses cl
           JOIN license_plans lp ON lp.id = cl.plan_id
           JOIN plan_modules pm ON pm.plan_id = lp.id
           WHERE cl.corporation_id = $1 AND cl.status = 'active' AND pm.module_key = $2
           LIMIT 1"#,
    )
    .bind(cid)
    .bind(module_key)
    .fetch_optional(pool)
    .await?;

    Ok(included.unwrap_or(true))
}
