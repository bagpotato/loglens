mod detect;
mod stream;

use std::collections::HashMap;

pub use stream::unmask_stream;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    Ip,
    User,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Ip => "IP",
            Kind::User => "USER",
        }
    }
}

/// Replaces IP addresses and usernames with stable placeholders (`<IP_1>`, `<USER_1>`)
/// before text is sent to an AI provider, and restores them in the answer.
#[derive(Debug, Clone, Default)]
pub struct Masker {
    enabled: bool,
    forward: HashMap<(Kind, String), String>,
    reverse: HashMap<String, String>,
    ips: usize,
    users: usize,
}

impl Masker {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// True once at least one value has been replaced.
    pub fn has_placeholders(&self) -> bool {
        !self.reverse.is_empty()
    }

    pub fn mask(&mut self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        let text = detect::replace_ips(text, |ip| self.placeholder(Kind::Ip, ip));
        detect::replace_users(&text, |user| self.placeholder(Kind::User, user))
    }

    pub fn unmask(&self, text: &str) -> String {
        if self.reverse.is_empty() {
            return text.to_string();
        }
        detect::replace_placeholders(text, |p| self.reverse.get(p).cloned())
    }

    fn placeholder(&mut self, kind: Kind, value: &str) -> String {
        let key = (kind, value.to_string());
        if let Some(existing) = self.forward.get(&key) {
            return existing.clone();
        }
        let counter = match kind {
            Kind::Ip => &mut self.ips,
            Kind::User => &mut self.users,
        };
        *counter += 1;
        let placeholder = format!("<{}_{}>", kind.label(), counter);
        self.reverse.insert(placeholder.clone(), value.to_string());
        self.forward.insert(key, placeholder.clone());
        placeholder
    }
}
