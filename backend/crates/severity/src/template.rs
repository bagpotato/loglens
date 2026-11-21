use std::sync::LazyLock;

use regex::{Captures, Regex, Replacer};
use sha2::{Digest, Sha256};

type Re = LazyLock<Option<Regex>>;

fn compile(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

static UUID: Re = LazyLock::new(|| {
    compile(r"(?i)\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b")
});
static EMAIL: Re = LazyLock::new(|| compile(r"(?i)\b[a-z0-9._%+-]+@[a-z0-9-]+(?:\.[a-z0-9-]+)+\b"));
static IPV6: Re = LazyLock::new(|| {
    let full = r"\b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b";
    let compressed =
        r"(?:\b[0-9a-f]{1,4}(?::[0-9a-f]{1,4}){0,6})?::(?:[0-9a-f]{1,4}(?::[0-9a-f]{1,4}){0,6}\b)?";
    compile(&format!(
        r"(?i)\[(?:{full}|{compressed})\](?::\d{{1,5}})?|{full}|{compressed}"
    ))
});
static IPV4: Re = LazyLock::new(|| compile(r"\b(?:\d{1,3}\.){3}\d{1,3}(?::\d{1,5})?\b"));
static HEX: Re = LazyLock::new(|| compile(r"(?i)\b0x[0-9a-f]+\b|\b[0-9a-f]{8,}\b"));
static PATH: Re = LazyLock::new(|| {
    compile(r#"(^|[\s=:"'(\[,])(?:/[\w.@+~-]+(?:/[\w.@+~-]*)*|[A-Za-z]:\\[^\s"']*)"#)
});
static NUM: Re = LazyLock::new(|| compile(r"\b\d+(?:\.\d+)?"));
static SPACES: Re = LazyLock::new(|| compile(r"\s+"));

fn sub(re: &Re, text: String, rep: impl Replacer) -> String {
    match re.as_ref() {
        Some(re) => re.replace_all(&text, rep).into_owned(),
        None => text,
    }
}

fn ipv6(c: &Captures) -> String {
    let m = &c[0];
    if m.chars().any(|ch| ch.is_ascii_digit()) {
        "<IP>".to_string()
    } else {
        m.to_string()
    }
}

fn hex(c: &Captures) -> String {
    let m = &c[0];
    let prefixed = m.starts_with("0x") || m.starts_with("0X");
    let has_digit = m.chars().any(|ch| ch.is_ascii_digit());
    let all_digits = m.chars().all(|ch| ch.is_ascii_digit());
    if prefixed || (has_digit && !all_digits) {
        "<HEX>".to_string()
    } else {
        m.to_string()
    }
}

/// Replaces variable parts of a log message with placeholders so that
/// messages produced by the same code path share one template.
pub fn normalize(message: &str) -> String {
    let s = sub(&UUID, message.to_string(), "<UUID>");
    let s = sub(&EMAIL, s, "<EMAIL>");
    let s = sub(&IPV6, s, ipv6);
    let s = sub(&IPV4, s, "<IP>");
    let s = sub(&HEX, s, hex);
    let s = sub(&PATH, s, "${1}<PATH>");
    let s = sub(&NUM, s, "<NUM>");
    sub(&SPACES, s.trim().to_string(), " ")
}

/// First 16 hex chars of sha256(normalize(message)).
pub fn template_hash(message: &str) -> String {
    hash_template(&normalize(message))
}

pub(crate) fn hash_template(template: &str) -> String {
    let digest = Sha256::digest(template.as_bytes());
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_patterns_compile() {
        for re in [&UUID, &EMAIL, &IPV6, &IPV4, &HEX, &PATH, &NUM, &SPACES] {
            assert!(re.is_some());
        }
    }

    #[test]
    fn sshd_failed_password() {
        let msg = "Failed password for root from 192.168.1.10 port 52814 ssh2";
        assert_eq!(
            normalize(msg),
            "Failed password for root from <IP> port <NUM> ssh2"
        );
    }

    #[test]
    fn ip_with_port_is_one_placeholder() {
        assert_eq!(
            normalize("connect to 10.0.0.1:8080 refused"),
            "connect to <IP> refused"
        );
        assert_eq!(normalize("peer [2001:db8::1]:443 reset"), "peer <IP> reset");
        assert_eq!(
            normalize("from fe80::1ff:fe23:4567:890a dropped"),
            "from <IP> dropped"
        );
        assert_eq!(
            normalize("from 2001:0db8:85a3:0000:0000:8a2e:0370:7334 x"),
            "from <IP> x"
        );
    }

    #[test]
    fn uuid_path_email_hex() {
        assert_eq!(
            normalize("container 3f2504e0-4f89-11d3-9a0c-0305e82c3301 started"),
            "container <UUID> started"
        );
        assert_eq!(
            normalize("cannot open /var/log/nginx/access.log: denied"),
            "cannot open <PATH>: denied"
        );
        assert_eq!(normalize("config=/etc/app.conf"), "config=<PATH>");
        assert_eq!(normalize("mail to bob@example.com"), "mail to <EMAIL>");
        assert_eq!(
            normalize("segfault at 0x7f3a ip 00007f3a1b2c sp 7ffd0a1b"),
            "segfault at <HEX> ip <HEX> sp <HEX>"
        );
        assert_eq!(normalize("commit deadbeef1"), "commit <HEX>");
        assert_eq!(normalize("word acceptable"), "word acceptable");
    }

    #[test]
    fn numbers_and_whitespace() {
        assert_eq!(
            normalize("  took   12ms,  retry 3 of 5.5 \t"),
            "took <NUM>ms, retry <NUM> of <NUM>"
        );
        assert_eq!(normalize("at 10:00:00"), "at <NUM>:<NUM>:<NUM>");
        assert_eq!(normalize("std::vec Foo::abc"), "std::vec Foo::abc");
    }

    #[test]
    fn same_template_same_hash() {
        let a = template_hash("Failed password for root from 1.2.3.4 port 22 ssh2");
        let b = template_hash("Failed password for root from 10.20.30.40 port 6000 ssh2");
        let c = template_hash("Accepted password for root from 1.2.3.4 port 22 ssh2");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|ch| ch.is_ascii_hexdigit()));
        assert_eq!(
            template_hash("job 3f2504e0-4f89-11d3-9a0c-0305e82c3301 wrote /tmp/a"),
            template_hash("job 00000000-0000-0000-0000-000000000000  wrote /srv/b/c")
        );
    }
}
