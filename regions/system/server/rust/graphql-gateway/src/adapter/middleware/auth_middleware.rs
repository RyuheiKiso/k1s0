use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::infra::auth::JwksVerifier;

/// JWT Claims（async-graphql の Extension として GraphQL Context に注入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: i64,
}

impl Claims {
    pub fn roles(&self) -> Vec<String> {
        self.realm_access
            .as_ref()
            .map(|r| r.roles.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_owned())
}

/// AuthMiddlewareLayer は JwksVerifier を保持し、JWT 検証ミドルウェアを提供する Tower Layer。
#[derive(Clone)]
pub struct AuthMiddlewareLayer {
    verifier: Arc<JwksVerifier>,
}

impl AuthMiddlewareLayer {
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

/// AuthMiddlewareService は JWT 検証を行う Tower Service。
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
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

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
            let token = extract_bearer_token(req.headers());

            let token = match token {
                Some(t) => t,
                None => {
                    return Ok((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": {
                                "code": "UNAUTHORIZED",
                                "message": "missing Authorization header"
                            }
                        })),
                    )
                        .into_response());
                }
            };

            let claims = match verifier.verify_token(&token).await {
                Ok(c) => c,
                Err(_) => {
                    return Ok((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": {
                                "code": "UNAUTHORIZED",
                                "message": "invalid or expired JWT token"
                            }
                        })),
                    )
                        .into_response());
                }
            };

            req.extensions_mut().insert(claims);
            inner.call(req).await
        })
    }
}
