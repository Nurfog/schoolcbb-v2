use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;

use crate::error::AttendanceResult;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/attendance/today", get(today))
        .route("/api/attendance/monthly/{year}/{month}", get(monthly))
        .route("/api/attendance/alerts", get(alerts))
}

async fn today(State(state): State<AppState>) -> AttendanceResult<Json<Value>> {
    let today = chrono::Utc::now().date_naive().to_string();
    let records = get_attendance_today(&state.pool, &today).await?;
    Ok(Json(serde_json::to_value(records).unwrap()))
}

async fn monthly(
    State(state): State<AppState>,
    axum::extract::Path((year, month)): axum::extract::Path<(i32, u32)>,
) -> AttendanceResult<Json<Value>> {
    let data = get_monthly_summary(&state.pool, year, month).await?;
    Ok(Json(serde_json::to_value(data).unwrap()))
}

async fn alerts(State(state): State<AppState>) -> AttendanceResult<Json<Value>> {
    let alerts = get_attendance_alerts(&state.pool).await?;
    Ok(Json(serde_json::to_value(alerts).unwrap()))
}

use schoolcbb_common::attendance::{AlertSeverity, AttendanceAlert, AttendanceStatus, DailyAttendance, MonthlyAttendanceSummary};
use uuid::Uuid;

async fn get_attendance_today(
    pool: &sqlx::PgPool,
    date: &str,
) -> Result<Vec<DailyAttendance>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawAttendance {
        id: Uuid,
        student_id: Uuid,
        course_id: Uuid,
        date: chrono::NaiveDate,
        time: Option<chrono::NaiveTime>,
        status: String,
        subject: String,
        teacher_id: Uuid,
        observation: Option<String>,
    }

    let raw = sqlx::query_as::<_, RawAttendance>(
        r#"
        SELECT id, student_id, course_id, date, time, status,
               subject, teacher_id, observation
        FROM attendance WHERE date = $1::date
        "#,
    )
    .bind(date)
    .fetch_all(pool)
    .await?;

    Ok(raw
        .into_iter()
        .map(|r| DailyAttendance {
            id: r.id,
            student_id: r.student_id,
            course_id: r.course_id,
            date: r.date,
            time: r.time,
            status: match r.status.as_str() {
                "Ausente" => AttendanceStatus::Ausente,
                "Atraso" => AttendanceStatus::Atraso,
                "Justificado" => AttendanceStatus::Justificado,
                "Licencia" => AttendanceStatus::Licencia,
                _ => AttendanceStatus::Presente,
            },
            subject: r.subject,
            teacher_id: r.teacher_id,
            observation: r.observation,
        })
        .collect())
}

async fn get_monthly_summary(
    pool: &sqlx::PgPool,
    year: i32,
    month: u32,
) -> Result<Vec<MonthlyAttendanceSummary>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawSummary {
        student_id: Uuid,
        student_name: String,
        rut: String,
        total_days: i32,
        present: i32,
        absent: i32,
        late: i32,
        justified: i32,
    }

    let raw = sqlx::query_as::<_, RawSummary>(
        r#"
        WITH monthly AS (
            SELECT
                s.id as student_id,
                CONCAT(s.first_name, ' ', s.last_name) as student_name,
                s.rut,
                COUNT(*) as total_days,
                COUNT(*) FILTER (WHERE a.status = 'Presente') as present,
                COUNT(*) FILTER (WHERE a.status = 'Ausente') as absent,
                COUNT(*) FILTER (WHERE a.status = 'Atraso') as late,
                COUNT(*) FILTER (WHERE a.status = 'Justificado') as justified
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            WHERE EXTRACT(YEAR FROM a.date) = $1
              AND EXTRACT(MONTH FROM a.date) = $2
              AND s.enrolled = true
            GROUP BY s.id, s.first_name, s.last_name, s.rut
        )
        SELECT * FROM monthly ORDER BY student_name
        "#,
    )
    .bind(year)
    .bind(month as i32)
    .fetch_all(pool)
    .await?;

    Ok(raw
        .into_iter()
        .map(|r| MonthlyAttendanceSummary {
            student_id: r.student_id,
            student_name: r.student_name,
            rut: r.rut,
            year,
            month,
            total_days: r.total_days,
            present: r.present,
            absent: r.absent,
            late: r.late,
            justified: r.justified,
        })
        .collect())
}

async fn get_attendance_alerts(
    pool: &sqlx::PgPool,
) -> Result<Vec<AttendanceAlert>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawAlert {
        student_id: Uuid,
        student_name: String,
        rut: String,
        month: i32,
        year: i32,
        attendance_percentage: f64,
        total_absences: i64,
        severity: String,
    }

    let raw = sqlx::query_as::<_, RawAlert>(
        r#"
        WITH recent AS (
            SELECT
                s.id as student_id,
                CONCAT(s.first_name, ' ', s.last_name) as student_name,
                s.rut,
                COUNT(*) FILTER (WHERE a.status = 'Ausente') as total_absences,
                COUNT(*) as total_days
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            WHERE a.date >= CURRENT_DATE - INTERVAL '30 days'
              AND s.enrolled = true
            GROUP BY s.id, s.first_name, s.last_name, s.rut
        )
        SELECT
            student_id, student_name, rut,
            EXTRACT(MONTH FROM CURRENT_DATE)::int as month,
            EXTRACT(YEAR FROM CURRENT_DATE)::int as year,
            CASE WHEN total_days > 0
                THEN (1.0 - total_absences::float / total_days) * 100
                ELSE 100.0
            END as attendance_percentage,
            total_absences,
            CASE
                WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 85 THEN 'Alto'
                WHEN (1.0 - total_absences::float / GREATEST(total_days, 1)) * 100 < 90 THEN 'Medio'
                ELSE 'Bajo'
            END as severity
        FROM recent
        WHERE total_absences > 0
        ORDER BY total_absences DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(raw
        .into_iter()
        .map(|r| AttendanceAlert {
            student_id: r.student_id,
            student_name: r.student_name,
            rut: r.rut,
            month: r.month as u32,
            year: r.year,
            attendance_percentage: r.attendance_percentage,
            total_absences: r.total_absences as i32,
            severity: match r.severity.as_str() {
                "Alto" => AlertSeverity::Alto,
                "Medio" => AlertSeverity::Medio,
                _ => AlertSeverity::Bajo,
            },
        })
        .collect())
}
