use std::time::Duration;

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{LogEntry, LogFilters, Meta, Stats, StoreError};

pub type LogStream = BoxStream<'static, Result<LogEntry, StoreError>>;

/// Read access to stored logs. Loki is the MVP implementation.
#[async_trait]
pub trait LogStore: Send + Sync {
    async fn query(&self, filters: &LogFilters) -> Result<Vec<LogEntry>, StoreError>;
    async fn meta(&self) -> Result<Meta, StoreError>;
    /// Counts over the last `range`.
    async fn stats(&self, range: Duration) -> Result<Stats, StoreError>;
    async fn tail(&self, filters: &LogFilters) -> Result<LogStream, StoreError>;
}
