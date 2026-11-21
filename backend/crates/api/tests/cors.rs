mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use common::{get, send, setup};

fn preflight(origin: &str) -> Request<Body> {
    Request::options("/api/meta")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .header(
            header::ACCESS_CONTROL_REQUEST_HEADERS,
            "authorization,content-type",
        )
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn preflight_from_native_app_is_allowed() {
    let app = setup(Vec::new()).await;
    let response = send(&app, preflight("http://tauri.localhost")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(
        headers[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://tauri.localhost"
    );
    let allowed_headers = headers[header::ACCESS_CONTROL_ALLOW_HEADERS]
        .to_str()
        .unwrap()
        .to_ascii_lowercase();
    assert!(allowed_headers.contains("authorization"));
    assert!(allowed_headers.contains("content-type"));
    let methods = headers[header::ACCESS_CONTROL_ALLOW_METHODS]
        .to_str()
        .unwrap();
    assert!(methods.contains("GET") && methods.contains("POST"));
}

#[tokio::test]
async fn preflight_from_unknown_origin_gets_no_allow_origin() {
    let app = setup(Vec::new()).await;
    let response = send(&app, preflight("http://evil.example")).await;
    assert!(response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());
}

#[tokio::test]
async fn simple_request_from_native_app_carries_allow_origin() {
    let app = setup(Vec::new()).await;
    let mut request = get("/api/health", None);
    request.headers_mut().insert(
        header::ORIGIN,
        header::HeaderValue::from_static("https://tauri.localhost"),
    );
    let response = send(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://tauri.localhost"
    );
}
