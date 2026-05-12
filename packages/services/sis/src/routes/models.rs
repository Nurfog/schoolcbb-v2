use sqlx::PgPool;
use uuid::Uuid;

use schoolccb_common::attendance::{
    AlertSeverity, AttendanceAlert, AttendanceStatus, DailyAttendance, MonthlyAttendanceSummary,
};

#[allow(dead_code)]
pub async fn get_students(
    pool: &PgPool,
) -> Result<Vec<schoolccb_common::student::Student>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawStudent {
        id: Uuid,
        rut: String,
        first_name: String,
        last_name: String,
        email: String,
        phone: Option<String>,
        grade_level: String,
        section: String,
        cod_nivel: Option<String>,
        condicion: String,
        prioritario: String,
        nee: String,
        enrolled: bool,
    }

    let raw = sqlx::query_as::<_, RawStudent>(
        r#"
        SELECT id, rut, first_name, last_name, email, phone,
               grade_level, section, cod_nivel, condicion, prioritario, nee, enrolled
        FROM students WHERE enrolled = true
        ORDER BY last_name, first_name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(raw
        .into_iter()
        .map(|r| schoolccb_common::student::Student {
            id: r.id,
            rut: schoolccb_common::rut::Rut(r.rut),
            first_name: r.first_name,
            last_name: r.last_name,
            email: r.email,
            phone: r.phone,
            grade_level: r.grade_level,
            section: r.section,
            cod_nivel: r.cod_nivel,
            condicion: match r.condicion.as_str() {
                "RE" => schoolccb_common::student::CondicionMatricula::Repitente,
                "TR" => schoolccb_common::student::CondicionMatricula::Trasladado,
                _ => schoolccb_common::student::CondicionMatricula::AlumnoRegular,
            },
            prioritario: match r.prioritario.as_str() {
                "1" => schoolccb_common::student::Prioritario::Si,
                "2" => schoolccb_common::student::Prioritario::Preferente,
                _ => schoolccb_common::student::Prioritario::No,
            },
            nee: match r.nee.as_str() {
                "T" => schoolccb_common::student::NEE::Transitoria,
                "P" => schoolccb_common::student::NEE::Permanente,
                _ => schoolccb_common::student::NEE::No,
            },
            enrolled: r.enrolled,
        })
        .collect())
}

pub async fn get_attendance_today(
    pool: &PgPool,
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

#[allow(dead_code)]
pub async fn get_monthly_summary(
    pool: &PgPool,
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

pub async fn get_attendance_alerts(pool: &PgPool) -> Result<Vec<AttendanceAlert>, sqlx::Error> {
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

pub async fn get_agenda_events(
    pool: &PgPool,
    date: &str,
) -> Result<Vec<schoolccb_common::user::AgendaEvent>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct RawEvent {
        id: Uuid,
        title: String,
        description: Option<String>,
        date: String,
        event_type: String,
    }

    let raw = sqlx::query_as::<_, RawEvent>(
        r#"
        SELECT id, title, description,
               event_date::text as "date",
               event_type::text as event_type
        FROM agenda_events
        WHERE event_date >= $1::date
        ORDER BY event_date
        LIMIT 5
        "#,
    )
    .bind(date)
    .fetch_all(pool)
    .await?;

    Ok(raw
        .into_iter()
        .map(|r| schoolccb_common::user::AgendaEvent {
            id: r.id,
            title: r.title,
            description: r.description,
            date: r.date,
            event_type: match r.event_type.as_str() {
                "Clase" => schoolccb_common::user::EventType::Clase,
                "Reunion" => schoolccb_common::user::EventType::Reunion,
                "Evaluacion" => schoolccb_common::user::EventType::Evaluacion,
                _ => schoolccb_common::user::EventType::Evento,
            },
        })
        .collect())
}

pub async fn get_dashboard_summary(
    pool: &PgPool,
) -> Result<schoolccb_common::user::DashboardSummary, sqlx::Error> {
    let total_students: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM students WHERE enrolled = true")
            .fetch_one(pool)
            .await?;

    let total_teachers: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'Profesor' AND active = true")
            .fetch_one(pool)
            .await?;

    let today = chrono::Utc::now().date_naive().to_string();
    let attendance_today: f64 = get_attendance_percentage_today(pool, &today).await?;

    let alerts_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM (
            SELECT s.id
            FROM students s
            JOIN attendance a ON a.student_id = s.id
            WHERE a.date >= CURRENT_DATE - INTERVAL '30 days'
              AND a.status = 'Ausente'
              AND s.enrolled = true
            GROUP BY s.id
            HAVING COUNT(*) >= 3
        ) sub
        "#,
    )
    .fetch_one(pool)
    .await?;

    let today_events = get_agenda_events(pool, &today).await?;

    Ok(schoolccb_common::user::DashboardSummary {
        total_students: total_students.0,
        total_teachers: total_teachers.0,
        attendance_today_percentage: attendance_today,
        pending_alerts: alerts_count.0,
        today_events,
    })
}

async fn get_attendance_percentage_today(pool: &PgPool, date: &str) -> Result<f64, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Row {
        total: i64,
        present: i64,
    }

    let result = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'Presente' OR status = 'Justificado') as present
        FROM attendance
        WHERE date = $1::date
        "#,
    )
    .bind(date)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(r) if r.total > 0 => Ok((r.present as f64 / r.total as f64) * 100.0),
        _ => Ok(100.0),
    }
}
