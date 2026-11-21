use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
    #[error("invalid value for {name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProviderKind {
    Ollama,
    OpenAi,
}

impl FromStr for AiProviderKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ollama" => Ok(Self::Ollama),
            "openai" => Ok(Self::OpenAi),
            other => Err(format!(
                "unknown provider '{other}', expected ollama or openai"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSettings {
    pub provider: AiProviderKind,
    /// Base URL without trailing slash.
    pub url: String,
    pub model: String,
    pub key: Option<String>,
    pub mask: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtfySettings {
    /// Base URL without trailing slash.
    pub url: String,
    pub topic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub bind: String,
    /// Base URL without trailing slash.
    pub loki_url: String,
    pub db_url: String,
    pub admin_user: String,
    pub admin_password: Option<String>,
    pub jwt_secret: String,
    pub ai: AiSettings,
    /// None disables alerts.
    pub ntfy: Option<NtfySettings>,
    /// Browser origins allowed by CORS (the native app's WebView).
    pub cors_origins: Vec<String>,
}

impl Settings {
    pub const DEFAULT_CORS_ORIGINS: &'static str =
        "http://tauri.localhost,https://tauri.localhost,tauri://localhost";

    pub fn cors_origins_from_env() -> Vec<String> {
        parse_origins(std::env::var("LOGLENS_CORS_ORIGINS").ok().as_deref())
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Builds settings from any variable source; used by tests.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let opt = |name: &str| {
            get(name)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        let or = |name: &str, default: &str| opt(name).unwrap_or_else(|| default.to_string());
        let url = |name: &str, default: &str| or(name, default).trim_end_matches('/').to_string();

        let jwt_secret =
            opt("LOGLENS_JWT_SECRET").ok_or(ConfigError::Missing("LOGLENS_JWT_SECRET"))?;
        if is_weak_secret(&jwt_secret) {
            return Err(ConfigError::Invalid {
                name: "LOGLENS_JWT_SECRET",
                reason: "must be at least 32 random characters, not a placeholder (openssl rand -hex 32)".into(),
            });
        }
        if let Some(pw) = opt("LOGLENS_ADMIN_PASSWORD") {
            if pw.len() < 10 || is_placeholder(&pw) {
                return Err(ConfigError::Invalid {
                    name: "LOGLENS_ADMIN_PASSWORD",
                    reason: "must be at least 10 characters and not a placeholder".into(),
                });
            }
        }

        let provider = or("LOGLENS_AI_PROVIDER", "ollama")
            .parse()
            .map_err(|reason| ConfigError::Invalid {
                name: "LOGLENS_AI_PROVIDER",
                reason,
            })?;
        let mask = parse_bool(&or("LOGLENS_AI_MASK", "true")).ok_or(ConfigError::Invalid {
            name: "LOGLENS_AI_MASK",
            reason: "expected true or false".into(),
        })?;

        let ntfy = opt("LOGLENS_NTFY_URL").map(|raw| NtfySettings {
            url: raw.trim_end_matches('/').to_string(),
            topic: or("LOGLENS_NTFY_TOPIC", "loglens-alerts"),
        });

        Ok(Settings {
            bind: or("LOGLENS_BIND", "0.0.0.0:8080"),
            loki_url: url("LOGLENS_LOKI_URL", "http://localhost:3100"),
            db_url: or("LOGLENS_DB_URL", "sqlite://loglens.db"),
            admin_user: or("LOGLENS_ADMIN_USER", "admin"),
            admin_password: opt("LOGLENS_ADMIN_PASSWORD"),
            jwt_secret,
            ai: AiSettings {
                provider,
                url: url("LOGLENS_AI_URL", "http://localhost:11434"),
                model: or("LOGLENS_AI_MODEL", "llama3.1:8b"),
                key: opt("LOGLENS_AI_KEY"),
                mask,
            },
            ntfy,
            cors_origins: parse_origins(get("LOGLENS_CORS_ORIGINS").as_deref()),
        })
    }
}

fn is_placeholder(v: &str) -> bool {
    let lower = v.to_ascii_lowercase();
    ["replace", "change-me", "changeme", "example", "password"]
        .iter()
        .any(|w| lower.contains(w))
}

fn is_weak_secret(v: &str) -> bool {
    let distinct = v.chars().collect::<std::collections::HashSet<_>>().len();
    v.len() < 32 || distinct < 8 || is_placeholder(v)
}

fn parse_origins(raw: Option<&str>) -> Vec<String> {
    let split = |v: &str| -> Vec<String> {
        v.split(',')
            .map(|o| o.trim().trim_end_matches('/').to_string())
            .filter(|o| !o.is_empty())
            .collect()
    };
    let origins = raw.map(split).unwrap_or_default();
    if origins.is_empty() {
        split(Settings::DEFAULT_CORS_ORIGINS)
    } else {
        origins
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn defaults_apply() {
        let s = Settings::from_lookup(lookup(&[("LOGLENS_JWT_SECRET", SECRET)])).unwrap();
        assert_eq!(s.ai.provider, AiProviderKind::Ollama);
        assert!(s.ai.mask);
        assert!(s.ntfy.is_none());
        assert_eq!(s.bind, "0.0.0.0:8080");
        assert_eq!(
            s.cors_origins,
            [
                "http://tauri.localhost",
                "https://tauri.localhost",
                "tauri://localhost"
            ]
        );
    }

    #[test]
    fn reads_cors_origins() {
        let s = Settings::from_lookup(lookup(&[
            ("LOGLENS_JWT_SECRET", SECRET),
            (
                "LOGLENS_CORS_ORIGINS",
                " http://tauri.localhost/ , https://logs.example.com,, ",
            ),
        ]))
        .unwrap();
        assert_eq!(
            s.cors_origins,
            ["http://tauri.localhost", "https://logs.example.com"]
        );
        assert_eq!(parse_origins(Some(" , ")).len(), 3);
    }

    #[test]
    fn rejects_placeholder_secret_and_password() {
        let r = Settings::from_lookup(lookup(&[(
            "LOGLENS_JWT_SECRET",
            "replace-with-64-random-hex-characters-from-openssl-rand-hex-32",
        )]));
        assert!(matches!(
            r,
            Err(ConfigError::Invalid {
                name: "LOGLENS_JWT_SECRET",
                ..
            })
        ));
        let r = Settings::from_lookup(lookup(&[
            ("LOGLENS_JWT_SECRET", SECRET),
            ("LOGLENS_ADMIN_PASSWORD", "change-me"),
        ]));
        assert!(matches!(
            r,
            Err(ConfigError::Invalid {
                name: "LOGLENS_ADMIN_PASSWORD",
                ..
            })
        ));
    }

    #[test]
    fn requires_jwt_secret() {
        assert_eq!(
            Settings::from_lookup(lookup(&[])),
            Err(ConfigError::Missing("LOGLENS_JWT_SECRET"))
        );
    }

    #[test]
    fn rejects_unknown_provider() {
        let r = Settings::from_lookup(lookup(&[
            ("LOGLENS_JWT_SECRET", SECRET),
            ("LOGLENS_AI_PROVIDER", "nope"),
        ]));
        assert!(matches!(
            r,
            Err(ConfigError::Invalid {
                name: "LOGLENS_AI_PROVIDER",
                ..
            })
        ));
    }

    #[test]
    fn reads_ntfy_and_openai() {
        let s = Settings::from_lookup(lookup(&[
            ("LOGLENS_JWT_SECRET", SECRET),
            ("LOGLENS_AI_PROVIDER", "openai"),
            ("LOGLENS_AI_KEY", "sk-test"),
            ("LOGLENS_AI_MASK", "false"),
            ("LOGLENS_NTFY_URL", "http://ntfy:80/"),
        ]))
        .unwrap();
        assert_eq!(s.ai.provider, AiProviderKind::OpenAi);
        assert_eq!(s.ai.key.as_deref(), Some("sk-test"));
        assert!(!s.ai.mask);
        let ntfy = s.ntfy.unwrap();
        assert_eq!(ntfy.url, "http://ntfy:80");
        assert_eq!(ntfy.topic, "loglens-alerts");
    }
}
