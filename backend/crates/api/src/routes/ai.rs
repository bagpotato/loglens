use std::convert::Infallible;

use axum::{
    extract::{rejection::JsonRejection, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::{stream::BoxStream, Stream, StreamExt};
use loglens_ai::{ChatEvent, ChatMessage, Role};
use loglens_core::LogEntry;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{ApiError, AppState};

#[derive(Debug, Deserialize)]
pub struct ExplainRequest {
    pub entry: LogEntry,
}

#[derive(Debug, Serialize)]
pub struct ExplainResponse {
    pub explanation: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub question: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
}

pub async fn explain(
    State(state): State<AppState>,
    body: Result<Json<ExplainRequest>, JsonRejection>,
) -> Result<Json<ExplainResponse>, ApiError> {
    let Json(req) = body?;
    let explanation = loglens_ai::explain(
        state.ai.as_ref(),
        state.store.as_ref(),
        &req.entry,
        state.ai_mask,
    )
    .await?;
    Ok(Json(ExplainResponse { explanation }))
}

pub async fn chat(
    State(state): State<AppState>,
    body: Result<Json<ChatRequest>, JsonRejection>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let Json(req) = body?;
    let question = req.question.trim().to_string();
    if question.is_empty() {
        return Err(ApiError::bad_request("question must not be empty"));
    }
    let history = req
        .history
        .into_iter()
        .filter(|m| m.role != Role::System)
        .collect();
    let started = loglens_ai::chat(
        state.ai.clone(),
        state.store.clone(),
        question,
        history,
        state.ai_mask,
    )
    .await;
    let events: BoxStream<'static, ChatEvent> = match started {
        Ok(events) => events,
        Err(err) => {
            tracing::warn!(error = %err, "chat failed to start");
            futures::stream::iter([ChatEvent::Error {
                message: err.to_string(),
            }])
            .boxed()
        }
    };
    let stream = events.map(|event| Ok(to_sse(&event)));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn to_sse(event: &ChatEvent) -> Event {
    Event::default().json_data(event).unwrap_or_else(|err| {
        let fallback = json!({ "type": "error", "message": err.to_string() });
        Event::default().data(fallback.to_string())
    })
}
