use serde_json::Value;

const API_BASE: &str = "http://localhost:3000";

pub async fn fetch_json(endpoint: &str) -> Result<Value, String> {
    let url = format!("{}{}", API_BASE, endpoint);
    reqwest::get(&url)
        .await
        .map_err(|e| format!("Request failed: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("Parse failed: {e}"))
}

#[allow(dead_code)]
pub async fn fetch_dashboard_summary() -> Result<Value, String> {
    fetch_json("/api/dashboard/summary").await
}

pub async fn fetch_attendance_today() -> Result<Value, String> {
    fetch_json("/api/dashboard/attendance-today").await
}

pub async fn fetch_student_alerts() -> Result<Value, String> {
    fetch_json("/api/dashboard/student-alerts").await
}

pub async fn fetch_agenda() -> Result<Value, String> {
    fetch_json("/api/dashboard/agenda").await
}

#[allow(dead_code)]
pub async fn fetch_monthly_attendance(year: i32, month: u32) -> Result<Value, String> {
    fetch_json(&format!("/api/attendance/monthly/{year}/{month}")).await
}
