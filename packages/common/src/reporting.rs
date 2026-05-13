use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Certificado de alumno regular para un estudiante.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRegular {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub grade_level: String,
    pub section: String,
    pub year: i32,
    pub enrollment_status: String,
    pub issued_at: String,
    pub issuer_name: String,
}

/// Concentración de notas de un estudiante en un año completo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeConcentration {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub year: i32,
    pub semesters: Vec<SemesterConcentration>,
    pub final_promotion: String,
    pub final_average: f64,
}

/// Concentración de notas de un semestre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemesterConcentration {
    pub semester: i32,
    pub subjects: Vec<SubjectConcentration>,
    pub global_average: f64,
}

/// Detalle de notas por asignatura dentro de una concentración.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectConcentration {
    pub subject_name: String,
    pub subject_code: String,
    pub grades: Vec<f64>,
    pub average: f64,
    pub min_grade: f64,
    pub max_grade: f64,
}

/// Acta de calificaciones finales de un curso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalRecord {
    pub course_id: Uuid,
    pub course_name: String,
    pub grade_level: String,
    pub section: String,
    pub year: i32,
    pub students: Vec<FinalRecordStudent>,
    pub summary: FinalRecordSummary,
}

/// Estudiante dentro de un acta de calificaciones finales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalRecordStudent {
    pub student_id: Uuid,
    pub student_name: String,
    pub rut: String,
    pub subjects: Vec<FinalSubjectGrade>,
    pub final_average: f64,
    pub promotion: String,
}

/// Calificación final de un estudiante en una asignatura (ambos semestres).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalSubjectGrade {
    pub subject_name: String,
    pub subject_code: String,
    pub semester1_avg: f64,
    pub semester2_avg: f64,
    pub final_avg: f64,
}

/// Resumen estadístico de un acta de calificaciones finales.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalRecordSummary {
    pub total_students: usize,
    pub promoted: usize,
    pub failed: usize,
    pub average_promotion_rate: f64,
}

/// Exportación de datos de estudiantes para plataforma SIGE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigeStudentExport {
    pub rows: Vec<SigeStudentRow>,
    pub total: usize,
    pub generated_at: String,
}

/// Fila individual de la exportación SIGE de estudiantes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigeStudentRow {
    pub rut: String,
    pub names: String,
    pub last_name: String,
    pub grade_level: String,
    pub section: String,
    pub cod_nivel: String,
    pub condicion: String,
    pub prioritario: String,
    pub nee: String,
}

/// Exportación de datos de asistencia para plataforma SIGE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigeAttendanceExport {
    pub rows: Vec<SigeAttendanceRow>,
    pub total: usize,
    pub year: i32,
    pub month: u32,
}

/// Fila individual de la exportación SIGE de asistencia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigeAttendanceRow {
    pub rut: String,
    pub student_name: String,
    pub grade_level: String,
    pub section: String,
    pub total_days: i32,
    pub present: i32,
    pub absent: i32,
    pub late: i32,
    pub justified: i32,
    pub attendance_percentage: f64,
}
