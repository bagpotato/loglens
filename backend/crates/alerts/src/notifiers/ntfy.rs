use std::time::Duration;

use async_trait::async_trait;
use chrono::SecondsFormat;
use loglens_config::NtfySettings;

use crate::{Alert, AlertError, Notifier};

const MAX_BODY_CHARS: usize = 1000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Sends alerts as urgent push notifications through an ntfy server.
pub struct NtfyNotifier {
    client: reqwest::Client,
    endpoint: String,
}

impl NtfyNotifier {
    pub fn new(settings: &NtfySettings) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: format!(
                "{}/{}",
                settings.url.trim_end_matches('/'),
                settings.topic.trim_start_matches('/')
            ),
        }
    }
}

#[async_trait]
impl Notifier for NtfyNotifier {
    async fn notify(&self, alert: &Alert) -> Result<(), AlertError> {
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(REQUEST_TIMEOUT)
            .header("Title", header_safe(&alert.title))
            .header("Priority", "urgent")
            .header("Tags", "rotating_light")
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(body(alert))
            .send()
            .await
            .map_err(|e| AlertError::Http(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body = response.text().await.unwrap_or_default();
        Err(AlertError::Status {
            status: status.as_u16(),
            body,
        })
    }
}

fn body(alert: &Alert) -> String {
    let text = format!(
        "{} {}",
        alert.timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
        alert.message
    );
    text.chars().take(MAX_BODY_CHARS).collect()
}

/// HTTP header values must be visible ASCII; anything else becomes `?`.
fn header_safe(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c == ' ' || c.is_ascii_graphic() {
                c
            } else {
                '?'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use wiremock::matchers::{body_string, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn alert(message: &str) -> Alert {
        Alert {
            title: "LogLens critical on web1".into(),
            message: message.into(),
            host: "web1".into(),
            timestamp: Utc.with_ymd_and_hms(2026, 11, 21, 10, 0, 0).unwrap(),
        }
    }

    fn notifier(server: &MockServer) -> NtfyNotifier {
        NtfyNotifier::new(&NtfySettings {
            url: server.uri(),
            topic: "loglens-alerts".into(),
        })
    }

    #[tokio::test]
    async fn posts_to_topic_with_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/loglens-alerts"))
            .and(header("Title", "LogLens critical on web1"))
            .and(header("Priority", "urgent"))
            .and(header("Tags", "rotating_light"))
            .and(body_string("2026-11-21T10:00:00Z kernel panic"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        notifier(&server)
            .notify(&alert("kernel panic"))
            .await
            .unwrap();
        server.verify().await;
    }

    #[tokio::test]
    async fn non_success_status_is_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
            .mount(&server)
            .await;

        let err = notifier(&server).notify(&alert("x")).await.unwrap_err();
        assert!(matches!(err, AlertError::Status { status: 403, .. }));
    }

    #[test]
    fn body_is_truncated() {
        let long = "é".repeat(5000);
        assert_eq!(body(&alert(&long)).chars().count(), MAX_BODY_CHARS);
    }

    #[test]
    fn header_strips_non_ascii() {
        assert_eq!(header_safe("on wéb\n1"), "on w?b?1");
    }
}
