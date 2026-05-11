use schoolcbb_proto::workflow_events_server::{WorkflowEvents, WorkflowEventsServer};
use schoolcbb_proto::{EventAck, EventNotification};
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct WorkflowService {
    pub pool: PgPool,
}

#[tonic::async_trait]
impl WorkflowEvents for WorkflowService {
    async fn notify_event(&self, req: Request<EventNotification>) -> Result<Response<EventAck>, Status> {
        let event = req.into_inner();
        tracing::info!(
            "gRPC event received: type={} source={} prospect={} student={} stage={}",
            event.event_type, event.source_service, event.prospect_id, event.student_id, event.stage_name,
        );

        match event.event_type.as_str() {
            "prospect_accepted" => {
                if !event.prospect_id.is_empty() {
                    let _prospect_id = Uuid::parse_str(&event.prospect_id).unwrap_or_default();
                    let _ = sqlx::query(
                        r#"INSERT INTO event_log (id, event_type, payload)
                           VALUES ($1, $2, $3)"#,
                    )
                    .bind(Uuid::new_v4())
                    .bind("prospect_accepted_grpc")
                    .bind(serde_json::json!({
                        "prospect_id": &event.prospect_id,
                        "stage": &event.stage_name,
                        "source": &event.source_service,
                    }))
                    .execute(&self.pool)
                    .await;
                }
            }
            _ => {
                tracing::warn!("Unknown gRPC event type: {}", event.event_type);
            }
        }

        Ok(Response::new(EventAck {
            ok: true,
            message: "Event received".into(),
        }))
    }
}

pub async fn start_grpc_server(pool: PgPool, addr: String) {
    let service = WorkflowService {
        pool,
    };

    tracing::info!("gRPC server starting on {addr}");

    let _ = tonic::transport::Server::builder()
        .add_service(WorkflowEventsServer::new(service))
        .serve(addr.parse().unwrap())
        .await;
}
