use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlertError {
    #[error("notification request failed: {0}")]
    Http(String),
    #[error("notification rejected with status {status}: {body}")]
    Status { status: u16, body: String },
}
