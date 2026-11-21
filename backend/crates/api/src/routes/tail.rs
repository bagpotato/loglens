use axum::{
    extract::{
        rejection::QueryRejection,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::StreamExt;
use loglens_core::{LogEntry, LogStream};
use loglens_severity::Classifier;

use crate::{ApiError, AppState, LogQuery};

pub async fn tail(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    query: Result<Query<LogQuery>, QueryRejection>,
) -> Result<Response, ApiError> {
    let Query(query) = query?;
    let filters = query.into_filters()?;
    let stream = state.store.tail(&filters).await?;
    Ok(ws
        .protocols(["bearer"])
        .on_upgrade(move |socket| forward(socket, stream, state)))
}

async fn forward(mut socket: WebSocket, mut stream: LogStream, state: AppState) {
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return,
                Some(Ok(_)) => {}
            },
            next = stream.next() => match next {
                Some(Ok(entry)) => {
                    let Some(text) = enrich_to_json(&state.classifier, entry).await else {
                        continue;
                    };
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        return;
                    }
                }
                Some(Err(err)) => {
                    tracing::warn!(error = %err, "log tail failed");
                    break;
                }
                None => break,
            },
        }
    }
    let _ = socket.send(Message::Close(None)).await;
}

async fn enrich_to_json(classifier: &Classifier, entry: LogEntry) -> Option<String> {
    let mut batch = [entry];
    classifier.enrich_cached(&mut batch).await;
    match serde_json::to_string(&batch[0]) {
        Ok(text) => Some(text),
        Err(err) => {
            tracing::warn!(error = %err, "failed to serialize log entry");
            None
        }
    }
}
