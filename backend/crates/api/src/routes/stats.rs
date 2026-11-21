use axum::{
    extract::{rejection::QueryRejection, Query, State},
    Json,
};
use loglens_core::Stats;

use crate::{parse_range, ApiError, AppState, StatsQuery};

pub async fn stats(
    State(state): State<AppState>,
    query: Result<Query<StatsQuery>, QueryRejection>,
) -> Result<Json<Stats>, ApiError> {
    let Query(query) = query?;
    let range = parse_range(query.range.as_deref())?;
    Ok(Json(state.store.stats(range).await?))
}
