use std::collections::HashMap;

use loglens_core::StoreError;
use serde::Deserialize;

/// Envelope of every Loki HTTP API response.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    #[serde(default)]
    pub status: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Data of a successful response; `None` when Loki returned no data.
    pub(crate) fn into_data(self) -> Result<Option<T>, StoreError> {
        if self.status.is_empty() || self.status == "success" {
            Ok(self.data)
        } else {
            Err(StoreError::InvalidResponse(format!(
                "loki status {}",
                self.status
            )))
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StreamsData {
    #[serde(default)]
    pub result: Vec<StreamResult>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StreamResult {
    #[serde(default)]
    pub stream: HashMap<String, String>,
    #[serde(default)]
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VectorData {
    #[serde(default)]
    pub result: Vec<VectorSample>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct VectorSample {
    #[serde(default)]
    pub metric: HashMap<String, String>,
    pub value: (serde_json::Value, String),
}

/// Frame pushed by `/loki/api/v1/tail`.
#[derive(Debug, Deserialize)]
pub(crate) struct TailFrame {
    #[serde(default)]
    pub streams: Vec<StreamResult>,
}
