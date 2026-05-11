use axum::{
    Json, Router,
    extract::{
        FromRequestParts, Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::request::Parts,
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::error::{NotifError, NotifResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub school_id: Option<String>,
    pub corporation_id: Option<String>,
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = NotifError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(NotifError::Unauthorized)?;

        let secret = &_state.config.jwt_secret;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|_| NotifError::Unauthorized)?;

        Ok(token_data.claims)
    }
}

pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), NotifError> {
    if !roles.contains(&claims.role.as_str()) {
        return Err(NotifError::Forbidden(format!(
            "Se requiere uno de los roles {:?}, tiene '{}'",
            roles, claims.role
        )));
    }
    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ws", get(ws_handler))
        .route(
            "/api/communications/messages",
            get(list_messages).post(send_message),
        )
        .route(
            "/api/communications/messages/unread-count",
            get(unread_count),
        )
        .route("/api/communications/messages/{id}", get(get_message))
        .route("/api/communications/messages/{id}/read", post(mark_read))
        .route(
            "/api/communications/interviews",
            get(list_interviews).post(create_interview),
        )
        .route(
            "/api/communications/interviews/{id}",
            get(get_interview)
                .put(update_interview)
                .delete(delete_interview),
        )
        .route(
            "/api/communications/interviews/student/{student_id}",
            get(interviews_by_student),
        )
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.ws_hub))
}

async fn handle_socket(socket: WebSocket, hub: std::sync::Arc<crate::ws::hub::WsHub>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = hub.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                hub.broadcast(&text);
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

async fn list_messages(claims: Claims, State(state): State<AppState>) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let user_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    let messages = sqlx::query_as::<_, schoolcbb_common::communication::Message>(
        r#"
        SELECT id, sender_id, receiver_id, subject, body, read, created_at
        FROM messages WHERE receiver_id = $1 OR sender_id = $1
        ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "messages": messages, "total": messages.len() }),
    ))
}

async fn send_message(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::communication::CreateMessagePayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let sender_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    if payload.subject.trim().is_empty() || payload.body.trim().is_empty() {
        return Err(NotifError::Validation(
            "Asunto y cuerpo son obligatorios".into(),
        ));
    }

    let recipients: Vec<Uuid> = resolve_recipients(&state.pool, &payload.audience).await?;

    if recipients.is_empty() {
        return Err(NotifError::Validation(
            "No hay destinatarios para la audiencia seleccionada".into(),
        ));
    }

    let mut sent: Vec<schoolcbb_common::communication::Message> = vec![];
    for recv_id in &recipients {
        let id = Uuid::new_v4();
        let msg = sqlx::query_as::<_, schoolcbb_common::communication::Message>(
            r#"
            INSERT INTO messages (id, sender_id, receiver_id, subject, body)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, sender_id, receiver_id, subject, body, read, created_at
            "#,
        )
        .bind(id)
        .bind(sender_id)
        .bind(recv_id)
        .bind(&payload.subject)
        .bind(&payload.body)
        .fetch_one(&state.pool)
        .await?;

        state.ws_hub.broadcast(
            &json!({
                "type": "new_message",
                "receiver_id": recv_id,
                "message": &msg
            })
            .to_string(),
        );

        sent.push(msg);
    }

    Ok(Json(json!({ "messages": sent, "total": sent.len() })))
}

async fn resolve_recipients(
    pool: &sqlx::PgPool,
    audience: &schoolcbb_common::communication::AudienceTarget,
) -> Result<Vec<Uuid>, NotifError> {
    match audience {
        schoolcbb_common::communication::AudienceTarget::User(uid) => Ok(vec![*uid]),
        schoolcbb_common::communication::AudienceTarget::Course(course_id) => {
            let rows: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT DISTINCT u.id FROM users u \
                 JOIN enrollments e ON e.student_id = u.id \
                 WHERE e.course_id = $1 AND e.active = true",
            )
            .bind(course_id)
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolcbb_common::communication::AudienceTarget::AllStudents => {
            let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE role = 'Alumno'")
                .fetch_all(pool)
                .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolcbb_common::communication::AudienceTarget::AllTeachers => {
            let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE role = 'Profesor'")
                .fetch_all(pool)
                .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
        schoolcbb_common::communication::AudienceTarget::AllStaff => {
            let rows: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM users WHERE role IN ('Administrador', 'Sostenedor', 'Director', 'UTP')"
            )
            .fetch_all(pool)
            .await?;
            Ok(rows.into_iter().map(|r| r.0).collect())
        }
    }
}

async fn get_message(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let msg = sqlx::query_as::<_, schoolcbb_common::communication::Message>(
        "SELECT id, sender_id, receiver_id, subject, body, read, created_at FROM messages WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Mensaje no encontrado".into()))?;

    Ok(Json(json!({ "message": msg })))
}

async fn mark_read(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let result = sqlx::query("UPDATE messages SET read = true WHERE id = $1 AND read = false")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({ "updated": result.rows_affected() > 0 })))
}

async fn unread_count(claims: Claims, State(state): State<AppState>) -> NotifResult<Json<Value>> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM messages WHERE receiver_id = $1 AND read = false")
            .bind(user_id)
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(json!({ "unread": count.0 })))
}

async fn list_interviews(
    claims: Claims,
    State(state): State<AppState>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let interviews = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        r#"
        SELECT il.id, il.student_id, il.teacher_id, il.date, il.reason, il.notes, il.follow_up, il.created_at
        FROM interview_logs il
        ORDER BY il.date DESC LIMIT 50
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(
        json!({ "interviews": interviews, "total": interviews.len() }),
    ))
}

async fn create_interview(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolcbb_common::communication::CreateInterviewPayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let teacher_id: Uuid = claims.sub.parse().map_err(|_| NotifError::Unauthorized)?;

    if payload.reason.trim().is_empty() || payload.notes.trim().is_empty() {
        return Err(NotifError::Validation(
            "Motivo y notas son obligatorios".into(),
        ));
    }

    let id = Uuid::new_v4();
    let date = payload
        .date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let result = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        r#"
        INSERT INTO interview_logs (id, student_id, teacher_id, date, reason, notes, follow_up)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, student_id, teacher_id, date, reason, notes, follow_up, created_at
        "#,
    )
    .bind(id)
    .bind(payload.student_id)
    .bind(teacher_id)
    .bind(date)
    .bind(&payload.reason)
    .bind(&payload.notes)
    .bind(&payload.follow_up)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "interview": result })))
}

async fn get_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &["Administrador", "Sostenedor", "Director", "UTP", "Profesor"],
    )?;

    let interview = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Entrevista no encontrada".into()))?;

    Ok(Json(json!({ "interview": interview })))
}

async fn update_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolcbb_common::communication::UpdateInterviewPayload>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP", "Profesor"])?;

    let existing = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(NotifError::NotFound("Entrevista no encontrada".into()))?;

    let reason = payload.reason.unwrap_or(existing.reason);
    let notes = payload.notes.unwrap_or(existing.notes);
    let follow_up = payload.follow_up.or(existing.follow_up);

    let result = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        r#"
        UPDATE interview_logs SET reason = $1, notes = $2, follow_up = $3
        WHERE id = $4
        RETURNING id, student_id, teacher_id, date, reason, notes, follow_up, created_at
        "#,
    )
    .bind(&reason)
    .bind(&notes)
    .bind(&follow_up)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({ "interview": result })))
}

async fn delete_interview(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(&claims, &["Administrador", "Director", "UTP"])?;

    let result = sqlx::query("DELETE FROM interview_logs WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(NotifError::NotFound("Entrevista no encontrada".into()));
    }

    Ok(Json(
        json!({ "message": "Entrevista eliminada correctamente" }),
    ))
}

async fn interviews_by_student(
    claims: Claims,
    State(state): State<AppState>,
    Path(student_id): Path<Uuid>,
) -> NotifResult<Json<Value>> {
    require_any_role(
        &claims,
        &[
            "Administrador",
            "Sostenedor",
            "Director",
            "UTP",
            "Profesor",
            "Apoderado",
        ],
    )?;

    let interviews = sqlx::query_as::<_, schoolcbb_common::communication::InterviewLog>(
        "SELECT id, student_id, teacher_id, date, reason, notes, follow_up, created_at FROM interview_logs WHERE student_id = $1 ORDER BY date DESC",
    )
    .bind(student_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({ "interviews": interviews })))
}
