use chrono::{DateTime, Duration, Utc};
use loglens_core::{Direction, LogEntry, LogFilters, StoreError};

use crate::client::LokiClient;
use crate::logql::build_query;
use crate::model::{ApiResponse, StreamResult, StreamsData};
use crate::parse::parse_entry;

const QUERY_RANGE_PATH: &str = "/loki/api/v1/query_range";

pub(crate) async fn query_range(
    client: &LokiClient,
    filters: &LogFilters,
) -> Result<Vec<LogEntry>, StoreError> {
    let end = filters.end.unwrap_or_else(Utc::now);
    let start = filters.start.unwrap_or(end - Duration::hours(24));
    if start > end {
        return Err(StoreError::InvalidQuery("start is after end".into()));
    }
    let limit = filters.effective_limit();
    let params = [
        ("query", build_query(filters)),
        ("start", to_nanos(start)?.to_string()),
        ("end", to_nanos(end)?.to_string()),
        ("limit", limit.to_string()),
        ("direction", direction_param(filters.direction).to_string()),
    ];
    let response: ApiResponse<StreamsData> = client.get_json(QUERY_RANGE_PATH, &params).await?;
    let streams = response.into_data()?.map(|d| d.result).unwrap_or_default();

    let mut entries = flatten_streams(streams);
    sort_entries(&mut entries, filters.direction);
    entries.truncate(limit as usize);
    Ok(entries)
}

/// Converts every value of every stream; unparseable values are skipped.
pub(crate) fn flatten_streams(streams: Vec<StreamResult>) -> Vec<LogEntry> {
    streams
        .iter()
        .flat_map(|s| {
            s.values
                .iter()
                .filter_map(|(ts, line)| parse_entry(&s.stream, ts, line))
        })
        .collect()
}

pub(crate) fn sort_entries(entries: &mut [LogEntry], direction: Direction) {
    match direction {
        Direction::Forward => {
            entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp).then(a.id.cmp(&b.id)))
        }
        Direction::Backward => {
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then(b.id.cmp(&a.id)))
        }
    }
}

pub(crate) fn to_nanos(time: DateTime<Utc>) -> Result<i64, StoreError> {
    time.timestamp_nanos_opt()
        .ok_or_else(|| StoreError::InvalidQuery(format!("timestamp out of range: {time}")))
}

fn direction_param(direction: Direction) -> &'static str {
    match direction {
        Direction::Forward => "forward",
        Direction::Backward => "backward",
    }
}
