use axum::{
    http::{header, HeaderValue, Method, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use loglens_auth::AuthUser;
use loglens_config::Settings;
use serde_json::json;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    routes::{ai, auth, logs, meta, stats, tail},
    AppState,
};

pub fn build_router(state: AppState) -> Router {
    build_router_with_origins(state, &Settings::cors_origins_from_env())
}

pub fn build_router_with_origins(state: AppState, cors_origins: &[String]) -> Router {
    let protected = Router::new()
        .route("/api/logs", get(logs::list))
        .route("/api/logs/tail", get(tail::tail))
        .route("/api/meta", get(meta::meta))
        .route("/api/stats", get(stats::stats))
        .route("/api/ai/explain", post(ai::explain))
        .route("/api/ai/chat", post(ai::chat))
        .route_layer(middleware::from_extractor_with_state::<AuthUser, _>(
            state.clone(),
        ));

    Router::new()
        .route("/api/health", get(meta::health))
        .route("/api/auth/login", post(auth::login))
        .merge(protected)
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer(cors_origins))
        .with_state(state)
}

fn cors_layer(origins: &[String]) -> CorsLayer {
    let allow = if origins.iter().any(|o| o == "*") {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(origins.iter().filter_map(|o| HeaderValue::from_str(o).ok()))
    };
    CorsLayer::new()
        .allow_origin(allow)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(json!({ "error": "not found" })))
}
