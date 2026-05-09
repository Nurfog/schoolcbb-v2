use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectHour {
    pub level: String,
    pub hours_per_week: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubjectPayload {
    pub code: String,
    pub name: String,
    pub level: Option<String>,
    pub hours_per_week: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubjectPayload {
    pub code: Option<String>,
    pub name: Option<String>,
    pub level: Option<String>,
    pub hours_per_week: Option<i32>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePeriodPayload {
    pub name: String,
    pub year: i32,
    pub semester: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePeriodPayload {
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_active: Option<bool>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCourseSubjectPayload {
    pub course_id: Uuid,
    pub subject_id: Uuid,
    pub teacher_id: Uuid,
    pub academic_year: i32,
    pub hours_per_week: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCourseSubjectPayload {
    pub teacher_id: Option<Uuid>,
    pub hours_per_week: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct GradeCategory {
    pub id: Uuid,
    pub course_subject_id: Uuid,
    pub name: String,
    pub weight_percentage: f64,
    pub evaluation_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryPayload {
    pub course_subject_id: Uuid,
    pub name: String,
    pub weight_percentage: f64,
    pub evaluation_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryPayload {
    pub name: Option<String>,
    pub weight_percentage: Option<f64>,
    pub evaluation_count: Option<i32>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGradePayload {
    pub grade: Option<f64>,
    pub grade_type: Option<String>,
    pub category_id: Option<Uuid>,
    pub observation: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentGradeEntry {
    pub student_id: Uuid,
    pub grade: f64,
    pub observation: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category_name: String,
    pub weight: f64,
    pub grades: Vec<f64>,
    pub category_average: f64,
    pub weighted_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyReport {
    pub student_id: Uuid,
    pub student_name: String,
    pub year: i32,
    pub first_semester: SemesterReport,
    pub second_semester: Option<SemesterReport>,
    pub final_promotion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemesterReport {
    pub semester: i32,
    pub subjects: Vec<WeightedSubjectAverage>,
    pub global_average: f64,
    pub is_promoted: bool,
    pub has_minimum_grades: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseSubjectGradeEntry {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub grades: Vec<f64>,
    pub average: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AcademicYear {
    pub id: Uuid,
    pub year: i32,
    pub name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAcademicYearPayload {
    pub year: i32,
    pub name: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAcademicYearPayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneYearPayload {
    pub from_year: i32,
    pub to_year: i32,
    pub to_year_name: Option<String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGradeLevelPayload {
    pub code: String,
    pub name: String,
    pub plan: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGradeLevelPayload {
    pub name: Option<String>,
    pub plan: Option<String>,
    pub sort_order: Option<i32>,
    pub active: Option<bool>,
}
