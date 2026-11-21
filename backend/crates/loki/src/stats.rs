use std::collections::BTreeMap;
use std::time::Duration;

use loglens_core::{Stats, StoreError};

use crate::client::LokiClient;
use crate::model::{ApiResponse, VectorData};

const INSTANT_QUERY_PATH: &str = "/loki/api/v1/query";
const ALL_STREAMS: &str = r#"{severity=~".+"}"#;
const UNKNOWN_LABEL: &str = "unknown";

pub(crate) async fn stats(client: &LokiClient, range: Duration) -> Result<Stats, StoreError> {
    let range = format_range(range)?;
    let (by_severity, by_source) = futures::try_join!(
        count_by(client, "severity", &range),
        count_by(client, "source", &range)
    )?;
    Ok(Stats {
        by_severity,
        by_source,
    })
}

async fn count_by(
    client: &LokiClient,
    label: &str,
    range: &str,
) -> Result<BTreeMap<String, u64>, StoreError> {
    let query = format!("sum by ({label}) (count_over_time({ALL_STREAMS}[{range}]))");
    let response: ApiResponse<VectorData> = client
        .get_json(INSTANT_QUERY_PATH, &[("query", query)])
        .await?;
    let samples = response.into_data()?.map(|d| d.result).unwrap_or_default();
    let mut counts = BTreeMap::new();
    for sample in samples {
        let value: f64 = sample.value.1.trim().parse().map_err(|_| {
            StoreError::InvalidResponse(format!("invalid sample value {:?}", sample.value.1))
        })?;
        let key = sample
            .metric
            .get(label)
            .filter(|v| !v.is_empty())
            .cloned()
            .unwrap_or_else(|| UNKNOWN_LABEL.to_string());
        *counts.entry(key).or_insert(0) += value.max(0.0).round() as u64;
    }
    Ok(counts)
}

/// LogQL range literal: whole hours as `24h`, whole minutes as `90m`, else seconds.
pub(crate) fn format_range(range: Duration) -> Result<String, StoreError> {
    let secs = range.as_secs();
    if secs == 0 {
        return Err(StoreError::InvalidQuery(
            "stats range must be at least 1s".into(),
        ));
    }
    Ok(if secs.is_multiple_of(3600) {
        format!("{}h", secs / 3600)
    } else if secs.is_multiple_of(60) {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ranges() {
        assert_eq!(format_range(Duration::from_secs(86_400)).unwrap(), "24h");
        assert_eq!(format_range(Duration::from_secs(5_400)).unwrap(), "90m");
        assert_eq!(format_range(Duration::from_secs(45)).unwrap(), "45s");
        assert!(format_range(Duration::ZERO).is_err());
    }
}
