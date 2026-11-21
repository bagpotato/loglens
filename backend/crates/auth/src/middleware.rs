use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::{AUTHORIZATION, SEC_WEBSOCKET_PROTOCOL};
use axum::http::request::Parts;
use axum::http::HeaderMap;

use crate::{AuthError, JwtKeys};

/// Authenticated caller, extracted from a valid JWT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub user_id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub role: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    JwtKeys: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = token_from_headers(&parts.headers).ok_or(AuthError::Unauthorized)?;
        let claims = JwtKeys::from_ref(state).verify(&token)?;
        Ok(AuthUser {
            user_id: claims.uid,
            tenant_id: claims.tid,
            username: claims.sub,
            role: claims.role,
        })
    }
}

/// `Authorization: Bearer <t>`, else `Sec-WebSocket-Protocol: bearer, <t>`.
fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    bearer_header(headers).or_else(|| websocket_protocol(headers))
}

fn bearer_header(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?.trim();
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("bearer") && !token.is_empty()).then(|| token.to_string())
}

fn websocket_protocol(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(str::trim)
        .skip_while(|p| !p.eq_ignore_ascii_case("bearer"))
        .nth(1)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use loglens_storage::User;
    use tower::ServiceExt;

    use super::*;

    fn setup() -> (Router, String) {
        let keys = JwtKeys::new("0123456789abcdef0123456789abcdef");
        let user = User {
            id: 3,
            tenant_id: 1,
            username: "alice".into(),
            password_hash: String::new(),
            role: "admin".into(),
        };
        let token = keys.issue(&user).unwrap();
        let app = Router::new()
            .route("/me", get(|u: AuthUser| async move { u.username }))
            .with_state(keys);
        (app, token)
    }

    async fn call(app: Router, header: Option<(&str, String)>) -> (StatusCode, String) {
        let mut req = Request::builder().uri("/me");
        if let Some((name, value)) = header {
            req = req.header(name, value);
        }
        let res = app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
        let status = res.status();
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn rejects_missing_token_with_json() {
        let (app, _) = setup();
        let (status, body) = call(app, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body, r#"{"error":"unauthorized"}"#);
    }

    #[tokio::test]
    async fn rejects_invalid_token() {
        let (app, _) = setup();
        let (status, _) = call(app, Some(("authorization", "Bearer nope".into()))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_bearer_header() {
        let (app, token) = setup();
        let (status, body) = call(app, Some(("authorization", format!("Bearer {token}")))).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "alice");
    }

    #[tokio::test]
    async fn accepts_websocket_protocol_header() {
        let (app, token) = setup();
        let header = ("sec-websocket-protocol", format!("bearer, {token}"));
        let (status, body) = call(app, Some(header)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "alice");
    }
}
