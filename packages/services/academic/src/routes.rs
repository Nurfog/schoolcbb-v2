use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;
use uuid::Uuid;

use crate::error::AcademicResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/grades/student/{student_id}/{semester}/{year}", get(student_grades))
}

async fn student_grades(
    State(state): State<AppState>,
    axum::extract::Path((student_id, semester, year)): axum::extract::Path<(Uuid, i32, i32)>,
) -> AcademicResult<Json<Value>> {
    let report = get_student_grades(&state.pool, student_id, semester, year).await?;
    Ok(Json(serde_json::to_value(report).unwrap()))
}

use schoolcbb_common::grades::{Semester, StudentGradeReport, SubjectAverage};

async fn get_student_grades(
    pool: &sqlx::PgPool,
    student_id: Uuid,
    semester: i32,
    year: i32,
) -> Result<StudentGradeReport, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawSubjectAverage {
        subject: String,
        average: f64,
        grades_count: i32,
        min_grade: f64,
        max_grade: f64,
    }

    let subjects: Vec<RawSubjectAverage> = sqlx::query_as::<_, RawSubjectAverage>(
        r#"
        SELECT
            subject,
            AVG(grade) as average,
            COUNT(*)::int as grades_count,
            MIN(grade) as min_grade,
            MAX(grade) as max_grade
        FROM grades
        WHERE student_id = $1 AND semester = $2 AND year = $3
        GROUP BY subject
        ORDER BY subject
        "#,
    )
    .bind(student_id)
    .bind(semester)
    .bind(year)
    .fetch_all(pool)
    .await?;

    let student: (String,) = sqlx::query_as(
        "SELECT CONCAT(first_name, ' ', last_name) FROM students WHERE id = $1",
    )
    .bind(student_id)
    .fetch_one(pool)
    .await?;

    let subject_averages: Vec<SubjectAverage> = subjects
        .into_iter()
        .map(|s| SubjectAverage {
            subject: s.subject,
            average: s.average,
            grades_count: s.grades_count,
            min_grade: s.min_grade,
            max_grade: s.max_grade,
        })
        .collect();

    let global_average = StudentGradeReport::calculate(&subject_averages);
    let is_promoted = global_average >= 4.0;

    Ok(StudentGradeReport {
        student_id,
        student_name: student.0,
        semester: if semester == 1 {
            Semester::First
        } else {
            Semester::Second
        },
        year,
        subjects: subject_averages,
        global_average,
        is_promoted,
    })
}
