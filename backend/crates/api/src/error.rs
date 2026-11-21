use axum::{
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use loglens_ai::AiError;
use loglens_auth::AuthError;
use loglens_core::StoreError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{message}")]
    Rejected { status: StatusCode, message: String },
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Ai(#[from] AiError),
    #[error(transparent)]
    Auth(#[from] AuthError),
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::Store(StoreError::InvalidQuery(_)) => {
                StatusCode::BAD_REQUEST
            }
            Self::Rejected { status, .. } => *status,
            Self::Store(_) | Self::Ai(_) => StatusCode::BAD_GATEWAY,
            Self::Auth(err) => err.status(),
        }
    }
}

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        Self::BadRequest(rejection.body_text())
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Self::Auth(err) = self {
            return err.into_response();
        }
        let status = self.status();
        if status.is_server_error() {
            tracing::warn!(error = %self, "request failed");
        }
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
