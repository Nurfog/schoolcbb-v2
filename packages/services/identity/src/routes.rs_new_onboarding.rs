async fn onboarding(
    State(state): State<AppState>,
    parts: axum::http::request::Parts,
    Json(payload): Json<schoolccb_common::school::OnboardingPayload>,
) -> AuthResult<Json<Value>> {
    // 1. Validation of internal secret
    let secret = parts.headers.get("X-Internal-Secret")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthError::Unauthorized("Falta secreto interno".into()))?;

    if secret != state.config.internal_api_secret {
        return Err(AuthError::Unauthorized("Secreto interno inválido".into()));
    }

    let mut tx = state.pool.begin().await?;

    // 2. Create Corporation
    let corp_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO corporations (id, name, rut, active) VALUES ($1, $2, $3, true)",
    )
    .bind(corp_id)
    .bind(&payload.corporation_name)
    .bind(&payload.corporation_rut)
    .execute(&mut *tx)
    .await?;

    // 3. Create Default School
    let school_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO schools (id, corporation_id, name, active) VALUES ($1, $2, $3, true)",
    )
    .bind(school_id)
    .bind(corp_id)
    .bind(&payload.school_name)
    .execute(&mut *tx)
    .await?;

    // 4. Create Admin User (Sostenedor)
    let temp_password = format!("C-{}", &Uuid::new_v4().to_string()[..8]);
    let hash = models::hash_password(&temp_password);
    let user_id = Uuid::new_v4();
    
    sqlx::query(
        "INSERT INTO users (id, rut, name, email, password_hash, role, active, corporation_id, school_id, admin_type)
         VALUES ($1, $2, $3, $4, $5, 'Sostenedor', true, $6, $7, 'global')",
    )
    .bind(user_id)
    .bind(&payload.admin_rut)
    .bind(&payload.admin_name)
    .bind(&payload.admin_email)
    .bind(&hash)
    .bind(corp_id)
    .bind(school_id)
    .execute(&mut *tx)
    .await?;

    // 5. Activate Modules (if provided)
    if let Some(modules) = payload.modules {
        for module_id in modules {
            sqlx::query(
                "INSERT INTO corporation_modules (corporation_id, module_id, active)
                 VALUES ($1, $2, true)
                 ON CONFLICT (corporation_id, module_id) DO UPDATE SET active = true",
            )
            .bind(corp_id)
            .bind(module_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(Json(json!({
        "message": "Onboarding completado",
        "corporation_id": corp_id,
        "school_id": school_id,
        "user_id": user_id,
        "temp_password": temp_password
    })))
}
