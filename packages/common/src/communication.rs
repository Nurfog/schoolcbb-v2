use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub subject: String,
    pub body: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum AudienceTarget {
    User(Uuid),
    Course(Uuid),
    AllStudents,
    AllTeachers,
    AllStaff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessagePayload {
    pub audience: AudienceTarget,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCount {
    pub total: i64,
    pub unread: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct InterviewLog {
    pub id: Uuid,
    pub student_id: Uuid,
    pub teacher_id: Uuid,
    pub date: NaiveDate,
    pub reason: String,
    pub notes: String,
    pub follow_up: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInterviewPayload {
    pub student_id: Uuid,
    pub reason: String,
    pub notes: String,
    pub follow_up: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInterviewPayload {
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub follow_up: Option<String>,
}
