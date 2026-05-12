use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

use crate::AppState;
use crate::error::{AuthError, AuthResult};
use crate::models::Claims;
use crate::routes::require_role;

fn require_root(claims: &Claims) -> Result<(), AuthError> {
    require_role(claims, "Root")
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        // Stats
        .route("/api/admin/stats/summary", get(stats_summary))
        .route("/api/admin/stats/monthly", get(stats_monthly))
        .route("/api/admin/stats/license-distribution", get(stats_license_distribution))
        // Corporations
        .route("/api/admin/corporations", get(admin_list_corporations).post(admin_create_corporation))
        .route("/api/admin/corporations/{id}/toggle", put(admin_toggle_corporation))
        .route("/api/admin/corporations/{id}", get(admin_get_corporation))
        // Plans
        .route("/api/admin/license-plans", get(admin_list_plans).post(admin_create_plan))
        .route("/api/admin/license-plans/{id}", put(admin_update_plan).delete(admin_delete_plan))
        .route("/api/admin/license-plans/{id}/modules", post(admin_set_plan_modules))
        // Licenses
        .route("/api/admin/licenses", get(admin_list_licenses).post(admin_assign_license))
        .route("/api/admin/licenses/{id}/extend", put(admin_extend_license))
        .route("/api/admin/licenses/{id}/change-plan", put(admin_change_plan))
        .route("/api/admin/licenses/{id}/status", put(admin_update_license_status))
        // Payments
        .route("/api/admin/payments", get(admin_list_payments).post(admin_register_payment))
        // Activity log
        .route("/api/admin/activity-log", get(admin_activity_log))
        // Health
        .route("/api/admin/system/health", get(admin_system_health))
        // Public endpoints (no auth)
        .route("/api/public/plans", get(public_plans))
        .route("/api/public/features", get(public_features))
        .route("/api/public/contact", post(public_contact))
}

// ─── Stats ───

async fn stats_summary(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let total_corporations: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM corporations").fetch_one(&state.pool).await?;
    let active_corporations: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM corporations WHERE active = true").fetch_one(&state.pool).await?;
    let total_schools: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM schools").fetch_one(&state.pool).await?;
    let total_students: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM students").fetch_one(&state.pool).await?;
    let total_employees: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM employees").fetch_one(&state.pool).await?;
    let active_licenses: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM corporation_licenses WHERE status = 'active'")
            .fetch_one(&state.pool).await.unwrap_or((0,));
    let expiring_licenses: (i64,) =
        sqlx::query_as(
            "SELECT COUNT(*) FROM corporation_licenses WHERE status = 'active' AND end_date IS NOT NULL AND end_date <= CURRENT_DATE + INTERVAL '30 days'",
        )
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let monthly_revenue: (Option<f64>,) =
        sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM license_payments WHERE status = 'completed' AND paid_at >= date_trunc('month', CURRENT_DATE)",
        )
        .fetch_one(&state.pool).await.unwrap_or((Some(0.0),));
    let total_revenue: (Option<f64>,) =
        sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM license_payments WHERE status = 'completed'",
        )
        .fetch_one(&state.pool).await.unwrap_or((Some(0.0),));

    Ok(Json(json!({
        "total_corporations": total_corporations.0,
        "active_corporations": active_corporations.0,
        "total_schools": total_schools.0,
        "total_students": total_students.0,
        "total_employees": total_employees.0,
        "active_licenses": active_licenses.0,
        "expiring_licenses": expiring_licenses.0,
        "monthly_revenue": monthly_revenue.0.unwrap_or(0.0),
        "total_revenue": total_revenue.0.unwrap_or(0.0),
    })))
}

async fn stats_monthly(claims: Claims, State(state): State<AppState>) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let rows: Vec<(String, Option<f64>,)> = sqlx::query_as(
        "SELECT to_char(paid_at, 'YYYY-MM') as month, SUM(amount) FROM license_payments
         WHERE status = 'completed' AND paid_at >= CURRENT_DATE - INTERVAL '12 months'
         GROUP BY month ORDER BY month",
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    let data: Vec<Value> = rows
        .into_iter()
        .map(|(m, a)| json!({"month": m, "revenue": a.unwrap_or(0.0)}))
        .collect();

    Ok(Json(json!({"monthly": data})))
}

async fn stats_license_distribution(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let rows: Vec<(String, i64,)> = sqlx::query_as(
        "SELECT lp.name, COUNT(cl.id) FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE cl.status = 'active'
         GROUP BY lp.name, lp.sort_order ORDER BY lp.sort_order",
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    Ok(Json(json!({"distribution": rows})))
}

// ─── Corporations ───

#[derive(Deserialize)]
struct AdminCreateCorpPayload {
    name: String,
    rut: Option<String>,
    plan_id: Uuid,
    start_date: Option<String>,
}

async fn admin_list_corporations(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let rows: Vec<Value> = sqlx::query_as::<_, (Uuid, String, Option<String>, bool, Option<String>, Option<String>, i64, i64)>(
        "SELECT c.id, c.name, c.rut, c.active, lp.name as plan_name, cl.status as license_status,
                COALESCE((SELECT COUNT(*) FROM schools s WHERE s.corporation_id = c.id), 0) as total_schools,
                COALESCE((SELECT COUNT(*) FROM students st JOIN schools sc ON sc.id = st.school_id WHERE sc.corporation_id = c.id), 0) as total_students
         FROM corporations c
         LEFT JOIN corporation_licenses cl ON cl.corporation_id = c.id AND cl.status = 'active'
         LEFT JOIN license_plans lp ON lp.id = cl.plan_id
         ORDER BY c.name",
    )
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(id, name, rut, active, plan, license_status, schools, students)| {
        json!({
            "id": id,
            "name": name,
            "rut": rut,
            "active": active,
            "plan_name": plan,
            "license_status": license_status,
            "total_schools": schools,
            "total_students": students,
        })
    })
    .collect();

    Ok(Json(json!({"corporations": rows})))
}

async fn admin_create_corporation(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateCorpPayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let corp_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO corporations (id, name, rut) VALUES ($1, $2, $3)",
    )
    .bind(corp_id)
    .bind(&payload.name)
    .bind(&payload.rut)
    .execute(&state.pool)
    .await?;

    let plan_id = payload.plan_id;
    let start_date = payload
        .start_date
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    sqlx::query(
        "INSERT INTO corporation_licenses (id, corporation_id, plan_id, start_date, status)
         VALUES ($1, $2, $3, $4, 'active')",
    )
    .bind(Uuid::new_v4())
    .bind(corp_id)
    .bind(plan_id)
    .bind(start_date)
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool, &claims, "create_corporation", "corporation", Some(corp_id),
        &json!({"name": &payload.name, "plan_id": plan_id}),
    ).await;

    Ok(Json(json!({"id": corp_id, "name": payload.name})))
}

async fn admin_get_corporation(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let corp = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, bool)>(
        "SELECT id, name, rut, logo_url, active FROM corporations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AuthError::NotFound("Corporación no encontrada".into()))?;

    let schools: Vec<Value> = sqlx::query_as::<_, (Uuid, String, bool)>(
        "SELECT id, name, active FROM schools WHERE corporation_id = $1 ORDER BY name",
    )
    .bind(id)
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(sid, sname, sactive)| json!({"id": sid, "name": sname, "active": sactive}))
    .collect();

    let license = sqlx::query_as::<_, (Uuid, String, Option<chrono::NaiveDate>, String)>(
        "SELECT cl.id, lp.name, cl.end_date, cl.status FROM corporation_licenses cl
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE cl.corporation_id = $1 AND cl.status = 'active'
         LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.pool).await.unwrap_or(None);

    Ok(Json(json!({
        "id": corp.0, "name": corp.1, "rut": corp.2, "logo_url": corp.3, "active": corp.4,
        "schools": schools,
        "license": license.map(|(lid, pname, end, status)| json!({
            "id": lid, "plan_name": pname, "end_date": end, "status": status
        })),
    })))
}

async fn admin_toggle_corporation(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    sqlx::query("UPDATE corporations SET active = NOT active WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_admin_action(
        &state.pool, &claims, "toggle_corporation", "corporation", Some(id), &json!({}),
    ).await;

    Ok(Json(json!({"message": "Corporación actualizada"})))
}

// ─── License Plans ───

async fn admin_list_plans(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let plans: Vec<Value> = sqlx::query_as::<_, (Uuid, String, Option<String>, f64, f64, bool, i32, bool)>(
        "SELECT id, name, description, price_monthly, price_yearly, featured, sort_order, active
         FROM license_plans ORDER BY sort_order",
    )
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(id, name, desc, pm, py, feat, sort, active)| {
        json!({"id": id, "name": name, "description": desc, "price_monthly": pm, "price_yearly": py, "featured": feat, "sort_order": sort, "active": active})
    })
    .collect();

    Ok(Json(json!({"plans": plans})))
}

async fn admin_create_plan(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::licensing::CreateLicensePlanPayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let plan_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO license_plans (id, name, description, price_monthly, price_yearly, featured, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(plan_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(payload.price_monthly)
    .bind(payload.price_yearly)
    .bind(payload.featured)
    .bind(payload.sort_order)
    .execute(&state.pool)
    .await?;

    for m in &payload.modules {
        sqlx::query(
            "INSERT INTO plan_modules (id, plan_id, module_key, module_name, included) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(plan_id)
        .bind(&m.module_key)
        .bind(&m.module_name)
        .bind(m.included)
        .execute(&state.pool).await?;
    }

    log_admin_action(
        &state.pool, &claims, "create_plan", "license_plan", Some(plan_id),
        &json!({"name": &payload.name}),
    ).await;

    Ok(Json(json!({"id": plan_id})))
}

async fn admin_update_plan(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    sqlx::query(
        "UPDATE license_plans SET name = COALESCE($1, name), description = COALESCE($2, description),
         price_monthly = COALESCE($3, price_monthly), price_yearly = COALESCE($4, price_yearly),
         featured = COALESCE($5, featured), active = COALESCE($6, active)
         WHERE id = $7",
    )
    .bind(payload.get("name").and_then(|v| v.as_str()))
    .bind(payload.get("description").and_then(|v| v.as_str()))
    .bind(payload.get("price_monthly").and_then(|v| v.as_f64()))
    .bind(payload.get("price_yearly").and_then(|v| v.as_f64()))
    .bind(payload.get("featured").and_then(|v| v.as_bool()))
    .bind(payload.get("active").and_then(|v| v.as_bool()))
    .bind(id)
    .execute(&state.pool)
    .await?;

    Ok(Json(json!({"message": "Plan actualizado"})))
}

async fn admin_delete_plan(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    sqlx::query("DELETE FROM license_plans WHERE id = $1 AND active = false")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Plan eliminado"})))
}

async fn admin_set_plan_modules(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::licensing::CreateLicensePlanPayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    sqlx::query("DELETE FROM plan_modules WHERE plan_id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    for m in &payload.modules {
        sqlx::query(
            "INSERT INTO plan_modules (id, plan_id, module_key, module_name, included) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(&m.module_key)
        .bind(&m.module_name)
        .bind(m.included)
        .execute(&state.pool).await?;
    }

    Ok(Json(json!({"message": "Módulos actualizados"})))
}

// ─── Licenses ───

#[derive(Deserialize)]
struct LicenseQuery {
    status: Option<String>,
    corporation_id: Option<Uuid>,
}

async fn admin_list_licenses(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<LicenseQuery>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let rows: Vec<Value> = sqlx::query_as::<_, (Uuid, String, String, Option<chrono::NaiveDate>, String, Option<chrono::NaiveDate>, String, i64)>(
        "SELECT cl.id, c.name as corporation_name, lp.name as plan_name,
                cl.end_date, cl.status, cl.start_date, cl.notes,
                COALESCE(EXTRACT(DAY FROM cl.end_date - CURRENT_DATE)::BIGINT, 0) as days_remaining
         FROM corporation_licenses cl
         JOIN corporations c ON c.id = cl.corporation_id
         JOIN license_plans lp ON lp.id = cl.plan_id
         WHERE ($1::varchar IS NULL OR cl.status = $1)
           AND ($2::uuid IS NULL OR cl.corporation_id = $2)
         ORDER BY cl.created_at DESC",
    )
    .bind(q.status.as_deref())
    .bind(q.corporation_id)
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(id, corp, plan, end, status, start, notes, days)| {
        json!({"id": id, "corporation_name": corp, "plan_name": plan, "start_date": start, "end_date": end, "status": status, "notes": notes, "days_remaining": days})
    })
    .collect();

    Ok(Json(json!({"licenses": rows})))
}

async fn admin_assign_license(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::licensing::AssignLicensePayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO corporation_licenses (id, corporation_id, plan_id, start_date, end_date, auto_renew, grace_period_days)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(payload.corporation_id)
    .bind(payload.plan_id)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.auto_renew)
    .bind(payload.grace_period_days.unwrap_or(30))
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool, &claims, "assign_license", "corporation_license", Some(id),
        &json!({"corporation_id": payload.corporation_id, "plan_id": payload.plan_id}),
    ).await;

    Ok(Json(json!({"id": id})))
}

async fn admin_extend_license(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<schoolccb_common::licensing::ExtendLicensePayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let admin_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::TokenInvalid("Invalid user ID".into()))?;

    sqlx::query(
        "INSERT INTO license_extensions (id, corporation_license_id, days_extended, reason, approved_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(payload.days)
    .bind(&payload.reason)
    .bind(admin_id)
    .execute(&state.pool)
    .await?;

    sqlx::query(
        "UPDATE corporation_licenses SET end_date = COALESCE(end_date, CURRENT_DATE) + $1::integer, updated_at = NOW() WHERE id = $2",
    )
    .bind(payload.days)
    .bind(id)
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool, &claims, "extend_license", "corporation_license", Some(id),
        &json!({"days": payload.days, "reason": &payload.reason}),
    ).await;

    Ok(Json(json!({"message": "Licencia prorrogada"})))
}

async fn admin_change_plan(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let new_plan_id = payload
        .get("plan_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AuthError::Internal("plan_id es requerido".into()))?;

    sqlx::query(
        "UPDATE corporation_licenses SET plan_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(new_plan_id)
    .bind(id)
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool, &claims, "change_plan", "corporation_license", Some(id),
        &json!({"new_plan_id": new_plan_id}),
    ).await;

    Ok(Json(json!({"message": "Plan cambiado"})))
}

async fn admin_update_license_status(
    claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let status = payload
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::Internal("status es requerido".into()))?;

    sqlx::query("UPDATE corporation_licenses SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(&state.pool)
        .await?;

    log_admin_action(
        &state.pool, &claims, "update_license_status", "corporation_license", Some(id),
        &json!({"status": status}),
    ).await;

    Ok(Json(json!({"message": "Estado actualizado"})))
}

// ─── Payments ───

async fn admin_list_payments(
    claims: Claims,
    State(state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let rows: Vec<Value> = sqlx::query_as::<_, (Uuid, String, f64, String, String, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT lp.id, c.name as corporation_name, lp.amount, lp.payment_method, lp.status, lp.transaction_id, lp.paid_at
         FROM license_payments lp
         JOIN corporation_licenses cl ON cl.id = lp.corporation_license_id
         JOIN corporations c ON c.id = cl.corporation_id
         ORDER BY lp.created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(id, corp, amount, method, status, tx, paid_at)| {
        json!({"id": id, "corporation_name": corp, "amount": amount, "payment_method": method, "status": status, "transaction_id": tx, "paid_at": paid_at})
    })
    .collect();

    Ok(Json(json!({"payments": rows})))
}

async fn admin_register_payment(
    claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<schoolccb_common::licensing::RegisterPaymentPayload>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let id = Uuid::new_v4();
    let year = chrono::Utc::now().format("%Y");
    let correlative: i64 = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) + 1 FROM license_payments WHERE EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM CURRENT_DATE)",
    )
    .fetch_one(&state.pool).await.unwrap_or((1,)).0;
    let transaction_id = format!("PAY-{}-{:04}", year, correlative);

    sqlx::query(
        "INSERT INTO license_payments (id, corporation_license_id, amount, currency, payment_method, status, transaction_id, paid_at, period_start, period_end, notes)
         VALUES ($1, $2, $3, $4, $5, 'completed', $6, NOW(), $7, $8, $9)",
    )
    .bind(id)
    .bind(payload.corporation_license_id)
    .bind(payload.amount)
    .bind(payload.currency.unwrap_or_else(|| "CLP".into()))
    .bind(&payload.payment_method)
    .bind(&transaction_id)
    .bind(payload.period_start)
    .bind(payload.period_end)
    .bind(&payload.notes)
    .execute(&state.pool)
    .await?;

    log_admin_action(
        &state.pool, &claims, "register_payment", "license_payment", Some(id),
        &json!({"amount": payload.amount, "corporation_license_id": payload.corporation_license_id}),
    ).await;

    Ok(Json(json!({"id": id, "transaction_id": transaction_id})))
}

// ─── Activity Log ───

#[derive(Deserialize)]
struct ActivityLogQuery {
    action: Option<String>,
    limit: Option<i64>,
}

async fn admin_activity_log(
    claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<ActivityLogQuery>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;

    let limit = q.limit.unwrap_or(100);

    let rows: Vec<Value> = sqlx::query_as::<_, (String, String, String, Option<String>, Option<serde_json::Value>, chrono::DateTime<chrono::Utc>)>(
        "SELECT u.name as admin_name, a.action, a.entity_type, a.entity_id::varchar, a.details, a.created_at
         FROM admin_activity_log a
         JOIN users u ON u.id = a.admin_id
         WHERE ($1::varchar IS NULL OR a.action = $1)
         ORDER BY a.created_at DESC LIMIT $2",
    )
    .bind(q.action.as_deref())
    .bind(limit)
    .fetch_all(&state.pool).await.unwrap_or_default()
    .into_iter()
    .map(|(admin, action, entity, eid, details, created)| {
        json!({"admin": admin, "action": action, "entity_type": entity, "entity_id": eid, "details": details, "created_at": created})
    })
    .collect();

    Ok(Json(json!({"activity_log": rows})))
}

// ─── System Health ───

async fn admin_system_health(
    claims: Claims,
    State(_state): State<AppState>,
) -> AuthResult<Json<Value>> {
    require_root(&claims)?;
    Ok(Json(json!({
        "status": "ok",
        "services": [
            {"name": "identity", "status": "healthy"},
            {"name": "gateway", "status": "healthy"},
        ]
    })))
}

// ─── Public Endpoints (no auth required) ───

#[derive(Serialize, FromRow)]
struct PublicPlan {
    id: Uuid,
    name: String,
    description: Option<String>,
    price_monthly: f64,
    price_yearly: f64,
    featured: bool,
    sort_order: i32,
}

async fn public_plans(State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let plan_rows = sqlx::query_as::<_, PublicPlan>(
        "SELECT id, name, description, price_monthly, price_yearly, featured, sort_order
         FROM license_plans WHERE active = true ORDER BY sort_order",
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    let mut plans = Vec::new();
    for p in plan_rows {
        let modules: Vec<Value> = sqlx::query_as::<_, (String, String, bool)>(
            "SELECT module_key, module_name, included FROM plan_modules WHERE plan_id = $1 ORDER BY module_key",
        )
        .bind(p.id)
        .fetch_all(&state.pool).await.unwrap_or_default()
        .into_iter()
        .map(|(k, n, inc)| json!({"key": k, "name": n, "included": inc}))
        .collect();

        plans.push(json!({
            "id": p.id, "name": p.name, "description": p.description,
            "price_monthly": p.price_monthly, "price_yearly": p.price_yearly,
            "featured": p.featured, "sort_order": p.sort_order,
            "modules": modules
        }));
    }

    Ok(Json(json!({"plans": plans})))
}

async fn public_features(State(state): State<AppState>) -> AuthResult<Json<Value>> {
    let module_keys: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT module_key FROM plan_modules ORDER BY module_key")
            .fetch_all(&state.pool).await.unwrap_or_default();

    let features: Vec<&str> = module_keys.iter().map(|(k,)| k.as_str()).collect();
    Ok(Json(json!({"features": features})))
}

async fn public_contact(
    Json(payload): Json<Value>,
) -> AuthResult<Json<Value>> {
    let _name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let _email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let _message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("");

    tracing::info!("Contact form submission: {_name} <{_email}>: {_message}");

    Ok(Json(json!({"message": "Mensaje recibido. Te contactaremos pronto."})))
}

// ─── Helpers ───

async fn log_admin_action(
    pool: &sqlx::PgPool,
    claims: &Claims,
    action: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    details: &Value,
) {
    let admin_id = Uuid::parse_str(&claims.sub).ok();
    if let Some(aid) = admin_id {
        let _ = sqlx::query(
            "INSERT INTO admin_activity_log (id, admin_id, action, entity_type, entity_id, details)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(aid)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(details)
        .execute(pool)
        .await;
    }
}
