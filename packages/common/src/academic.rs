use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Asignatura o subsector del plan de estudios.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Subject {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub level: Option<String>,
    pub hours_per_week: i32,
    pub active: bool,
}

/// Horas pedagógicas semanales por nivel para una asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectHour {
    pub level: String,
    pub hours_per_week: i32,
}

/// Payload para crear una nueva asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubjectPayload {
    pub code: String,
    pub name: String,
    pub level: Option<String>,
    pub hours_per_week: Option<i32>,
}

/// Payload para actualizar una asignatura existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubjectPayload {
    pub code: Option<String>,
    pub name: Option<String>,
    pub level: Option<String>,
    pub hours_per_week: Option<i32>,
}

/// Período académico (semestre o trimestre).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AcademicPeriod {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub semester: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
}

/// Payload para crear un nuevo período académico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePeriodPayload {
    pub name: String,
    pub year: i32,
    pub semester: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Payload para actualizar un período académico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePeriodPayload {
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

/// Asignación de una asignatura a un curso con un profesor responsable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct CourseSubject {
    pub id: Uuid,
    pub course_id: Uuid,
    pub subject_id: Uuid,
    pub teacher_id: Uuid,
    pub academic_year: i32,
    pub hours_per_week: i32,
}

/// Payload para asignar una asignatura a un curso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCourseSubjectPayload {
    pub course_id: Uuid,
    pub subject_id: Uuid,
    pub teacher_id: Uuid,
    pub academic_year: i32,
    pub hours_per_week: Option<i32>,
}

/// Payload para modificar la asignación de una asignatura a un curso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCourseSubjectPayload {
    pub teacher_id: Option<Uuid>,
    pub hours_per_week: Option<i32>,
}

/// Categoría de evaluación dentro de una asignatura (ej: "Tareas", "Pruebas", "Disertaciones").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct GradeCategory {
    pub id: Uuid,
    pub course_subject_id: Uuid,
    pub name: String,
    pub weight_percentage: f64,
    pub evaluation_count: i32,
}

/// Payload para crear una nueva categoría de evaluación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryPayload {
    pub course_subject_id: Uuid,
    pub name: String,
    pub weight_percentage: f64,
    pub evaluation_count: Option<i32>,
}

/// Payload para actualizar una categoría de evaluación.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryPayload {
    pub name: Option<String>,
    pub weight_percentage: Option<f64>,
    pub evaluation_count: Option<i32>,
}

/// Payload para registrar una calificación individual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradePayload {
    pub student_id: Uuid,
    pub course_subject_id: Uuid,
    pub grade: f64,
    pub grade_type: String,
    pub semester: i32,
    pub year: i32,
    pub date: NaiveDate,
    pub teacher_id: Uuid,
    pub category_id: Option<Uuid>,
    pub observation: Option<String>,
}

/// Payload para modificar una calificación existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGradePayload {
    pub grade: Option<f64>,
    pub grade_type: Option<String>,
    pub category_id: Option<Uuid>,
    pub observation: Option<String>,
}

/// Entrada de carga masiva de calificaciones para un curso-asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkGradeEntry {
    pub course_subject_id: Uuid,
    pub semester: i32,
    pub year: i32,
    pub date: NaiveDate,
    pub teacher_id: Uuid,
    pub grade_type: String,
    pub category_id: Option<Uuid>,
    pub grades: Vec<StudentGradeEntry>,
}

/// Calificación individual dentro de una carga masiva.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentGradeEntry {
    pub student_id: Uuid,
    pub grade: f64,
    pub observation: Option<String>,
}

/// Promedio ponderado de un estudiante en una asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedSubjectAverage {
    pub subject_name: String,
    pub subject_code: String,
    pub categories: Vec<CategoryBreakdown>,
    pub weighted_average: f64,
    pub simple_average: f64,
    pub grades_count: i32,
    pub min_grade: f64,
    pub max_grade: f64,
}

/// Desglose de una categoría dentro del promedio ponderado de una asignatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category_name: String,
    pub weight: f64,
    pub grades: Vec<f64>,
    pub category_average: f64,
    pub weighted_contribution: f64,
}

/// Reporte anual completo de calificaciones y promoción de un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyReport {
    pub student_id: Uuid,
    pub student_name: String,
    pub year: i32,
    pub first_semester: SemesterReport,
    pub second_semester: Option<SemesterReport>,
    pub final_promotion: String,
}

/// Reporte semestral de calificaciones de un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemesterReport {
    pub semester: i32,
    pub subjects: Vec<WeightedSubjectAverage>,
    pub global_average: f64,
    pub is_promoted: bool,
    pub has_minimum_grades: bool,
}

/// Entrada de calificaciones de un estudiante para una asignatura-curso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSubjectGradeEntry {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub grades: Vec<f64>,
    pub average: f64,
}

/// Año académico o año escolar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AcademicYear {
    pub id: Uuid,
    pub year: i32,
    pub name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Payload para crear un nuevo año académico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAcademicYearPayload {
    pub year: i32,
    pub name: String,
    pub is_active: Option<bool>,
}

/// Payload para actualizar un año académico existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAcademicYearPayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

/// Payload para clonar la estructura de un año académico a otro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneYearPayload {
    pub from_year: i32,
    pub to_year: i32,
    pub to_year_name: Option<String>,
}

/// Nivel de enseñanza (1° Básico, 2° Medio, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct GradeLevel {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub plan: Option<String>,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Payload para crear un nuevo nivel de enseñanza.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeLevelPayload {
    pub code: String,
    pub name: String,
    pub plan: Option<String>,
    pub sort_order: Option<i32>,
}

/// Payload para actualizar un nivel de enseñanza existente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGradeLevelPayload {
    pub name: Option<String>,
    pub plan: Option<String>,
    pub sort_order: Option<i32>,
    pub active: Option<bool>,
}
