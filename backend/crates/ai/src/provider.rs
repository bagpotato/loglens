use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AiError {
    #[error("AI provider request failed: {0}")]
    Http(String),
    #[error("AI provider returned status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("AI provider returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("log store error: {0}")]
    Store(String),
}

/// Stream of text chunks as the model produces them.
pub type TextStream = BoxStream<'static, Result<String, AiError>>;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, messages: &[ChatMessage]) -> Result<String, AiError>;
    async fn stream(&self, messages: &[ChatMessage]) -> Result<TextStream, AiError>;
}
