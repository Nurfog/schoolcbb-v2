use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use serde_json::Value;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot {}, EmptyMutation, EmptySubscription)
        .finish()
}

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn students(&self, ctx: &Context<'_>, search: Option<String>) -> Vec<StudentGql> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let endpoint = match search {
            Some(q) => format!("/api/students?search={}", q.replace(' ', "%20")),
            None => "/api/students".to_string(),
        };

        let token = get_token(ctx);
        let mut req = client.get(format!("http://localhost:3002{}", endpoint));
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
                }
                vec![]
            }
            Err(_) => vec![],
        }
    }

    async fn subjects(&self, ctx: &Context<'_>) -> Vec<SubjectGql> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let token = get_token(ctx);
        let mut req = client.get("http://localhost:3003/api/grades/subjects");
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
                }
                vec![]
            }
            Err(_) => vec![],
        }
    }

    async fn student_report(
        &self,
        ctx: &Context<'_>,
        student_id: String,
        year: i32,
    ) -> Option<StudentReportGql> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let token = get_token(ctx);
        let mut req = client.get(format!(
            "http://localhost:3003/api/grades/reports/student/{}/{}",
            student_id, year
        ));
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        match req.send().await {
            Ok(resp) => {
                if let Ok(data) = resp.json::<Value>().await {
                    Some(StudentReportGql {
                        student_name: data["student_name"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        year,
                        final_promotion: data["final_promotion"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
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
