use axum::{extract::State, Json};
use loglens_core::Meta;
use serde_json::{json, Value};

use crate::{ApiError, AppState};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn meta(State(state): State<AppState>) -> Result<Json<Meta>, ApiError> {
    Ok(Json(state.store.meta().await?))
}
