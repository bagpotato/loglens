use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("invalid severity: {0}")]
    InvalidSeverity(String),
    #[error("invalid severity source: {0}")]
    InvalidSeveritySource(String),
    #[error("invalid filter: {0}")]
    InvalidFilter(String),
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("log store unavailable: {0}")]
    Unavailable(String),
    #[error("log store returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("invalid query: {0}")]
    InvalidQuery(String),
}
