pub mod graphql;

#[cfg(test)]
mod tests;

use axum::http::HeaderMap;

pub fn extract_jwt_from_cookie(headers: &HeaderMap) -> Option<String> {
    headers.get("cookie")?
        .to_str().ok()?
        .split(';')
        .find_map(|c| c.trim().strip_prefix("jwt_token="))
        .map(|v| v.to_string())
}
