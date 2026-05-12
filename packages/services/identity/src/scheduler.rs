use sqlx::PgPool;

pub async fn start(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));
        loop {
            interval.tick().await;
            check_licenses(&pool).await;
        }
    });
    tracing::info!("License scheduler started (daily check)");
}

async fn check_licenses(pool: &PgPool) {
    tracing::info!("Running daily license check...");

    // Licenses expiring in 30 days
    let expiring_30: Vec<(uuid::Uuid, String, chrono::NaiveDate)> = sqlx::query_as(
        "SELECT cl.id, c.name, cl.end_date FROM corporation_licenses cl
         JOIN corporations c ON c.id = cl.corporation_id
         WHERE cl.status = 'active' AND cl.end_date IS NOT NULL
           AND cl.end_date <= CURRENT_DATE + INTERVAL '30 days'
           AND cl.end_date > CURRENT_DATE",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (id, name, end_date) in &expiring_30 {
        let days = (*end_date - chrono::Utc::now().date_naive()).num_days();
        tracing::info!("License {id} for {name} expiring in {days} days (end: {end_date})");
    }

    // Expire licenses past end_date
    let expired: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT cl.id, c.name FROM corporation_licenses cl
         JOIN corporations c ON c.id = cl.corporation_id
         WHERE cl.status = 'active' AND cl.end_date IS NOT NULL AND cl.end_date < CURRENT_DATE",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (id, name) in &expired {
        tracing::warn!("License {id} for {name} has expired — setting status to 'expired'");
        let _ = sqlx::query("UPDATE corporation_licenses SET status = 'expired', updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }

    tracing::info!(
        "License check complete: {} expiring, {} expired",
        expiring_30.len(),
        expired.len()
    );
}
