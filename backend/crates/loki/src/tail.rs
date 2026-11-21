use chrono::Utc;
use futures::{stream, StreamExt};
use loglens_core::{Direction, LogEntry, LogFilters, LogStream, StoreError};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::logql::build_query;
use crate::model::TailFrame;
use crate::query::{flatten_streams, sort_entries, to_nanos};

const TAIL_PATH: &str = "/loki/api/v1/tail";

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) async fn tail(base_url: &str, filters: &LogFilters) -> Result<LogStream, StoreError> {
    let url = tail_url(base_url, filters)?;
    let (ws, _) = connect_async(url.as_str())
        .await
        .map_err(|e| StoreError::Unavailable(format!("loki tail connect: {e}")))?;
    Ok(into_entries(ws))
}

/// `http(s)://host/x` -> `ws(s)://host/x/loki/api/v1/tail?query=...&start=<ns>`.
pub(crate) fn tail_url(base_url: &str, filters: &LogFilters) -> Result<String, StoreError> {
    let base = base_url.trim_end_matches('/');
    let ws_base = if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if base.starts_with("ws://") || base.starts_with("wss://") {
        base.to_string()
    } else {
        return Err(StoreError::InvalidQuery(format!(
            "unsupported loki url scheme: {base}"
        )));
    };
    let start = to_nanos(filters.start.unwrap_or_else(Utc::now))?;
    Ok(format!(
        "{ws_base}{TAIL_PATH}?query={}&start={start}",
        urlencoding::encode(&build_query(filters))
    ))
}

/// Ends on close frame, end of stream, or after the first transport error.
fn into_entries(ws: WsStream) -> LogStream {
    stream::unfold(Some(ws), |state| async move {
        let mut ws = state?;
        loop {
            match ws.next().await {
                None | Some(Ok(Message::Close(_))) => return None,
                Some(Ok(Message::Text(text))) => {
                    return Some((decode_frame(text.as_str()), Some(ws)))
                }
                Some(Ok(Message::Binary(bytes))) => {
                    let items = match std::str::from_utf8(&bytes) {
                        Ok(text) => decode_frame(text),
                        Err(e) => vec![Err(StoreError::InvalidResponse(e.to_string()))],
                    };
                    return Some((items, Some(ws)));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => {
                    let err = StoreError::Unavailable(format!("loki tail: {e}"));
                    return Some((vec![Err(err)], None));
                }
            }
        }
    })
    .flat_map(stream::iter)
    .boxed()
}

fn decode_frame(text: &str) -> Vec<Result<LogEntry, StoreError>> {
    match serde_json::from_str::<TailFrame>(text) {
        Ok(frame) => {
            let mut entries = flatten_streams(frame.streams);
            sort_entries(&mut entries, Direction::Forward);
            entries.into_iter().map(Ok).collect()
        }
        Err(e) => vec![Err(StoreError::InvalidResponse(format!(
            "invalid tail frame: {e}"
        )))],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_websocket_url() {
        let f = LogFilters::default();
        let url = tail_url("http://loki:3100/", &f).unwrap();
        assert!(url.starts_with(
            "ws://loki:3100/loki/api/v1/tail?query=%7Bseverity%3D~%22.%2B%22%7D&start="
        ));
        let secure = tail_url("https://loki.example", &f).unwrap();
        assert!(secure.starts_with("wss://loki.example/loki/api/v1/tail?"));
        assert!(tail_url("ftp://x", &f).is_err());
    }

    #[test]
    fn decodes_push_frame() {
        let frame = r#"{"streams":[{"stream":{"host":"a","severity":"critical"},"values":[["2","b"],["1","a"]]}]}"#;
        let items = decode_frame(frame);
        assert_eq!(items.len(), 2);
        let first = items[0].as_ref().unwrap();
        assert_eq!(first.message, "a");
        assert!(decode_frame("nope")[0].is_err());
    }
}
