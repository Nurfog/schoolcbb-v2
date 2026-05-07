use schoolcbb_proto::academic_service_server::AcademicService;
use schoolcbb_proto::{
    CoursePerformanceRequest, CoursePerformanceResponse, SemesterData, StudentReportRequest,
    StudentReportResponse, Subject, SubjectGrade, SubjectListRequest, SubjectListResponse,
};
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct AcademicGrpc {
    pub pool: PgPool,
}

#[tonic::async_trait]
impl AcademicService for AcademicGrpc {
    async fn get_student_report(
        &self,
        req: Request<StudentReportRequest>,
    ) -> Result<Response<StudentReportResponse>, Status> {
        let inner = req.into_inner();
        let student_id = Uuid::parse_str(&inner.student_id)
            .map_err(|_| Status::invalid_argument("Invalid student_id"))?;

        let name: Option<(String,)> = sqlx::query_as(
            "SELECT CONCAT(first_name, ' ', last_name) FROM students WHERE id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let student_name = name.map(|r| r.0).unwrap_or_default();

        let s1 = build_semester_grpc(&self.pool, student_id, 1, inner.year).await;
        let s2 = build_semester_grpc(&self.pool, student_id, 2, inner.year).await;

        let final_promotion = match (&s1, &s2) {
            (Some(s1_data), Some(s2_data)) => {
                let avg = (s1_data.global_average + s2_data.global_average) / 2.0;
                if avg >= 4.0 { "Promovido" } else { "Reprobado" }
            }
            _ => "Pendiente",
        };

        let semesters: Vec<SemesterData> = vec![s1, s2].into_iter().flatten().collect();

        Ok(Response::new(StudentReportResponse {
            student_id: inner.student_id,
            student_name,
            global_average: semesters.first().map(|s| s.global_average).unwrap_or(0.0),
            final_promotion: final_promotion.to_string(),
            semesters,
        }))
    }

    async fn get_subject_list(
        &self,
        _req: Request<SubjectListRequest>,
    ) -> Result<Response<SubjectListResponse>, Status> {
        let rows: Vec<(String, String, String, Option<String>, i32, bool)> = sqlx::query_as(
            "SELECT id::text, code, name, level, hours_per_week, active FROM subjects ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let subjects = rows
            .into_iter()
            .map(|(id, code, name, level, hours, active)| Subject {
                id,
                code,
                name,
                level: level.unwrap_or_default(),
                hours_per_week: hours,
                active,
            })
            .collect();

        Ok(Response::new(SubjectListResponse { subjects }))
    }

    async fn get_course_performance(
        &self,
        req: Request<CoursePerformanceRequest>,
    ) -> Result<Response<CoursePerformanceResponse>, Status> {
        let inner = req.into_inner();
        Ok(Response::new(CoursePerformanceResponse {
            course_id: inner.course_id,
            year: inner.year,
            subjects: vec![],
        }))
    }
}

async fn build_semester_grpc(
    pool: &PgPool,
    student_id: Uuid,
    semester: i32,
    year: i32,
) -> Option<SemesterData> {
    let rows: Vec<(String, String, f64, i32, f64, f64)> = sqlx::query_as(
        r#"
        SELECT COALESCE(s.name, g.subject), COALESCE(s.code, ''),
               ROUND(AVG(g.grade)::numeric, 1)::float8 as avg,
               COUNT(*)::int as cnt,
               ROUND(MIN(g.grade)::numeric, 1)::float8 as min_g,
               ROUND(MAX(g.grade)::numeric, 1)::float8 as max_g
        FROM grades g
        LEFT JOIN course_subjects cs ON cs.id = g.course_subject_id
        LEFT JOIN subjects s ON s.id = cs.subject_id
        WHERE g.student_id = $1 AND g.semester = $2 AND g.year = $3
        GROUP BY s.name, s.code, g.subject
        ORDER BY s.name
        "#,
    )
    .bind(student_id)
    .bind(semester)
    .bind(year)
    .fetch_all(pool)
    .await
    .ok()?;

    if rows.is_empty() {
        return None;
    }

    let subjects: Vec<SubjectGrade> = rows
        .into_iter()
        .map(|(name, code, avg, cnt, min_g, max_g)| SubjectGrade {
            subject_name: name,
            subject_code: code,
            average: avg,
            grades_count: cnt,
            min_grade: min_g,
            max_grade: max_g,
        })
        .collect();

    let global_avg = subjects.iter().map(|s| s.average).sum::<f64>() / subjects.len() as f64;

    Some(SemesterData {
        semester,
        global_average: (global_avg * 10.0).round() / 10.0,
        subjects,
    })
}
