use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::LazyLock;

use regex::Regex;

type Pattern = LazyLock<Option<Regex>>;

static IPV4: Pattern = LazyLock::new(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").ok());
static IPV6: Pattern = LazyLock::new(|| {
    Regex::new(
        r"(?i)[0-9a-f]{0,4}(?::[0-9a-f]{0,4}){1,6}:(?:\d{1,3}\.){3}\d{1,3}|[0-9a-f]{0,4}(?::[0-9a-f]{0,4}){2,7}",
    )
    .ok()
});
static USERS: Pattern = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:r?user=|user\s+)([a-z0-9._@][a-z0-9._@-]*)|\bfor\s+([a-z0-9._@][a-z0-9._@-]*)\s+from\b",
    )
    .ok()
});
static PLACEHOLDER: Pattern = LazyLock::new(|| Regex::new(r"<(?:IP|USER)_\d+>").ok());

/// Replaces every IPv6 and then IPv4 address with `f(address)`.
pub(super) fn replace_ips(text: &str, mut f: impl FnMut(&str) -> String) -> String {
    let text = replace_valid(text, &IPV6, is_ipv6, &mut f);
    replace_valid(&text, &IPV4, |s| s.parse::<Ipv4Addr>().is_ok(), &mut f)
}

/// Replaces usernames found in typical auth-log phrases with `f(username)`.
pub(super) fn replace_users(text: &str, mut f: impl FnMut(&str) -> String) -> String {
    let Some(re) = USERS.as_ref() else {
        return text.to_string();
    };
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for caps in re.captures_iter(text) {
        let Some(name) = caps.get(1).or_else(|| caps.get(2)) else {
            continue;
        };
        out.push_str(&text[last..name.start()]);
        out.push_str(&f(name.as_str()));
        last = name.end();
    }
    out.push_str(&text[last..]);
    out
}

/// Replaces placeholders for which `lookup` knows the original value.
pub(super) fn replace_placeholders(text: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let Some(re) = PLACEHOLDER.as_ref() else {
        return text.to_string();
    };
    re.replace_all(text, |caps: &regex::Captures<'_>| {
        let found = &caps[0];
        lookup(found).unwrap_or_else(|| found.to_string())
    })
    .into_owned()
}

fn is_ipv6(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_hexdigit()) && s.parse::<Ipv6Addr>().is_ok()
}

fn replace_valid(
    text: &str,
    pattern: &Pattern,
    valid: impl Fn(&str) -> bool,
    f: &mut impl FnMut(&str) -> String,
) -> String {
    let Some(re) = pattern.as_ref() else {
        return text.to_string();
    };
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text) {
        if !standalone(text, m.start(), m.end()) || !valid(m.as_str()) {
            continue;
        }
        out.push_str(&text[last..m.start()]);
        out.push_str(&f(m.as_str()));
        last = m.end();
    }
    out.push_str(&text[last..]);
    out
}

/// True when the match is not glued to a word character (e.g. `std::io`, `Foo::bar`).
fn standalone(text: &str, start: usize, end: usize) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    !text[..start].chars().next_back().is_some_and(word)
        && !text[end..].chars().next().is_some_and(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_patterns_compile() {
        for p in [&IPV4, &IPV6, &USERS, &PLACEHOLDER] {
            assert!(p.as_ref().is_some());
        }
    }

    #[test]
    fn ignores_times_macs_and_paths() {
        let text = "10:00:00 aa:bb:cc:dd:ee:ff std::io Foo::bar 999.1.1.1";
        assert_eq!(replace_ips(text, |_| "X".into()), text);
    }

    #[test]
    fn finds_ipv6_forms() {
        let text = "a ::1 b fe80::1%eth0 c 2001:db8::ff00:42:8329 d ::ffff:10.0.0.1";
        let out = replace_ips(text, |_| "X".into());
        assert_eq!(out, "a X b X%eth0 c X d X");
    }
}
