use sqlx::PgPool;

pub async fn run(pool: &PgPool) {
    let statements = vec![
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            rut VARCHAR(12) UNIQUE NOT NULL,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role VARCHAR(20) NOT NULL DEFAULT 'Profesor',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS students (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            rut VARCHAR(12) UNIQUE NOT NULL,
            first_name VARCHAR(255) NOT NULL,
            last_name VARCHAR(255) NOT NULL,
            email VARCHAR(255),
            phone VARCHAR(20),
            grade_level VARCHAR(20) NOT NULL,
            section VARCHAR(10) NOT NULL,
            cod_nivel VARCHAR(10),
            condicion VARCHAR(2) NOT NULL DEFAULT 'AL',
            prioritario VARCHAR(1) NOT NULL DEFAULT '0',
            nee VARCHAR(1) NOT NULL DEFAULT 'N',
            enrolled BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS courses (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(255) NOT NULL,
            subject VARCHAR(255) NOT NULL,
            grade_level VARCHAR(20) NOT NULL,
            section VARCHAR(10) NOT NULL,
            teacher_id UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS enrollments (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            course_id UUID NOT NULL REFERENCES courses(id),
            year INTEGER NOT NULL,
            active BOOLEAN NOT NULL DEFAULT true,
            UNIQUE(student_id, course_id, year)
        )",
        "CREATE TABLE IF NOT EXISTS attendance (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            course_id UUID NOT NULL REFERENCES courses(id),
            date DATE NOT NULL,
            time TIME,
            status VARCHAR(20) NOT NULL DEFAULT 'Presente',
            subject VARCHAR(255) NOT NULL,
            teacher_id UUID NOT NULL REFERENCES users(id),
            observation TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(student_id, course_id, date, subject)
        )",
        "CREATE TABLE IF NOT EXISTS grades (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            student_id UUID NOT NULL REFERENCES students(id),
            subject VARCHAR(255) NOT NULL,
            grade DOUBLE PRECISION NOT NULL,
            grade_type VARCHAR(20) NOT NULL DEFAULT 'Sumativa',
            semester INTEGER NOT NULL DEFAULT 1,
            year INTEGER NOT NULL,
            date DATE NOT NULL,
            teacher_id UUID NOT NULL REFERENCES users(id),
            observation TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE TABLE IF NOT EXISTS agenda_events (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            description TEXT,
            event_date DATE NOT NULL,
            event_type VARCHAR(20) NOT NULL DEFAULT 'Evento',
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        "CREATE INDEX IF NOT EXISTS idx_attendance_date ON attendance(date)",
        "CREATE INDEX IF NOT EXISTS idx_attendance_student ON attendance(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_student ON grades(student_id)",
        "CREATE INDEX IF NOT EXISTS idx_grades_subject ON grades(subject)",
    ];

    for stmt in statements {
        sqlx::query(stmt).execute(pool).await.unwrap_or_else(|e| {
            tracing::warn!("Schema statement skipped: {e}");
            Default::default()
        });
    }

    tracing::info!("Database schema initialized");
}
