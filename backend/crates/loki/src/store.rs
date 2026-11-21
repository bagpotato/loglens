use std::time::Duration;

use async_trait::async_trait;
use loglens_core::{LogEntry, LogFilters, LogStore, LogStream, Meta, Stats, StoreError};

use crate::client::LokiClient;
use crate::{labels, query, stats, tail};

/// `LogStore` backed by a Loki server (`LOGLENS_LOKI_URL`).
#[derive(Debug, Clone)]
pub struct LokiStore {
    client: LokiClient,
}

impl LokiStore {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: LokiClient::new(base_url),
        }
    }
}

#[async_trait]
impl LogStore for LokiStore {
    async fn query(&self, filters: &LogFilters) -> Result<Vec<LogEntry>, StoreError> {
        query::query_range(&self.client, filters).await
    }

    async fn meta(&self) -> Result<Meta, StoreError> {
        labels::meta(&self.client).await
    }

    async fn stats(&self, range: Duration) -> Result<Stats, StoreError> {
        stats::stats(&self.client, range).await
    }

    async fn tail(&self, filters: &LogFilters) -> Result<LogStream, StoreError> {
        tail::tail(self.client.base_url(), filters).await
    }
}
