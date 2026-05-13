use axum::http::HeaderMap;

use crate::extract_jwt_from_cookie;

#[test]
fn test_extract_jwt_from_cookie_returns_none_for_empty_headers() {
    let headers = HeaderMap::new();
    assert!(extract_jwt_from_cookie(&headers).is_none());
}

#[test]
fn test_extract_jwt_from_cookie_returns_none_for_no_cookie_header() {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "application/json".parse().unwrap());
    assert!(extract_jwt_from_cookie(&headers).is_none());
}

#[test]
fn test_extract_jwt_from_cookie_returns_token_when_present() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        "jwt_token=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dummy; other=val"
            .parse()
            .unwrap(),
    );
    let token = extract_jwt_from_cookie(&headers);
    assert_eq!(
        token.as_deref(),
        Some("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dummy")
    );
}

#[test]
fn test_extract_jwt_from_cookie_returns_none_for_wrong_cookie_name() {
    let mut headers = HeaderMap::new();
    headers.insert("cookie", "session_id=abc123".parse().unwrap());
    assert!(extract_jwt_from_cookie(&headers).is_none());
}
