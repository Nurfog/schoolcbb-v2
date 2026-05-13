use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use serde_json::Value;
use std::fmt::Write;
use tracing;

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                let _ = write!(result, "%{:02X}", byte);
            }
        }
    }
    result
}

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(sis_url: &str, academic_url: &str, client: reqwest::Client) -> AppSchema {
    Schema::build(QueryRoot {}, EmptyMutation, EmptySubscription)
        .data(ServicesConfig {
            sis_url: sis_url.to_string(),
            academic_url: academic_url.to_string(),
        })
        .data(client)
        .finish()
}

#[derive(Debug)]
pub struct ServicesConfig {
    pub sis_url: String,
    pub academic_url: String,
}

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn students(&self, ctx: &Context<'_>, search: Option<String>) -> Vec<StudentGql> {
        let client = match ctx.data::<reqwest::Client>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("students resolver: missing reqwest::Client in context: {e:?}");
                return vec![];
            }
        };
        let cfg = match ctx.data::<ServicesConfig>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("students resolver: missing ServicesConfig in context: {e:?}");
                return vec![];
            }
        };
        let endpoint = match search {
            Some(q) => format!(
                "/api/students?search={}",
                urlencoding(&q)
            ),
            None => "/api/students".to_string(),
        };

        let token = get_token(ctx);
        let mut req = client.get(format!("{}{}", cfg.sis_url, endpoint));
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(data) = resp.json::<Value>().await {
                    if let Some(students) = data["students"].as_array() {
                        return students
                            .iter()
                            .map(|s| StudentGql {
                                id: s["id"].as_str().unwrap_or("").to_string(),
                                rut: s["rut"].as_str().unwrap_or("").to_string(),
                                first_name: s["first_name"].as_str().unwrap_or("").to_string(),
                                last_name: s["last_name"].as_str().unwrap_or("").to_string(),
                                grade_level: s["grade_level"].as_str().unwrap_or("").to_string(),
                                section: s["section"].as_str().unwrap_or("").to_string(),
                            })
                            .collect();
                    }
                    tracing::warn!("students resolver: response missing 'students' array");
                } else {
                    tracing::warn!("students resolver: failed to parse upstream JSON response");
                }
                vec![]
            }
            Err(e) => {
                tracing::warn!("students resolver: upstream request failed: {e}");
                vec![]
            }
        }
    }

    async fn subjects(&self, ctx: &Context<'_>) -> Vec<SubjectGql> {
        let client = match ctx.data::<reqwest::Client>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("subjects resolver: missing reqwest::Client in context: {e:?}");
                return vec![];
            }
        };
        let cfg = match ctx.data::<ServicesConfig>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("subjects resolver: missing ServicesConfig in context: {e:?}");
                return vec![];
            }
        };
        let token = get_token(ctx);
        let mut req = client.get(format!("{}/api/grades/subjects", cfg.academic_url));
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(data) = resp.json::<Value>().await {
                    if let Some(subjects) = data["subjects"].as_array() {
                        return subjects
                            .iter()
                            .map(|s| SubjectGql {
                                id: s["id"].as_str().unwrap_or("").to_string(),
                                code: s["code"].as_str().unwrap_or("").to_string(),
                                name: s["name"].as_str().unwrap_or("").to_string(),
                                level: s["level"].as_str().unwrap_or("").to_string(),
                            })
                            .collect();
                    }
                    tracing::warn!("subjects resolver: response missing 'subjects' array");
                } else {
                    tracing::warn!("subjects resolver: failed to parse upstream JSON response");
                }
                vec![]
            }
            Err(e) => {
                tracing::warn!("subjects resolver: upstream request failed: {e}");
                vec![]
            }
        }
    }

    async fn student_report(
        &self,
        ctx: &Context<'_>,
        student_id: String,
        year: i32,
    ) -> Option<StudentReportGql> {
        let client = match ctx.data::<reqwest::Client>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("student_report resolver: missing reqwest::Client in context: {e:?}");
                return None;
            }
        };
        let cfg = match ctx.data::<ServicesConfig>() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("student_report resolver: missing ServicesConfig in context: {e:?}");
                return None;
            }
        };
        let token = get_token(ctx);
        let mut req = client.get(format!(
            "{}/api/grades/reports/student/{}/{}",
            cfg.academic_url, student_id, year
        ));
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(data) = resp.json::<Value>().await {
                    Some(StudentReportGql {
                        student_name: data["student_name"].as_str().unwrap_or("").to_string(),
                        year,
                        final_promotion: data["final_promotion"].as_str().unwrap_or("").to_string(),
                    })
                } else {
                    tracing::warn!("student_report resolver: failed to parse upstream JSON response");
                    None
                }
            }
            Err(e) => {
                tracing::warn!("student_report resolver: upstream request failed: {e}");
                None
            }
        }
    }
}

fn get_token(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<String>().cloned()
}

#[derive(SimpleObject)]
pub struct StudentGql {
    pub id: String,
    pub rut: String,
    pub first_name: String,
    pub last_name: String,
    pub grade_level: String,
    pub section: String,
}

#[derive(SimpleObject)]
pub struct SubjectGql {
    pub id: String,
    pub code: String,
    pub name: String,
    pub level: String,
}

#[derive(SimpleObject)]
pub struct StudentReportGql {
    pub student_name: String,
    pub year: i32,
    pub final_promotion: String,
}
