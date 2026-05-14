use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::request::Parts,
    routing::{get, post, put},
};
use jsonwebtoken::{DecodingKey, Validation};
use schoolccb_common::rut::Rut;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use schoolccb_common::school::OnboardingPayload;
use crate::error::{CrmError, CrmResult};
use crate::models::{self, Claims};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Pipeline stages
        .route("/api/sales/stages", get(list_stages).post(create_stage))
        .route("/api/sales/stages/{id}", put(update_stage).delete(delete_stage))
        // Prospects
        .route("/api/sales/prospects", get(list_prospects).post(create_prospect))
        .route("/api/sales/prospects/{id}", get(get_prospect).put(update_prospect).delete(delete_prospect))
        .route("/api/sales/prospects/{id}/assign", put(assign_prospect))
        .route("/api/sales/prospects/{id}/move", put(move_prospect_stage))
        // Activities
        .route("/api/sales/prospects/{id}/activities", get(list_activities).post(create_activity))
        .route("/api/sales/activities/{id}", put(update_activity))
        // Proposals
        .route("/api/sales/proposals", get(list_proposals).post(create_proposal))
        .route("/api/sales/proposals/{id}", get(get_proposal))
        .route("/api/sales/proposals/{id}/discount", put(apply_discount))
        // Contracts
        .route("/api/sales/contracts", post(create_contract))
        .route("/api/sales/contracts/{id}", get(get_contract))
        .route("/api/sales/contracts/{id}/verify-signatures", put(verify_signatures))
        .route("/api/sales/contracts/{id}/activate", post(activate_license))
        // Documents
        .route("/api/sales/contracts/{id}/documents", get(list_documents).post(upload_document))
        // Plans (from licensing)
        .route("/api/sales/plans", get(list_plans))
        // Dashboard
        .route("/api/sales/dashboard/summary", get(dashboard_summary))
        // Public
        .route("/api/public/sales/prospects", post(public_create_prospect))
}

// ─── Auth ───

fn require_sales_role(claims: &Claims) -> Result<(), CrmError> {
    if claims.role == "Root" || claims.role == "GerenteGeneral" || claims.role == "JefeVentas" || claims.role == "AgenteVentas" {
        return Ok(());
    }
    Err(CrmError::Forbidden("Se requiere rol de ventas".into()))
}

fn require_sales_manager(claims: &Claims) -> Result<(), CrmError> {
    if claims.role == "Root" || claims.role == "GerenteGeneral" || claims.role == "JefeVentas" {
        return Ok(());
    }
    Err(CrmError::Forbidden("Se requiere rol GerenteGeneral o JefeVentas".into()))
}

impl FromRequestParts<AppState> for Claims {
    type Rejection = CrmError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(CrmError::Unauthorized)?;

        let token_data = jsonwebtoken::decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(_state.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => CrmError::TokenExpired,
            _ => CrmError::TokenInvalid("Token inválido".into()),
        })?;

        Ok(token_data.claims)
    }
}

// ─── Stages ───

async fn list_stages(claims: Claims, State(state): State<AppState>) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let stages = sqlx::query_as::<_, models::SalesStage>(
        "SELECT id, name, sort_order, is_final, color, created_at FROM crm_sales_stages ORDER BY sort_order",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"stages": stages})))
}

async fn create_stage(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    let id = Uuid::new_v4();
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let sort_order = payload.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let is_final = payload.get("is_final").and_then(|v| v.as_bool()).unwrap_or(false);
    let color = payload.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());

    sqlx::query(
        "INSERT INTO crm_sales_stages (id, name, sort_order, is_final, color) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(name)
    .bind(sort_order)
    .bind(is_final)
    .bind(&color)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id, "name": name})))
}

async fn update_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    sqlx::query(
        "UPDATE crm_sales_stages SET name = COALESCE($1, name), sort_order = COALESCE($2, sort_order),
         is_final = COALESCE($3, is_final), color = COALESCE($4, color) WHERE id = $5",
    )
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(payload.get("sort_order").and_then(|v| v.as_i64()).map(|v| v as i32))
    .bind(payload.get("is_final").and_then(|v| v.as_bool()))
    .bind(payload.get("color").and_then(|v| v.as_str()))
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Etapa actualizada"})))
}

async fn delete_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    sqlx::query("UPDATE crm_sales_prospects SET current_stage_id = NULL WHERE current_stage_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM crm_sales_stages WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Etapa eliminada"})))
}

// ─── Prospects ───

#[derive(Deserialize)]
struct ProspectQuery {
    stage_id: Option<Uuid>,
    assigned_to: Option<Uuid>,
    search: Option<String>,
}

async fn list_prospects(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ProspectQuery>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let is_agent = claims.role == "AgenteVentas";

    let prospects = if is_agent {
        sqlx::query_as::<_, models::SalesProspect>(
            "SELECT p.id, p.first_name, p.last_name, p.email, p.phone, p.company, p.position,
                    p.source, p.requirements, p.current_stage_id, p.assigned_to,
                    p.estimated_value, p.notes, p.created_at, p.updated_at
             FROM crm_sales_prospects p
             WHERE p.assigned_to = $1
               AND ($2::uuid IS NULL OR p.current_stage_id = $2)
               AND ($3::text IS NULL OR p.first_name ILIKE $3 OR p.last_name ILIKE $3 OR p.company ILIKE $3)
             ORDER BY p.updated_at DESC",
        )
        .bind(user_id)
        .bind(q.stage_id)
        .bind(q.search.as_ref().map(|s| format!("%{}%", s)))
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, models::SalesProspect>(
            "SELECT p.id, p.first_name, p.last_name, p.email, p.phone, p.company, p.position,
                    p.source, p.requirements, p.current_stage_id, p.assigned_to,
                    p.estimated_value, p.notes, p.created_at, p.updated_at
             FROM crm_sales_prospects p
             WHERE ($1::uuid IS NULL OR p.current_stage_id = $1)
               AND ($2::text IS NULL OR p.first_name ILIKE $2 OR p.last_name ILIKE $2 OR p.company ILIKE $2)
             ORDER BY p.updated_at DESC",
        )
        .bind(q.stage_id)
        .bind(q.search.as_ref().map(|s| format!("%{}%", s)))
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(json!({"prospects": prospects})))
}

async fn create_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<models::CreateProspectPayload>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();

    let default_stage: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM crm_sales_stages ORDER BY sort_order LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?;
    let stage_id = payload.current_stage_id.or(default_stage.map(|r| r.0));

    // Validate RUT if provided
    if let Some(ref r) = payload.rut {
        if Rut::new(r).is_err() {
            return Err(CrmError::Validation("RUT inválido".into()));
        }
    }

    sqlx::query(
        "INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position,
         source, requirements, current_stage_id, assigned_to, estimated_value, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(id)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.rut)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.company)
    .bind(&payload.position)
    .bind(&payload.source)
    .bind(&payload.requirements)
    .bind(stage_id)
    .bind(user_id)
    .bind(payload.estimated_value)
    .bind(&payload.notes)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": id})))
}

async fn public_create_prospect(
    State(state): State<AppState>,
    Json(payload): Json<models::CreateProspectPayload>,
) -> CrmResult<Json<Value>> {
    let id = Uuid::new_v4();
    
    let default_stage: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM crm_sales_stages ORDER BY sort_order LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?;
    let stage_id = payload.current_stage_id.or(default_stage.map(|r| r.0));

    // Validate RUT if provided
    if let Some(ref r) = payload.rut {
        if Rut::new(r).is_err() {
            return Err(CrmError::Validation("RUT inválido".into()));
        }
    }

    sqlx::query(
        "INSERT INTO crm_sales_prospects (id, first_name, last_name, rut, email, phone, company, position,
         source, requirements, current_stage_id, assigned_to, estimated_value, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NULL, $11, $12)",
    )
    .bind(id)
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.rut)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.company)
    .bind(&payload.position)
    .bind("web") // Source is always web for public form
    .bind(&payload.requirements)
    .bind(stage_id)
    .bind(payload.estimated_value)
    .bind(&payload.notes)
    .execute(&state.pool)
    .await?;

    log_activity(&state.pool, id, "web_contact", "Contacto desde sitio web", None).await;

    Ok(Json(json!({"id": id, "message": "Prospecto creado correctamente"})))
}

async fn get_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let prospect = sqlx::query_as::<_, models::SalesProspect>(
        "SELECT id, first_name, last_name, email, phone, company, position, source,
         requirements, current_stage_id, assigned_to, estimated_value, notes, created_at, updated_at
         FROM crm_sales_prospects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Prospecto no encontrado".into()))?;

    let stage = sqlx::query_as::<_, models::SalesStage>(
        "SELECT id, name, sort_order, is_final, color, created_at FROM crm_sales_stages WHERE id = $1",
    )
    .bind(prospect.current_stage_id)
    .fetch_optional(&state.pool)
    .await?;

    let assigned = match prospect.assigned_to {
        Some(uid) => sqlx::query_as::<_, (String, String)>(
            "SELECT name, email FROM users WHERE id = $1",
        )
        .bind(uid)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    let contracts = sqlx::query_as::<_, models::SalesContract>(
        "SELECT id, prospect_id, plan_id, modules, total_value, discount, status,
         signed_at, verified_at, activated_at, notes, created_at, updated_at
         FROM crm_sales_contracts WHERE prospect_id = $1 ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({
        "prospect": prospect,
        "stage": stage,
        "assigned_user": assigned.map(|(name, email)| json!({"name": name, "email": email})),
        "contracts": contracts,
    })))
}

async fn update_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::UpdateProspectPayload>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    // Validate RUT if provided
    if let Some(ref r) = payload.rut {
        if Rut::new(r).is_err() {
            return Err(CrmError::Validation("RUT inválido".into()));
        }
    }

    sqlx::query(
        "UPDATE crm_sales_prospects SET
         first_name = COALESCE($1, first_name), last_name = COALESCE($2, last_name),
         rut = COALESCE($3, rut),
         email = COALESCE($4, email), phone = COALESCE($5, phone),
         company = COALESCE($6, company), position = COALESCE($7, position),
         source = COALESCE($8, source),
         requirements = COALESCE($9, requirements),
         current_stage_id = COALESCE($10, current_stage_id),
         assigned_to = COALESCE($11, assigned_to),
         estimated_value = COALESCE($12, estimated_value),
         notes = COALESCE($13, notes),
         updated_at = NOW()
         WHERE id = $14",
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.rut)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.company)
    .bind(&payload.position)
    .bind(&payload.source)
    .bind(&payload.requirements)
    .bind(payload.current_stage_id)
    .bind(payload.assigned_to)
    .bind(payload.estimated_value)
    .bind(&payload.notes)
    .bind(id)
    .execute(&state.pool)
    .await?;

    log_activity(&state.pool, id, "update", "Prospecto actualizado", None).await;

    Ok(Json(json!({"message": "Prospecto actualizado"})))
}

async fn delete_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    sqlx::query("DELETE FROM crm_sales_activities WHERE prospect_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM crm_sales_proposals WHERE prospect_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM crm_sales_contracts WHERE prospect_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    sqlx::query("DELETE FROM crm_sales_prospects WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Prospecto eliminado"})))
}

async fn assign_prospect(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    let agent_id = payload.get("agent_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(CrmError::Internal("agent_id es requerido".into()))?;

    sqlx::query("UPDATE crm_sales_prospects SET assigned_to = $1, updated_at = NOW() WHERE id = $2")
        .bind(agent_id)
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_activity(&state.pool, id, "assign", "Prospecto asignado", None).await;

    Ok(Json(json!({"message": "Prospecto asignado"})))
}

async fn move_prospect_stage(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let stage_id = payload.get("stage_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(CrmError::Internal("stage_id es requerido".into()))?;

    let stage_name: (String,) = sqlx::query_as(
        "SELECT name FROM crm_sales_stages WHERE id = $1",
    )
    .bind(stage_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Etapa no encontrada".into()))?;

    sqlx::query("UPDATE crm_sales_prospects SET current_stage_id = $1, updated_at = NOW() WHERE id = $2")
        .bind(stage_id)
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_activity(&state.pool, id, "stage_change", &format!("Movido a: {}", stage_name.0), None).await;

    Ok(Json(json!({"message": "Prospecto movido de etapa"})))
}

// ─── Activities ───

async fn list_activities(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let activities = sqlx::query_as::<_, models::SalesActivity>(
        "SELECT a.id, a.prospect_id, a.activity_type, a.subject, a.description,
         a.scheduled_at, a.is_completed, a.created_by, a.created_at
         FROM crm_sales_activities a
         JOIN crm_sales_prospects p ON p.id = a.prospect_id
         WHERE a.prospect_id = $1
         ORDER BY a.created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"activities": activities})))
}

async fn create_activity(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::CreateActivityPayload>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| CrmError::TokenInvalid("ID de usuario inválido".into()))?;

    let activity_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, description, scheduled_at, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(activity_id)
    .bind(id)
    .bind(&payload.activity_type)
    .bind(&payload.subject)
    .bind(&payload.description)
    .bind(payload.scheduled_at)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": activity_id})))
}

async fn update_activity(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    sqlx::query(
        "UPDATE crm_sales_activities SET is_completed = COALESCE($1, is_completed),
         description = COALESCE($2, description), scheduled_at = COALESCE($3, scheduled_at)
         WHERE id = $4",
    )
    .bind(payload.get("is_completed").and_then(|v| v.as_bool()))
    .bind(payload.get("description").and_then(|v| v.as_str()))
    .bind(payload.get("scheduled_at").and_then(|v| v.as_str()).and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc))))
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Actividad actualizada"})))
}

// ─── Proposals ───

async fn list_proposals(claims: Claims, State(state): State<AppState>, Query(q): Query<ProspectQuery>) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let proposals = sqlx::query_as::<_, models::SalesProposal>(
        "SELECT id, prospect_id, plan_id, modules, total_value, discount, version, status, notes, created_by, created_at
         FROM crm_sales_proposals ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"proposals": proposals})))
}

async fn create_proposal(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let id = Uuid::new_v4();
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let prospect_id = payload.get("prospect_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(CrmError::Internal("prospect_id es requerido".into()))?;
    let plan_id = payload.get("plan_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let total_value = payload.get("total_value").and_then(|v| v.as_f64()).unwrap_or(0.0);

    sqlx::query(
        "INSERT INTO crm_sales_proposals (id, prospect_id, plan_id, modules, total_value, discount, version, status, notes, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, 1, 'draft', $7, $8)",
    )
    .bind(id)
    .bind(prospect_id)
    .bind(plan_id)
    .bind(payload.get("modules"))
    .bind(total_value)
    .bind(payload.get("discount").and_then(|v| v.as_f64()).unwrap_or(0.0))
    .bind(payload.get("notes").and_then(|v| v.as_str()))
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    log_activity(&state.pool, prospect_id, "proposal", "Propuesta creada", None).await;

    Ok(Json(json!({"id": id})))
}

async fn get_proposal(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let proposal = sqlx::query_as::<_, models::SalesProposal>(
        "SELECT id, prospect_id, plan_id, modules, total_value, discount, version, status, notes, created_by, created_at
         FROM crm_sales_proposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Propuesta no encontrada".into()))?;

    Ok(Json(json!({"proposal": proposal})))
}

async fn apply_discount(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    let discount = payload.get("discount").and_then(|v| v.as_f64()).unwrap_or(0.0);

    sqlx::query("UPDATE crm_sales_proposals SET discount = $1, version = version + 1 WHERE id = $2")
        .bind(discount)
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Descuento aplicado"})))
}

// ─── Contracts ───

async fn create_contract(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<models::CreateContractPayload>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO crm_sales_contracts (id, prospect_id, plan_id, modules, total_value, discount, status, notes)
         VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7)",
    )
    .bind(id)
    .bind(payload.prospect_id)
    .bind(payload.plan_id)
    .bind(&payload.modules)
    .bind(payload.total_value)
    .bind(payload.discount.unwrap_or(0.0))
    .bind(&payload.notes)
    .execute(&state.pool)
    .await?;

    log_activity(&state.pool, payload.prospect_id, "contract", "Contrato creado", None).await;

    Ok(Json(json!({"id": id})))
}

async fn get_contract(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let contract = sqlx::query_as::<_, models::SalesContract>(
        "SELECT id, prospect_id, plan_id, modules, total_value, discount, status,
         signed_at, verified_at, activated_at, notes, created_at, updated_at
         FROM crm_sales_contracts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Contrato no encontrado".into()))?;

    let documents = sqlx::query_as::<_, models::SalesDocument>(
        "SELECT id, contract_id, file_name, file_url, doc_type, is_verified, uploaded_by, created_at
         FROM crm_sales_documents WHERE contract_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"contract": contract, "documents": documents})))
}

async fn verify_signatures(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    sqlx::query(
        "UPDATE crm_sales_contracts SET verified_at = NOW(), status = 'verified', updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Firmas verificadas"})))
}

async fn activate_license(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_manager(&claims)?;

    let contract = sqlx::query_as::<_, models::SalesContract>(
        "SELECT id, prospect_id, tax_id, plan_id, modules, total_value, discount, status,
         signed_at, verified_at, activated_at, notes, created_at, updated_at
         FROM crm_sales_contracts WHERE id = $1 AND status = 'verified'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Contrato no encontrado o no verificado".into()))?;

    let prospect = sqlx::query_as::<_, models::SalesProspect>(
        "SELECT id, first_name, last_name, rut, email, phone, company, position, source,
         requirements, current_stage_id, assigned_to, estimated_value, notes, created_at, updated_at
         FROM crm_sales_prospects WHERE id = $1",
    )
    .bind(contract.prospect_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(CrmError::NotFound("Prospecto no encontrado".into()))?;

    let plan_id = contract.plan_id.ok_or(CrmError::Internal("El contrato no tiene un plan asociado".into()))?;

    // 1. Prepare onboarding payload for Identity
    let onboarding_payload = OnboardingPayload {
        corporation_name: prospect.company.clone().unwrap_or_else(|| format!("{} {}", prospect.first_name, prospect.last_name)),
        corporation_rut: contract.tax_id.clone().or(prospect.rut.clone()).unwrap_or_else(|| "0.0.0.0-0".into()),
        school_name: format!("Colegio {}", prospect.company.as_deref().unwrap_or(&prospect.first_name)),
        admin_name: format!("{} {}", prospect.first_name, prospect.last_name),
        admin_email: prospect.email.clone().unwrap_or_default(),
        admin_rut: prospect.rut.clone().unwrap_or_else(|| "0.0.0.0-0".into()),
        plan_id,
        modules: None, // Will use plan defaults for now
    };

    let onboarding_url = format!("{}/api/internal/onboarding", state.config.identity_url);
    let onboarding_resp = state.client.post(&onboarding_url)
        .header("X-Internal-Secret", &state.config.internal_api_secret)
        .json(&onboarding_payload)
        .send()
        .await
        .map_err(|e| CrmError::External(format!("Error llamando a Identity: {e}")))?;

    if !onboarding_resp.status().is_success() {
        let err_text = onboarding_resp.text().await.unwrap_or_default();
        return Err(CrmError::External(format!("Error en onboarding de Identity: {err_text}")));
    }

    let onboarding_data: Value = onboarding_resp.json().await
        .map_err(|e| CrmError::Internal(format!("Error parseando respuesta de onboarding: {e}")))?;

    let corp_id = onboarding_data.get("corporation_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(CrmError::Internal("No se recibió corporation_id de Identity".into()))?;

    // 2. Create corporation_license in CRM DB
    let license_id = Uuid::new_v4();
    let start_date = chrono::Utc::now().date_naive();
    let end_date = start_date + chrono::Duration::days(365);

    sqlx::query(
        "INSERT INTO corporation_licenses (id, corporation_id, plan_id, start_date, end_date, status, grace_period_days)
         VALUES ($1, $2, $3, $4, $5, 'active', 30)",
    )
    .bind(license_id)
    .bind(corp_id)
    .bind(plan_id)
    .bind(start_date)
    .bind(end_date)
    .execute(&state.pool)
    .await?;

    // 3. Mark contract as activated
    sqlx::query(
        "UPDATE crm_sales_contracts SET status = 'active', activated_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    // 4. Send Welcome Email (Simulation for now)
    let temp_pass = onboarding_data["temp_password"].as_str().unwrap_or("****");
    tracing::info!("📧 BIENVENIDA: Enviando credenciales a {} -> Password: {}", prospect.email.as_deref().unwrap_or("-"), temp_pass);

    // 4. Update prospect to "Cerrado Ganado"
    let won_stage: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM crm_sales_stages WHERE is_final = true AND name ILIKE '%Ganado%' LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some((stage_id,)) = won_stage {
        sqlx::query(
            "UPDATE crm_sales_prospects SET current_stage_id = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(stage_id)
        .bind(contract.prospect_id)
        .execute(&state.pool)
        .await?;
    }

    log_activity(&state.pool, contract.prospect_id, "activation", "Licencia activada y onboarding completado", None).await;

    Ok(Json(json!({
        "message": "Licencia activada correctamente",
        "onboarding": onboarding_data
    })))
}

// ─── Documents ───

async fn list_documents(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let documents = sqlx::query_as::<_, models::SalesDocument>(
        "SELECT id, contract_id, file_name, file_url, doc_type, is_verified, uploaded_by, created_at
         FROM crm_sales_documents WHERE contract_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(json!({"documents": documents})))
}

async fn upload_document(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::CreateDocumentPayload>,
) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let doc_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO crm_sales_documents (id, contract_id, file_name, doc_type, uploaded_by) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(doc_id)
    .bind(id)
    .bind(&payload.file_name)
    .bind(&payload.doc_type)
    .bind(user_id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"id": doc_id})))
}

// ─── Plans ───

async fn list_plans(claims: Claims, State(state): State<AppState>) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    #[derive(sqlx::FromRow)]
    struct PlanRow {
        id: Uuid,
        name: String,
        description: Option<String>,
        price_monthly: f64,
        price_yearly: f64,
        featured: bool,
        sort_order: i32,
    }

    let plans = sqlx::query_as::<_, PlanRow>(
        "SELECT id, name, description, price_monthly, price_yearly, featured, sort_order
         FROM license_plans WHERE active = true ORDER BY sort_order",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut result = Vec::new();
    for p in plans {
        let modules: Vec<Value> = sqlx::query_as::<_, (String, String, bool)>(
            "SELECT module_key, module_name, included FROM plan_modules WHERE plan_id = $1 ORDER BY module_key",
        )
        .bind(p.id)
        .fetch_all(&state.pool)
        .await.unwrap_or_default()
        .into_iter()
        .map(|(k, n, inc)| json!({"key": k, "name": n, "included": inc}))
        .collect();

        result.push(json!({
            "id": p.id, "name": p.name, "description": p.description,
            "price_monthly": p.price_monthly, "price_yearly": p.price_yearly,
            "featured": p.featured, "sort_order": p.sort_order,
            "modules": modules,
        }));
    }

    Ok(Json(json!({"plans": result})))
}

// ─── Dashboard ───

async fn dashboard_summary(claims: Claims, State(state): State<AppState>) -> CrmResult<Json<Value>> {
    require_sales_role(&claims)?;

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let is_agent = claims.role == "AgenteVentas";

    let my_prospects = if is_agent {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crm_sales_prospects WHERE assigned_to = $1")
            .bind(user_id)
            .fetch_one(&state.pool)
            .await.unwrap_or((0,));
        count.0
    } else {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crm_sales_prospects")
            .fetch_one(&state.pool)
            .await.unwrap_or((0,));
        count.0
    };

    let total_prospects: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crm_sales_prospects")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let total_contracts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crm_sales_contracts WHERE status IN ('active', 'verified')")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let total_value: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total_value), 0) FROM crm_sales_contracts WHERE status IN ('active', 'verified')",
    )
    .fetch_one(&state.pool).await.unwrap_or((Some(0.0),));

    let stages: Vec<Value> = sqlx::query_as::<_, (String, i64)>(
        "SELECT s.name, COUNT(p.id) FROM crm_sales_stages s
         LEFT JOIN crm_sales_prospects p ON p.current_stage_id = s.id
         GROUP BY s.name, s.sort_order ORDER BY s.sort_order",
    )
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(name, count)| json!({"name": name, "count": count}))
    .collect();

    Ok(Json(json!({
        "my_prospects": my_prospects,
        "total_prospects": total_prospects.0,
        "total_contracts": total_contracts.0,
        "total_value": total_value.0.unwrap_or(0.0),
        "pipeline": stages,
    })))
}

// ─── Helpers ───

async fn log_activity(pool: &sqlx::PgPool, prospect_id: Uuid, activity_type: &str, subject: &str, scheduled_at: Option<chrono::DateTime<chrono::Utc>>) {
    let _ = sqlx::query(
        "INSERT INTO crm_sales_activities (id, prospect_id, activity_type, subject, scheduled_at, is_completed)
         VALUES ($1, $2, $3, $4, $5, true)",
    )
    .bind(Uuid::new_v4())
    .bind(prospect_id)
    .bind(activity_type)
    .bind(subject)
    .bind(scheduled_at)
    .execute(pool)
    .await;
}
