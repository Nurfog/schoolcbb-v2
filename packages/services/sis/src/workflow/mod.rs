use std::sync::Arc;

use schoolcbb_common::event_bus::{BroadcastBus, EventBus, SystemEvent};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct ProspectRow {
    id: Uuid,
    first_name: String,
    last_name: String,
    rut: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrmEvent {
    StageChanged {
        prospect_id: Uuid,
        from_stage_id: Option<Uuid>,
        to_stage_id: Uuid,
        to_stage_name: String,
        triggered_by: Option<Uuid>,
    },
    DocumentUploaded {
        prospect_id: Uuid,
        document_id: Uuid,
        doc_type: String,
        uploaded_by: Option<Uuid>,
    },
    DocumentVerified {
        document_id: Uuid,
        prospect_id: Uuid,
    },
}

#[derive(Debug)]
struct WorkflowRule {
    stage_name: String,
    action: WorkflowAction,
}

#[derive(Debug)]
#[allow(dead_code)]
enum WorkflowAction {
    PromoteToStudent,
    NotifyFinance,
    CreateTask { description: String },
}

pub struct WorkflowEngine {
    pool: PgPool,
    rules: Vec<WorkflowRule>,
    finance_grpc_url: Option<String>,
    source_service: String,
    event_bus: Option<Arc<BroadcastBus>>,
}

impl WorkflowEngine {
    pub fn new(pool: PgPool) -> Self {
        Self::with_grpc(pool, None, "sis".into())
    }

    pub fn with_grpc(pool: PgPool, finance_grpc_url: Option<String>, source_service: String) -> Self {
        let rules = vec![
            WorkflowRule {
                stage_name: "Aceptado".to_string(),
                action: WorkflowAction::NotifyFinance,
            },
            WorkflowRule {
                stage_name: "Matriculado".to_string(),
                action: WorkflowAction::PromoteToStudent,
            },
        ];

        Self { pool, rules, finance_grpc_url, source_service, event_bus: None }
    }

    pub fn with_event_bus(mut self, bus: Arc<BroadcastBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub async fn process(&self, event: CrmEvent) {
        match event {
            CrmEvent::StageChanged {
                prospect_id,
                to_stage_name,
                triggered_by,
                ..
            } => {
                tracing::info!("Workflow: StageChanged -> {} for prospect {}", to_stage_name, prospect_id);

                self.log_event("stage_changed", &serde_json::json!({
                    "prospect_id": prospect_id,
                    "to_stage": &to_stage_name,
                })).await;

                self.publish_event("stage_changed", &prospect_id.to_string(), "", &serde_json::json!({"stage": &to_stage_name})).await;
                self.notify_finance_grpc("stage_changed", &prospect_id.to_string(), "", &to_stage_name).await;

                for rule in &self.rules {
                    if rule.stage_name == to_stage_name {
                        self.execute_action(&rule.action, prospect_id, triggered_by).await;
                    }
                }
            }
            CrmEvent::DocumentUploaded {
                prospect_id,
                doc_type,
                uploaded_by,
                ..
            } => {
                tracing::info!("Workflow: DocumentUploaded type={} for prospect {}", doc_type, prospect_id);

                self.log_event("document_uploaded", &serde_json::json!({
                    "prospect_id": prospect_id,
                    "doc_type": &doc_type,
                })).await;

                self.publish_event("document_uploaded", &prospect_id.to_string(), "", &serde_json::json!({"doc_type": &doc_type})).await;
                self.notify_finance_grpc("document_uploaded", &prospect_id.to_string(), "", &doc_type).await;

                self.create_verification_activity(prospect_id, &doc_type, uploaded_by).await;
            }
            CrmEvent::DocumentVerified { prospect_id, .. } => {
                tracing::info!("Workflow: DocumentVerified for prospect {}", prospect_id);

                self.log_event("document_verified", &serde_json::json!({
                    "prospect_id": prospect_id,
                })).await;

                self.publish_event("document_verified", &prospect_id.to_string(), "", &serde_json::json!({})).await;
                self.notify_finance_grpc("document_verified", &prospect_id.to_string(), "", "").await;
            }
        }
    }

    async fn execute_action(&self, action: &WorkflowAction, prospect_id: Uuid, triggered_by: Option<Uuid>) {
        match action {
            WorkflowAction::PromoteToStudent => {
                self.promote_to_student(prospect_id, triggered_by).await;
            }
            WorkflowAction::NotifyFinance => {
                self.notify_finance(prospect_id, triggered_by).await;
            }
            WorkflowAction::CreateTask { description } => {
                self.create_task_activity(prospect_id, description, triggered_by).await;
            }
        }
    }

    async fn promote_to_student(&self, prospect_id: Uuid, _triggered_by: Option<Uuid>) {
        let prospect: Option<ProspectRow> = sqlx::query_as(
            r#"SELECT id, first_name, last_name, rut, email, phone FROM prospects WHERE id = $1"#,
        )
        .bind(prospect_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        let prospect = match prospect {
            Some(p) => p,
            None => {
                tracing::warn!("Workflow: prospect {} not found for promotion", prospect_id);
                return;
            }
        };

        let rut = prospect.rut.as_deref().unwrap_or("");
        if rut.is_empty() {
            tracing::warn!("Workflow: prospect {} has no RUT, skipping promotion", prospect_id);
            return;
        }

        let existing: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM students WHERE rut = $1")
            .bind(rut)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

        if existing.map(|e| e.0).unwrap_or(0) > 0 {
            tracing::info!("Workflow: student with RUT {} already exists", rut);
            return;
        }

        let student_id = Uuid::new_v4();
        let insert_result = sqlx::query(
            r#"INSERT INTO students (id, rut, first_name, last_name, email, phone, grade_level, section, enrolled)
               VALUES ($1, $2, $3, $4, $5, $6, 'Pendiente', 'A', true)"#,
        )
        .bind(student_id)
        .bind(rut)
        .bind(&prospect.first_name)
        .bind(&prospect.last_name)
        .bind(&prospect.email)
        .bind(&prospect.phone)
        .execute(&self.pool)
        .await;

        match insert_result {
            Ok(_) => {
                tracing::info!("Workflow: Prospect {} promoted to student {}", prospect_id, student_id);

                self.create_activity(prospect_id, "system", format!("Postulante matriculado como alumno {}", student_id)).await;
            }
            Err(e) => {
                tracing::error!("Workflow: failed to promote prospect {}: {}", prospect_id, e);
            }
        }
    }

    async fn notify_finance(&self, prospect_id: Uuid, _triggered_by: Option<Uuid>) {
        tracing::info!("Workflow: Notifying finance for prospect {}", prospect_id);

        self.create_activity(prospect_id, "system", "Postulante aceptado - Pendiente generación de cupón de pago".to_string()).await;
    }

    async fn create_verification_activity(&self, prospect_id: Uuid, doc_type: &str, created_by: Option<Uuid>) {
        let subject = format!("Verificar documento: {}", doc_type);
        let activity_id = Uuid::new_v4();

        let _ = sqlx::query(
            r#"INSERT INTO prospect_activities (id, prospect_id, activity_type, subject, description, is_completed, created_by)
               VALUES ($1, $2, 'task', $3, $4, false, $5)"#,
        )
        .bind(activity_id)
        .bind(prospect_id)
        .bind(&subject)
        .bind("Documento pendiente de verificación por el equipo de admisión")
        .bind(created_by)
        .execute(&self.pool)
        .await;

        tracing::info!("Workflow: verification task created for prospect {} doc_type={}", prospect_id, doc_type);
    }

    async fn create_task_activity(&self, prospect_id: Uuid, description: &str, created_by: Option<Uuid>) {
        let activity_id = Uuid::new_v4();

        let _ = sqlx::query(
            r#"INSERT INTO prospect_activities (id, prospect_id, activity_type, subject, description, is_completed, created_by)
               VALUES ($1, $2, 'task', $3, $4, false, $5)"#,
        )
        .bind(activity_id)
        .bind(prospect_id)
        .bind("Tarea automática del workflow")
        .bind(description)
        .bind(created_by)
        .execute(&self.pool)
        .await;
    }

    async fn create_activity(&self, prospect_id: Uuid, activity_type: &str, subject: String) {
        let activity_id = Uuid::new_v4();
        let _ = sqlx::query(
            r#"INSERT INTO prospect_activities (id, prospect_id, activity_type, subject, is_completed)
               VALUES ($1, $2, $3, $4, true)"#,
        )
        .bind(activity_id)
        .bind(prospect_id)
        .bind(activity_type)
        .bind(&subject)
        .execute(&self.pool)
        .await;
    }

    async fn publish_event(&self, event_type: &str, prospect_id: &str, student_id: &str, payload: &serde_json::Value) {
        if let Some(ref bus) = self.event_bus {
            let event = SystemEvent {
                event_type: event_type.to_string(),
                source: self.source_service.clone(),
                prospect_id: if prospect_id.is_empty() { None } else { Some(prospect_id.to_string()) },
                student_id: if student_id.is_empty() { None } else { Some(student_id.to_string()) },
                payload: payload.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            bus.publish(event).await;
        }
    }

    async fn log_event(&self, event_type: &str, payload: &serde_json::Value) {
        let _ = sqlx::query(
            r#"INSERT INTO event_log (id, event_type, payload) VALUES ($1, $2, $3)"#,
        )
        .bind(Uuid::new_v4())
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await;
    }

    async fn notify_finance_grpc(&self, event_type: &str, prospect_id: &str, student_id: &str, stage_name: &str) {
        let grpc_url = match &self.finance_grpc_url {
            Some(url) => url.clone(),
            None => return,
        };

        use schoolcbb_proto::workflow_events_client::WorkflowEventsClient;
        use schoolcbb_proto::EventNotification;

        let notification = EventNotification {
            event_type: event_type.to_string(),
            source_service: self.source_service.clone(),
            prospect_id: prospect_id.to_string(),
            student_id: student_id.to_string(),
            stage_name: stage_name.to_string(),
        };

        match WorkflowEventsClient::connect(grpc_url).await {
            Ok(mut client) => {
                match client.notify_event(notification).await {
                    Ok(resp) => {
                        let ack = resp.into_inner();
                        if ack.ok {
                            tracing::info!("gRPC notify success: {}", ack.message);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("gRPC notify error: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("gRPC connect error: {}", e);
            }
        }
    }
}

#[allow(dead_code)]
pub type SharedWorkflowEngine = Arc<WorkflowEngine>;
