use futures::{SinkExt, StreamExt};
use loglens_core::{LogFilters, LogStore, Severity};
use loglens_loki::LokiStore;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;

const FRAME: &str = r#"{"streams":[{"stream":{"host":"web1","source":"kernel","severity":"critical"},"values":[["1763719200000000000","{\"message\":\"kernel panic test\",\"severity_source\":\"rule\"}"]]}]}"#;

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn tail_yields_pushed_entries_and_ends_on_close() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (uri_tx, uri_rx) = oneshot::channel::<String>();

    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let callback = move |req: &Request, resp: Response| {
            let _ = uri_tx.send(req.uri().to_string());
            Ok(resp)
        };
        let mut ws = tokio_tungstenite::accept_hdr_async(tcp, callback)
            .await
            .unwrap();
        ws.send(Message::text(FRAME)).await.unwrap();
        ws.close(None).await.unwrap();
        while ws.next().await.is_some() {}
    });

    let store = LokiStore::new(&format!("http://{addr}"));
    let filters = LogFilters {
        severity: vec![Severity::Critical],
        ..Default::default()
    };
    let stream = store.tail(&filters).await.unwrap();
    let items: Vec<_> = tokio::time::timeout(std::time::Duration::from_secs(10), stream.collect())
        .await
        .unwrap();

    assert_eq!(items.len(), 1);
    let entry = items[0].as_ref().unwrap();
    assert_eq!(entry.message, "kernel panic test");
    assert_eq!(entry.severity, Severity::Critical);
    assert_eq!(entry.host, "web1");

    let uri = uri_rx.await.unwrap();
    assert!(uri.starts_with("/loki/api/v1/tail?query="), "{uri}");
    assert!(uri.contains("severity%3D%22critical%22"), "{uri}");
    assert!(uri.contains("&start="), "{uri}");
    server.await.unwrap();
}
