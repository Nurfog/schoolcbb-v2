use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchRequest {
    q: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResult {
    text: String,
    fuente: String,
    nivel: String,
    asignatura: String,
    score: usize,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/curriculum/search", post(search))
        .route("/api/curriculum/info", get(info))
}

async fn info(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "total_chunks": state.kb.len(),
        "status": "ok"
    }))
}

async fn search(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let query = payload.q.trim().to_lowercase();
    if query.is_empty() {
        return Ok(Json(json!({ "results": [] })));
    }

    let limit = payload.limit.unwrap_or(10).min(50);
    let query_words: Vec<&str> = query.split_whitespace().collect();

    let mut scored: Vec<(usize, &crate::KbEntry)> = state
        .kb
        .iter()
        .filter_map(|entry| {
            let lower = entry.text.to_lowercase();
            let matches: usize = query_words
                .iter()
                .filter(|w| lower.contains(*w))
                .count();
            if matches > 0 {
                Some((matches, entry))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let results: Vec<SearchResult> = scored
        .into_iter()
        .take(limit)
        .map(|(score, entry)| SearchResult {
            text: entry.text.clone(),
            fuente: entry.fuente.clone(),
            nivel: entry.nivel.clone(),
            asignatura: entry.asignatura.clone(),
            score,
        })
        .collect();

    Ok(Json(json!({ "results": results })))
}
