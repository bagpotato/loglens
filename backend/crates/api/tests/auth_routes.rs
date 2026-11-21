mod common;

use axum::http::StatusCode;
use common::{body_json, get, login, post_json, send, setup, ADMIN, PASSWORD};
use serde_json::json;

#[tokio::test]
async fn health_is_public() {
    let app = setup(Vec::new()).await;
    let response = send(&app, get("/api/health", None)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, json!({ "status": "ok" }));
}

#[tokio::test]
async fn protected_routes_require_token() {
    let app = setup(Vec::new()).await;
    for uri in ["/api/logs", "/api/meta", "/api/stats", "/api/logs/tail"] {
        let response = send(&app, get(uri, None)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
    for uri in ["/api/ai/explain", "/api/ai/chat"] {
        let response = send(&app, post_json(uri, None, &json!({}))).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
    let response = send(&app, get("/api/meta", Some("not-a-jwt"))).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(body_json(response).await["error"].is_string());
}

#[tokio::test]
async fn login_returns_usable_token() {
    let app = setup(Vec::new()).await;
    let body = json!({ "username": ADMIN, "password": PASSWORD });
    let response = send(&app, post_json("/api/auth/login", None, &body)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["username"], ADMIN);
    assert!(json["role"].is_string());

    let token = login(&app).await;
    let response = send(&app, get("/api/meta", Some(&token))).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response).await,
        json!({ "hosts": ["web1"], "sources": ["ssh"] })
    );
}

#[tokio::test]
async fn login_rejects_bad_password() {
    let app = setup(Vec::new()).await;
    let body = json!({ "username": ADMIN, "password": "wrong" });
    let response = send(&app, post_json("/api/auth/login", None, &body)).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(body_json(response).await["error"].is_string());

    let body = json!({ "username": "nobody", "password": PASSWORD });
    let response = send(&app, post_json("/api/auth/login", None, &body)).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn malformed_login_body_is_json_error() {
    let app = setup(Vec::new()).await;
    let response = send(&app, post_json("/api/auth/login", None, &json!({}))).await;
    assert!(response.status().is_client_error());
    assert!(body_json(response).await["error"].is_string());
}
