use std::sync::Arc;

use axum::{
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

/// auth_layer は Authorization ヘッダーの Bearer JWT を検証する axum ミドルウェアレイヤーを返す。
/// `verifier` を Arc でキャプチャし、`from_fn` ミドルウェアとして返す。
pub fn auth_layer(
    verifier: Arc<JwksVerifier>,
) -> axum::middleware::FromFnLayer<
    impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
        + Clone,
    (),
    (),
> {
    axum::middleware::from_fn(move |req: Request, next: Next| {
        let verifier = verifier.clone();
        Box::pin(async move { verify_jwt(verifier, req, next).await })
            as std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
    })
}

async fn verify_jwt(verifier: Arc<JwksVerifier>, mut req: Request, next: Next) -> Response {
    let token = extract_bearer_token(req.headers());

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "missing Authorization header"
                    }
                })),
            )
                .into_response();
        }
    };

    let claims = match verifier.verify_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "invalid or expired JWT token"
                    }
                })),
            )
                .into_response();
        }
    };

    req.extensions_mut().insert(claims);
    next.run(req).await
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_owned())
}
