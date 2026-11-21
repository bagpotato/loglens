use std::fmt;
use std::sync::Arc;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use loglens_storage::User;
use serde::{Deserialize, Serialize};

use crate::AuthError;

const TOKEN_TTL_HOURS: i64 = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Username.
    pub sub: String,
    pub uid: i64,
    pub tid: i64,
    pub role: String,
    pub exp: usize,
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
}

/// HS256 signing keys. Cheap to clone.
#[derive(Clone)]
pub struct JwtKeys(Arc<Keys>);

impl fmt::Debug for JwtKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JwtKeys(..)")
    }
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self(Arc::new(Keys {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::new(Algorithm::HS256),
        }))
    }

    /// Token valid for 12 hours.
    pub fn issue(&self, user: &User) -> Result<String, AuthError> {
        self.issue_with_ttl(user, Duration::hours(TOKEN_TTL_HOURS))
    }

    pub(crate) fn issue_with_ttl(&self, user: &User, ttl: Duration) -> Result<String, AuthError> {
        let exp = (Utc::now() + ttl).timestamp().max(0);
        let claims = Claims {
            sub: user.username.clone(),
            uid: user.id,
            tid: user.tenant_id,
            role: user.role.clone(),
            exp: usize::try_from(exp).map_err(|e| AuthError::Token(e.to_string()))?,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.0.encoding)
            .map_err(|e| AuthError::Token(e.to_string()))
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.0.decoding, &self.0.validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn user(id: i64, name: &str) -> User {
        User {
            id,
            tenant_id: 1,
            username: name.into(),
            password_hash: String::new(),
            role: "admin".into(),
        }
    }

    #[test]
    fn roundtrip() {
        let keys = JwtKeys::new(SECRET);
        let claims = keys
            .verify(&keys.issue(&user(7, "alice")).unwrap())
            .unwrap();
        assert_eq!(claims.sub, "alice");
        assert_eq!((claims.uid, claims.tid), (7, 1));
        assert_eq!(claims.role, "admin");
        let ttl = claims.exp as i64 - Utc::now().timestamp();
        assert!((TOKEN_TTL_HOURS * 3600 - 5..=TOKEN_TTL_HOURS * 3600).contains(&ttl));
    }

    #[test]
    fn tampered_token_is_rejected() {
        let keys = JwtKeys::new(SECRET);
        let a = keys.issue(&user(1, "alice")).unwrap();
        let b = keys.issue(&user(2, "mallory")).unwrap();
        let a_parts: Vec<&str> = a.split('.').collect();
        let b_parts: Vec<&str> = b.split('.').collect();
        let forged = format!("{}.{}.{}", a_parts[0], b_parts[1], a_parts[2]);
        assert!(matches!(keys.verify(&forged), Err(AuthError::Unauthorized)));

        let other = JwtKeys::new("another-secret-another-secret-123");
        assert!(matches!(other.verify(&a), Err(AuthError::Unauthorized)));
        assert!(matches!(
            keys.verify("garbage"),
            Err(AuthError::Unauthorized)
        ));
    }

    #[test]
    fn expired_token_is_rejected() {
        let keys = JwtKeys::new(SECRET);
        let token = keys
            .issue_with_ttl(&user(1, "alice"), Duration::hours(-2))
            .unwrap();
        assert!(matches!(keys.verify(&token), Err(AuthError::Unauthorized)));
    }
}
