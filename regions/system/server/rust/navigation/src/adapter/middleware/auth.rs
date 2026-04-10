//! JWT 認証ミドルウェア（Tower Layer / Service）。
//!
//! Authorization ヘッダーから Bearer トークンを取得し、`k1s0_auth::JwksVerifier` で
//! 署名検証を行う。検証成功時は `k1s0_auth::Claims` をリクエストエクステンションに格納する。
//! navigation-server の REST API (`/api/v1/navigation`) で使用する。

use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use k1s0_auth::JwksVerifier;

/// Authorization ヘッダーから Bearer トークンを抽出する。
/// RFC 7235: Authorization スキーム名は大文字小文字を区別しない（RUST-HIGH-001 対応）
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    // HIGH-001 監査対応: const は文より前に定義する
    // "Bearer ", "bearer ", "BEARER " いずれも受け入れる
    const BEARER_PREFIX_LEN: usize = 7; // "bearer ".len()
    let auth_str = headers.get("Authorization").and_then(|v| v.to_str().ok())?;
    if auth_str.len() < BEARER_PREFIX_LEN {
        return None;
    }
    if !auth_str[..BEARER_PREFIX_LEN].eq_ignore_ascii_case("bearer ") {
        return None;
    }
    let token = &auth_str[BEARER_PREFIX_LEN..];
    if token.is_empty() {
        None
    } else {
        Some(token.to_owned())
    }
}

/// エラーレスポンスを生成するヘルパー。
fn unauthorized_response(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "SYS_AUTH_UNAUTHENTICATED",
            "message": message
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Tower Layer / Service
// ---------------------------------------------------------------------------

/// `AuthMiddlewareLayer` は `JwksVerifier` を保持し、JWT 検証ミドルウェアを提供する Tower Layer。
#[derive(Clone)]
pub struct AuthMiddlewareLayer {
    verifier: Arc<JwksVerifier>,
}

impl AuthMiddlewareLayer {
    #[must_use]
    pub fn new(verifier: Arc<JwksVerifier>) -> Self {
        Self { verifier }
    }
}

impl<S> tower::Layer<S> for AuthMiddlewareLayer {
    type Service = AuthMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddlewareService {
            inner,
            verifier: self.verifier.clone(),
        }
    }
}

/// `AuthMiddlewareService` は JWT 検証を行う Tower Service。
///
/// 検証成功時は `k1s0_auth::Claims` をリクエストエクステンションに挿入し、
/// 後続の RBAC ミドルウェアやハンドラから参照できるようにする。
#[derive(Clone)]
pub struct AuthMiddlewareService<S> {
    inner: S,
    verifier: Arc<JwksVerifier>,
}

impl<S> tower::Service<Request<Body>> for AuthMiddlewareService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let verifier = self.verifier.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // HIGH-001 監査対応: let...else パターンで可読性を向上する
            // Bearer トークンを取得
            let Some(token) = extract_bearer_token(req.headers()) else {
                return Ok(unauthorized_response("missing Authorization header"));
            };

            // JWKS 検証
            let Ok(claims) = verifier.verify_token(&token).await else {
                return Ok(unauthorized_response("invalid or expired JWT token"));
            };

            // Claims をエクステンションに格納（後続ミドルウェア/ハンドラで利用可能）
            req.extensions_mut().insert(claims);
            inner.call(req).await
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn extract_bearer_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer abc123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), Some("abc123".to_string()));
    }

    #[test]
    fn extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        assert_eq!(extract_bearer_token(&headers), None);
    }

    #[test]
    fn extract_bearer_token_no_bearer_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic abc123".parse().unwrap());
        assert_eq!(extract_bearer_token(&headers), None);
    }
}
