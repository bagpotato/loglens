use axum::{
    extract::{rejection::QueryRejection, Query, State},
    Json,
};
use loglens_core::LogEntry;
use serde::Serialize;

use crate::{ApiError, AppState, LogQuery};

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub entries: Vec<LogEntry>,
}

pub async fn list(
    State(state): State<AppState>,
    query: Result<Query<LogQuery>, QueryRejection>,
) -> Result<Json<LogsResponse>, ApiError> {
    let Query(query) = query?;
    let filters = query.into_filters()?;
    let entries =
        loglens_severity::query_classified(state.store.as_ref(), &state.classifier, &filters)
            .await?;
    Ok(Json(LogsResponse { entries }))
}
