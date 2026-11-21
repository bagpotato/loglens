use std::time::Duration;

use loglens_core::StoreError;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Thin HTTP wrapper around the Loki API that maps failures to `StoreError`.
#[derive(Debug, Clone)]
pub(crate) struct LokiClient {
    http: reqwest::Client,
    base_url: String,
}

impl LokiClient {
    pub(crate) fn new(base_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T, StoreError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| StoreError::Unavailable(format!("GET {path}: {e}")))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(status_error(status, path, body.trim()));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| StoreError::InvalidResponse(format!("GET {path}: {e}")))
    }
}

fn status_error(status: StatusCode, path: &str, body: &str) -> StoreError {
    let msg = format!("GET {path} returned {status}: {body}");
    if status == StatusCode::BAD_REQUEST {
        StoreError::InvalidQuery(msg)
    } else if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS {
        StoreError::Unavailable(msg)
    } else {
        StoreError::InvalidResponse(msg)
    }
}
