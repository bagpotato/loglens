use loglens_core::LogFilters;

/// Stream selector for the filters. Always contains a non-empty matcher.
pub fn build_selector(filters: &LogFilters) -> String {
    let mut matchers = Vec::new();
    if let Some(host) = non_empty(&filters.host) {
        matchers.push(format!("host=\"{}\"", escape_string(host)));
    }
    if let Some(source) = non_empty(&filters.source) {
        matchers.push(format!("source=\"{}\"", escape_string(source)));
    }
    match filters.severity.as_slice() {
        [] => matchers.push("severity=~\".+\"".to_string()),
        [one] => matchers.push(format!("severity=\"{}\"", one.as_str())),
        many => {
            let alternatives: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
            matchers.push(format!("severity=~\"{}\"", alternatives.join("|")));
        }
    }
    format!("{{{}}}", matchers.join(", "))
}

/// Full LogQL query: selector plus an optional case-insensitive line filter.
pub fn build_query(filters: &LogFilters) -> String {
    let selector = build_selector(filters);
    match non_empty(&filters.q) {
        Some(q) => format!("{selector} |~ \"(?i){}\"", escape_string(&regex::escape(q))),
        None => selector,
    }
}

/// Escapes a value for use inside a double-quoted LogQL string.
pub(crate) fn escape_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use loglens_core::Severity;

    use super::*;

    #[test]
    fn no_filters_matches_everything() {
        let f = LogFilters::default();
        assert_eq!(build_query(&f), r#"{severity=~".+"}"#);
    }

    #[test]
    fn single_severity_uses_equality() {
        let f = LogFilters {
            severity: vec![Severity::Warning],
            ..Default::default()
        };
        assert_eq!(build_selector(&f), r#"{severity="warning"}"#);
    }

    #[test]
    fn multiple_severities_use_regex() {
        let f = LogFilters {
            severity: vec![Severity::Critical, Severity::Important],
            ..Default::default()
        };
        assert_eq!(build_query(&f), r#"{severity=~"critical|important"}"#);
    }

    #[test]
    fn host_and_source() {
        let f = LogFilters {
            host: Some("web1".into()),
            source: Some("ssh".into()),
            ..Default::default()
        };
        assert_eq!(
            build_query(&f),
            r#"{host="web1", source="ssh", severity=~".+"}"#
        );
    }

    #[test]
    fn label_values_are_escaped() {
        let f = LogFilters {
            host: Some(r#"we"b\1"#.into()),
            ..Default::default()
        };
        assert_eq!(build_selector(&f), r#"{host="we\"b\\1", severity=~".+"}"#);
    }

    #[test]
    fn search_is_regex_and_string_escaped() {
        let f = LogFilters {
            q: Some(r#"say "hi" (a.b*)"#.into()),
            ..Default::default()
        };
        assert_eq!(
            build_query(&f),
            r#"{severity=~".+"} |~ "(?i)say \"hi\" \\(a\\.b\\*\\)""#
        );
    }

    #[test]
    fn blank_filters_are_ignored() {
        let f = LogFilters {
            host: Some("  ".into()),
            q: Some(String::new()),
            ..Default::default()
        };
        assert_eq!(build_query(&f), r#"{severity=~".+"}"#);
    }
}
