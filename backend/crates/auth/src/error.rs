use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use loglens_storage::StorageError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    /// Missing, malformed, tampered or expired token.
    #[error("unauthorized")]
    Unauthorized,
    #[error("an admin password is required to create the first user")]
    MissingAdminPassword,
    #[error("password hashing failed: {0}")]
    Hash(String),
    #[error("token issuing failed: {0}")]
    Token(String),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl AuthError {
    pub fn status(&self) -> StatusCode {
        match self {
            AuthError::InvalidCredentials | AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "auth failure");
            "internal server error".to_string()
        } else {
            self.to_string()
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_errors_are_401_others_500() {
        assert_eq!(
            AuthError::InvalidCredentials.into_response().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AuthError::Unauthorized.into_response().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AuthError::Hash("x".into()).into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
