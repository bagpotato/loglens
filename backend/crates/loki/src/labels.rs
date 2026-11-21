use chrono::{Duration, Utc};
use loglens_core::{Meta, StoreError};

use crate::client::LokiClient;
use crate::model::ApiResponse;
use crate::query::to_nanos;

const LOOKBACK_DAYS: i64 = 7;

pub(crate) async fn meta(client: &LokiClient) -> Result<Meta, StoreError> {
    let (hosts, sources) =
        futures::try_join!(label_values(client, "host"), label_values(client, "source"))?;
    Ok(Meta { hosts, sources })
}

/// Sorted, de-duplicated values of one label. A missing label yields an empty list.
async fn label_values(client: &LokiClient, label: &str) -> Result<Vec<String>, StoreError> {
    let end = Utc::now();
    let start = end - Duration::days(LOOKBACK_DAYS);
    let params = [
        ("start", to_nanos(start)?.to_string()),
        ("end", to_nanos(end)?.to_string()),
    ];
    let path = format!("/loki/api/v1/label/{label}/values");
    let response: ApiResponse<Vec<String>> = client.get_json(&path, &params).await?;
    let mut values: Vec<String> = response
        .into_data()?
        .unwrap_or_default()
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect();
    values.sort();
    values.dedup();
    Ok(values)
}
