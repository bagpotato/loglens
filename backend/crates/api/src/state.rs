use std::sync::Arc;

use axum::extract::FromRef;
use loglens_ai::AiProvider;
use loglens_auth::JwtKeys;
use loglens_core::LogStore;
use loglens_severity::Classifier;
use loglens_storage::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt: JwtKeys,
    pub store: Arc<dyn LogStore>,
    pub ai: Arc<dyn AiProvider>,
    pub classifier: Arc<Classifier>,
    pub ai_mask: bool,
}

impl FromRef<AppState> for JwtKeys {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}
