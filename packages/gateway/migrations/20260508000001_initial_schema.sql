-- Initial schema for SchoolCBB v2

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rut VARCHAR(12) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'Profesor',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS students (
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
    diseases TEXT,
    allergies TEXT,
    emergency_contact_name VARCHAR(255),
    emergency_contact_phone VARCHAR(20),
    emergency_contact_relation VARCHAR(100),
    enrolled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS courses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    grade_level VARCHAR(20) NOT NULL,
    section VARCHAR(10) NOT NULL,
    teacher_id UUID REFERENCES users(id),
    plan VARCHAR(2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES students(id),
    course_id UUID NOT NULL REFERENCES courses(id),
    year INTEGER NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(student_id, course_id, year)
);

CREATE TABLE IF NOT EXISTS attendance (
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
);

CREATE TABLE IF NOT EXISTS grades (
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
);

CREATE TABLE IF NOT EXISTS agenda_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    event_date DATE NOT NULL,
    event_type VARCHAR(20) NOT NULL DEFAULT 'Evento',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS guardian_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES students(id),
    guardian_user_id UUID NOT NULL REFERENCES users(id),
    relationship VARCHAR(50) NOT NULL,
    authorized_pickup BOOLEAN NOT NULL DEFAULT false,
    receives_notifications BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(student_id, guardian_user_id)
);

CREATE TABLE IF NOT EXISTS subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    level VARCHAR(20),
    hours_per_week INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS academic_years (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    year INTEGER NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS academic_periods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    year INTEGER NOT NULL,
    semester INTEGER NOT NULL DEFAULT 1,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS course_subjects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id UUID NOT NULL REFERENCES courses(id),
    subject_id UUID NOT NULL REFERENCES subjects(id),
    teacher_id UUID NOT NULL REFERENCES users(id),
    academic_year INTEGER NOT NULL,
    hours_per_week INTEGER NOT NULL DEFAULT 0,
    UNIQUE(course_id, subject_id, academic_year),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS grade_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_subject_id UUID NOT NULL REFERENCES course_subjects(id),
    name VARCHAR(255) NOT NULL,
    weight_percentage DOUBLE PRECISION NOT NULL DEFAULT 100.0,
    evaluation_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID NOT NULL REFERENCES users(id),
    receiver_id UUID NOT NULL REFERENCES users(id),
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS interview_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES students(id),
    teacher_id UUID NOT NULL REFERENCES users(id),
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    reason TEXT NOT NULL,
    notes TEXT NOT NULL,
    follow_up TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES students(id),
    description VARCHAR(255) NOT NULL,
    amount DOUBLE PRECISION NOT NULL,
    due_date DATE NOT NULL,
    paid BOOLEAN NOT NULL DEFAULT false,
    paid_date DATE,
    paid_amount DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    fee_id UUID NOT NULL REFERENCES fees(id),
    student_id UUID NOT NULL REFERENCES students(id),
    amount DOUBLE PRECISION NOT NULL,
    payment_date DATE NOT NULL DEFAULT CURRENT_DATE,
    payment_method VARCHAR(50) NOT NULL DEFAULT 'Efectivo',
    reference VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS scholarships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id UUID NOT NULL REFERENCES students(id),
    name VARCHAR(255) NOT NULL,
    discount_percentage DOUBLE PRECISION NOT NULL DEFAULT 0,
    approved BOOLEAN NOT NULL DEFAULT false,
    approved_by UUID REFERENCES users(id),
    valid_from DATE NOT NULL,
    valid_until DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_favorites (
    user_id UUID NOT NULL REFERENCES users(id),
    module_id VARCHAR(50) NOT NULL,
    PRIMARY KEY (user_id, module_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS school_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    school_name VARCHAR(255) NOT NULL DEFAULT '',
    school_logo_url VARCHAR(500) NOT NULL DEFAULT '',
    primary_color VARCHAR(7) NOT NULL DEFAULT '#1A2B3C',
    secondary_color VARCHAR(7) NOT NULL DEFAULT '#243B4F',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    show_module_manager BOOLEAN NOT NULL DEFAULT true,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS subject_hours (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id UUID NOT NULL REFERENCES subjects(id),
    level VARCHAR(20) NOT NULL,
    hours_per_week INTEGER NOT NULL DEFAULT 0,
    UNIQUE(subject_id, level)
);

CREATE TABLE IF NOT EXISTS classrooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    capacity INTEGER NOT NULL DEFAULT 30,
    location VARCHAR(255),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE courses ADD COLUMN IF NOT EXISTS classroom_id UUID REFERENCES classrooms(id);

CREATE TABLE IF NOT EXISTS pipeline_stages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_final BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS prospects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    rut VARCHAR(12),
    email VARCHAR(255),
    phone VARCHAR(20),
    current_stage_id UUID REFERENCES pipeline_stages(id),
    assigned_user_id UUID REFERENCES users(id),
    source VARCHAR(50),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS prospect_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prospect_id UUID NOT NULL REFERENCES prospects(id) ON DELETE CASCADE,
    activity_type VARCHAR(20) NOT NULL DEFAULT 'note',
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    scheduled_at TIMESTAMPTZ,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS prospect_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prospect_id UUID NOT NULL REFERENCES prospects(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    s3_url VARCHAR(500),
    doc_type VARCHAR(50) NOT NULL DEFAULT 'other',
    is_verified BOOLEAN NOT NULL DEFAULT false,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_prospects_stage ON prospects(current_stage_id);
CREATE INDEX IF NOT EXISTS idx_prospects_assigned ON prospects(assigned_user_id);
CREATE INDEX IF NOT EXISTS idx_prospect_activities_prospect ON prospect_activities(prospect_id);
CREATE INDEX IF NOT EXISTS idx_prospect_documents_prospect ON prospect_documents(prospect_id);

CREATE TABLE IF NOT EXISTS grade_levels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    plan VARCHAR(20),
    sort_order INTEGER NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE grades ADD COLUMN IF NOT EXISTS course_subject_id UUID REFERENCES course_subjects(id);
ALTER TABLE grades ADD COLUMN IF NOT EXISTS category_id UUID REFERENCES grade_categories(id);

CREATE INDEX IF NOT EXISTS idx_attendance_date ON attendance(date);
CREATE INDEX IF NOT EXISTS idx_attendance_student ON attendance(student_id);
CREATE INDEX IF NOT EXISTS idx_grades_student ON grades(student_id);
CREATE INDEX IF NOT EXISTS idx_grades_subject ON grades(subject);
CREATE INDEX IF NOT EXISTS idx_grades_course_subject ON grades(course_subject_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_guardian_relationships_student ON guardian_relationships(student_id);
CREATE INDEX IF NOT EXISTS idx_guardian_relationships_guardian ON guardian_relationships(guardian_user_id);
CREATE INDEX IF NOT EXISTS idx_subjects_active ON subjects(active);
CREATE INDEX IF NOT EXISTS idx_academic_periods_year ON academic_periods(year);
CREATE INDEX IF NOT EXISTS idx_course_subjects_course ON course_subjects(course_id);
CREATE INDEX IF NOT EXISTS idx_course_subjects_teacher ON course_subjects(teacher_id);
CREATE INDEX IF NOT EXISTS idx_grade_categories_subject ON grade_categories(course_subject_id);
CREATE INDEX IF NOT EXISTS idx_messages_receiver ON messages(receiver_id);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_interview_logs_student ON interview_logs(student_id);
CREATE INDEX IF NOT EXISTS idx_fees_student ON fees(student_id);
CREATE INDEX IF NOT EXISTS idx_payments_fee ON payments(fee_id);
CREATE INDEX IF NOT EXISTS idx_scholarships_student ON scholarships(student_id);
CREATE INDEX IF NOT EXISTS idx_subject_hours_subject ON subject_hours(subject_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON password_reset_tokens(token_hash);

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL,
    user_id UUID,
    changes JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created_at);
