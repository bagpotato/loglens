use axum::{
    extract::{rejection::JsonRejection, State},
    Json,
};
use loglens_auth::LoginResponse;
use serde::Deserialize;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    body: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<LoginResponse>, ApiError> {
    let Json(req) = body?;
    let response =
        loglens_auth::login(&state.pool, &state.jwt, req.username.trim(), &req.password).await?;
    Ok(Json(response))
}
