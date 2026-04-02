use std::pin::Pin;
use std::task::{Context, Poll};

use http::{self, Request, Response};
use tonic::body::{empty_body, BoxBody};
use tower::{Layer, Service};

use crate::middleware::auth_middleware::AuthState;
use crate::middleware::rbac::{check_permission, Tier};

/// 汎用 gRPC 認証レイヤー。
///
/// `action_mapper` でサービス固有の gRPC メソッド→アクション変換を注入する。
#[derive(Clone)]
pub struct GrpcAuthLayer {
    auth_state: Option<AuthState>,
    tier: Tier,
    action_mapper: fn(&str) -> &'static str,
}

impl GrpcAuthLayer {
    pub fn new(
        auth_state: Option<AuthState>,
        tier: Tier,
        action_mapper: fn(&str) -> &'static str,
    ) -> Self {
        Self {
            auth_state,
            tier,
            action_mapper,
        }
    }
}

impl<S> Layer<S> for GrpcAuthLayer {
    type Service = GrpcAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GrpcAuthService {
            inner,
            auth_state: self.auth_state.clone(),
            tier: self.tier,
            action_mapper: self.action_mapper,
        }
    }
}

#[derive(Clone)]
pub struct GrpcAuthService<S> {
    inner: S,
    auth_state: Option<AuthState>,
    tier: Tier,
    action_mapper: fn(&str) -> &'static str,
}

impl<S, ReqBody> Service<Request<ReqBody>> for GrpcAuthService<S>
where
    S: Service<Request<ReqBody>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Error: Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let auth_state = self.auth_state.clone();
        let tier = self.tier;
        let action_mapper = self.action_mapper;
        let path = req.uri().path().to_string();

        Box::pin(async move {
            if let Some(auth_state) = auth_state {
                let token = match extract_bearer_token(&req) {
                    Some(token) => token,
                    None => {
                        return Ok(unauthenticated_response(
                            "Authorization metadata with Bearer token is required",
                        ));
                    }
                };

                let claims = match auth_state.verifier.verify_token(&token).await {
                    Ok(claims) => claims,
                    Err(_) => {
                        return Ok(unauthenticated_response("Token validation failed"));
                    }
                };

                let method = grpc_method_name(&path);
                let action = action_mapper(method);
                if !check_permission(tier, claims.realm_roles(), action) {
                    return Ok(permission_denied_response(&format!(
                        "Insufficient permissions: action '{}' is not allowed for gRPC method '{}'.",
                        action, path
                    )));
                }

                req.extensions_mut().insert(claims);
            }

            inner.call(req).await
        })
    }
}

fn extract_bearer_token<B>(req: &Request<B>) -> Option<String> {
    let auth_header = req.headers().get(http::header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    // RFC 7235: Authorization スキーム名は大文字小文字を区別しない（lessons.md HIGH-007 対応）
    // "Bearer ", "bearer ", "BEARER " いずれも受け入れる
    const BEARER_PREFIX_LEN: usize = 7; // "bearer ".len()
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
        Some(token.to_string())
    }
}

fn grpc_method_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn unauthenticated_response(message: &str) -> Response<BoxBody> {
    let mut response = Response::new(empty_body());
    *response.status_mut() = http::StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/grpc"),
    );
    headers.insert("grpc-status", http::HeaderValue::from_static("16"));
    headers.insert(
        "grpc-message",
        http::HeaderValue::from_str(message)
            .unwrap_or_else(|_| http::HeaderValue::from_static("Unauthenticated")),
    );
    response
}

fn permission_denied_response(message: &str) -> Response<BoxBody> {
    let mut response = Response::new(empty_body());
    *response.status_mut() = http::StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/grpc"),
    );
    headers.insert("grpc-status", http::HeaderValue::from_static("7"));
    headers.insert(
        "grpc-message",
        http::HeaderValue::from_str(message)
            .unwrap_or_else(|_| http::HeaderValue::from_static("Permission denied")),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;

    fn make_request_with_auth(value: &str) -> Request<()> {
        Request::builder()
            .header(http::header::AUTHORIZATION, value)
            .body(())
            .unwrap()
    }

    #[test]
    fn test_extract_bearer_token_uppercase() {
        // 大文字 Bearer が正常に動作することを確認（RFC 7235 準拠）
        let req = make_request_with_auth("Bearer test-token");
        assert_eq!(
            extract_bearer_token(&req),
            Some("test-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_lowercase() {
        // 小文字 bearer が受け入れられることを確認（RFC 7235: スキーム名は大文字小文字不区別）
        let req = make_request_with_auth("bearer test-token");
        assert_eq!(
            extract_bearer_token(&req),
            Some("test-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_mixed_case() {
        // 混在ケース BEARER が受け入れられることを確認
        let req = make_request_with_auth("BEARER test-token");
        assert_eq!(
            extract_bearer_token(&req),
            Some("test-token".to_string())
        );
    }

    #[test]
    fn test_extract_bearer_token_empty() {
        // トークンが空の場合は None を返す
        let req = make_request_with_auth("Bearer ");
        assert_eq!(extract_bearer_token(&req), None);
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        // Authorization ヘッダーがない場合は None を返す
        let req = Request::builder().body(()).unwrap();
        assert_eq!(extract_bearer_token(&req), None);
    }
}
