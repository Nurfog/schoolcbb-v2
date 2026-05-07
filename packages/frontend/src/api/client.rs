use serde_json::Value;
use std::sync::OnceLock;

fn base_url() -> String {
    String::new()
}

fn abs_url(endpoint: &str) -> String {
    let base = base_url();
    let ep = endpoint.trim_start_matches('/');
    format!("{}/{}", base, ep)
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .build()
            .expect("Failed to create HTTP client")
    })
}

fn get_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok().flatten()?;
    storage.get_item("jwt_token").ok().flatten()
}

fn auth_header() -> Option<String> {
    get_token().map(|t| format!("Bearer {}", t))
}

async fn request(method: &str, endpoint: &str, body: Option<&Value>) -> Result<Value, String> {
    let url = abs_url(endpoint);
    let mut req = match method {
        "GET" => client().get(&url),
        "POST" => client().post(&url),
        "PUT" => client().put(&url),
        "DELETE" => client().delete(&url),
        _ => return Err(format!("Invalid method: {method}")),
    };
    if let Some(b) = body {
        req = req.json(b);
    }
    if let Some(auth) = auth_header() {
        req = req.header("Authorization", auth);
    }
    req.send()
        .await
        .map_err(|e| format!("Error: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("Parse: {e}"))
}

pub async fn fetch_json(endpoint: &str) -> Result<Value, String> {
    request("GET", endpoint, None).await
}

pub async fn post_json(endpoint: &str, body: &Value) -> Result<Value, String> {
    request("POST", endpoint, Some(body)).await
}

pub async fn login(email: &str, password: &str) -> Result<Value, String> {
    let body = serde_json::json!({ "email": email, "password": password });
    let resp = client()
        .post(&abs_url("/api/auth/login"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error: {e}"))?;

    let result: Value = resp.json().await.map_err(|e| format!("Parse: {e}"))?;

    if let Some(token) = result.get("token").and_then(|v| v.as_str()) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("jwt_token", token);
            }
        }
    }
    Ok(result)
}

// ─── Dashboard ───
#[allow(dead_code)]
pub async fn fetch_dashboard_summary() -> Result<Value, String> {
    fetch_json("/api/dashboard/summary").await
}
#[allow(dead_code)]
pub async fn fetch_attendance_today() -> Result<Value, String> {
    fetch_json("/api/dashboard/attendance-today").await
}
#[allow(dead_code)]
pub async fn fetch_student_alerts() -> Result<Value, String> {
    fetch_json("/api/dashboard/student-alerts").await
}
#[allow(dead_code)]
pub async fn fetch_agenda() -> Result<Value, String> {
    fetch_json("/api/dashboard/agenda").await
}

// ─── Students ───
pub async fn search_students(query: &str) -> Result<Value, String> {
    let q = urlencoding(query);
    fetch_json(&format!("/api/students?search={q}")).await
}
pub async fn fetch_students(grade_level: Option<&str>, section: Option<&str>, search: Option<&str>) -> Result<Value, String> {
    let mut params = vec![];
    if let Some(gl) = grade_level { params.push(format!("grade_level={}", urlencoding(gl))); }
    if let Some(sec) = section { params.push(format!("section={}", urlencoding(sec))); }
    if let Some(q) = search { params.push(format!("search={}", urlencoding(q))); }
    let qs = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    fetch_json(&format!("/api/students{}", qs)).await
}
#[allow(dead_code)]
pub async fn fetch_student_full(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/students/{}", student_id)).await
}

// ─── Grades ───
pub async fn fetch_subjects() -> Result<Value, String> {
    fetch_json("/api/grades/subjects").await
}
pub async fn fetch_grades_student(student_id: &str, semester: i32, year: i32) -> Result<Value, String> {
    fetch_json(&format!("/api/grades/student/{}/{}/{}", student_id, semester, year)).await
}
pub async fn fetch_student_report(student_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!("/api/grades/reports/student/{}/{}", student_id, year)).await
}
#[allow(dead_code)]
pub async fn fetch_course_performance(course_id: &str, year: i32) -> Result<Value, String> {
    fetch_json(&format!("/api/grades/reports/course/{}/{}", course_id, year)).await
}

// ─── Attendance ───
pub async fn fetch_attendance_monthly(year: i32, month: u32) -> Result<Value, String> {
    fetch_json(&format!("/api/attendance/monthly/{}/{}", year, month)).await
}
#[allow(dead_code)]
pub async fn fetch_attendance_by_course_date(course_id: &str, date: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/attendance/course/{}/date/{}", course_id, date)).await
}
// ─── Communications ───
pub async fn fetch_interviews_student(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/communications/interviews/student/{}", student_id)).await
}

// ─── Finance ───
pub async fn fetch_fees_student(student_id: &str) -> Result<Value, String> {
    fetch_json(&format!("/api/finance/fees/student/{}", student_id)).await
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
}
