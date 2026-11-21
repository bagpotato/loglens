mod common;

use common::{entry, login, setup};
use futures::StreamExt;
use loglens_core::{LogEntry, Severity, SeveritySource};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

#[tokio::test]
async fn tail_streams_entries_over_websocket() {
    let app = setup(vec![entry(
        "1-a",
        "kernel panic",
        Severity::Critical,
        SeveritySource::Rule,
    )])
    .await;
    let token = login(&app).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let router = app.router.clone();
    tokio::spawn(async move { axum::serve(listener, router).await });

    let url = format!("ws://{addr}/api/logs/tail?severity=critical");
    let unauthenticated = tokio_tungstenite::connect_async(url.as_str()).await;
    assert!(unauthenticated.is_err());

    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        format!("bearer, {token}").parse().unwrap(),
    );
    let (mut ws, response) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(response.headers()["sec-websocket-protocol"], "bearer");

    let Some(Ok(Message::Text(text))) = ws.next().await else {
        panic!("expected a text frame");
    };
    let received: LogEntry = serde_json::from_str(text.as_str()).unwrap();
    assert_eq!(received.id, "1-a");
    assert_eq!(received.severity, Severity::Critical);

    let closing = ws.next().await;
    assert!(matches!(closing, Some(Ok(Message::Close(_))) | None));
}
