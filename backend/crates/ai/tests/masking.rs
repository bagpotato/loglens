use futures::{stream, StreamExt};
use loglens_ai::{unmask_stream, AiError, Masker, TextStream};

fn chunks(parts: &[&str]) -> TextStream {
    let items: Vec<Result<String, AiError>> = parts.iter().map(|p| Ok(p.to_string())).collect();
    stream::iter(items).boxed()
}

async fn collect(stream: TextStream) -> Vec<String> {
    stream.map(|c| c.unwrap()).collect().await
}

#[test]
fn masks_ips_and_users_with_stable_placeholders() {
    let mut m = Masker::new(true);
    let a = m.mask("Failed password for invalid user admin from 203.0.113.9 port 22 ssh2");
    assert_eq!(
        a,
        "Failed password for invalid user <USER_1> from <IP_1> port 22 ssh2"
    );
    let b = m.mask("Accepted publickey for alice from 203.0.113.9 port 50000");
    assert_eq!(b, "Accepted publickey for <USER_2> from <IP_1> port 50000");
    let c = m.mask("Invalid user admin from 198.51.100.7");
    assert_eq!(c, "Invalid user <USER_1> from <IP_2>");
}

#[test]
fn masks_key_value_users_and_ipv6() {
    let mut m = Masker::new(true);
    let line = "pam_unix(sudo:auth): logname= uid=1000 ruser=bob rhost=2001:db8::1 user=bob";
    assert_eq!(
        m.mask(line),
        "pam_unix(sudo:auth): logname= uid=1000 ruser=<USER_1> rhost=<IP_1> user=<USER_1>"
    );
    let sudo = "bob : TTY=pts/0 ; PWD=/home ; USER=root ; COMMAND=/bin/ls";
    assert_eq!(
        m.mask(sudo),
        "bob : TTY=pts/0 ; PWD=/home ; USER=<USER_2> ; COMMAND=/bin/ls"
    );
    assert_eq!(
        m.mask("listening on [::1]:8080"),
        "listening on [<IP_2>]:8080"
    );
    assert_eq!(
        m.mask("session opened for user root(uid=0) by (uid=0)"),
        "session opened for user <USER_2>(uid=0) by (uid=0)"
    );
}

#[test]
fn leaves_non_sensitive_text_alone() {
    let mut m = Masker::new(true);
    let line =
        "[2026-11-21T10:00:00Z] kernel: panic at std::io::Error mac aa:bb:cc:dd:ee:ff v1.2.3";
    assert_eq!(m.mask(line), line);
    assert!(!m.has_placeholders());
}

#[test]
fn disabled_masker_is_identity() {
    let mut m = Masker::new(false);
    let line = "Failed password for root from 10.0.0.1";
    assert_eq!(m.mask(line), line);
    assert_eq!(m.unmask("<IP_1>"), "<IP_1>");
    assert!(!m.is_enabled());
}

#[test]
fn unmask_restores_known_placeholders_only() {
    let mut m = Masker::new(true);
    m.mask("Failed password for root from 10.0.0.1");
    assert_eq!(
        m.unmask("<USER_1> tried from <IP_1>; <IP_9> is unknown"),
        "root tried from 10.0.0.1; <IP_9> is unknown"
    );
}

#[tokio::test]
async fn unmask_stream_handles_split_placeholders() {
    let mut m = Masker::new(true);
    m.mask("Failed password for root from 10.0.0.1");
    m.mask("from 10.0.0.2 and 10.0.0.3 and 10.0.0.4 and 10.0.0.5 and 10.0.0.6");
    m.mask("from 10.0.0.7 and 10.0.0.8 and 10.0.0.9 and 10.0.0.10 and 10.0.0.11");
    let parts = [
        "User <",
        "USER_",
        "1> came from <IP",
        "_1",
        "1> and <IP_1",
        ">",
        " a<b <IP_2",
        "> <US",
    ];
    let out = collect(unmask_stream(m, chunks(&parts))).await;
    assert_eq!(
        out.concat(),
        "User root came from 10.0.0.11 and 10.0.0.1 a<b 10.0.0.2 <US"
    );
    assert!(out.iter().all(|c| !c.is_empty()));
}

#[tokio::test]
async fn unmask_stream_passes_errors_and_through_when_nothing_masked() {
    let mut m = Masker::new(true);
    m.mask("from 10.0.0.1");
    let items: Vec<Result<String, AiError>> = vec![
        Ok("<IP".into()),
        Err(AiError::Http("down".into())),
        Ok("_1>!".into()),
    ];
    let out: Vec<_> = unmask_stream(m, stream::iter(items).boxed())
        .collect()
        .await;
    assert_eq!(out.len(), 2);
    assert!(matches!(out[0], Err(AiError::Http(_))));
    assert_eq!(out[1].as_ref().unwrap(), "10.0.0.1!");

    let plain = collect(unmask_stream(Masker::new(true), chunks(&["<IP_1>", " x"]))).await;
    assert_eq!(plain, vec!["<IP_1>", " x"]);
}
