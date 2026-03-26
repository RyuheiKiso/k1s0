use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response};
use k1s0_auth::Claims;
use tonic::body::{empty_body, BoxBody};
use tower::{Layer, Service};

use crate::adapter::middleware::auth::AuthState;
use crate::adapter::middleware::rbac::check_system_permission;

#[derive(Clone)]
pub struct GrpcAuthLayer {
    auth_state: Option<AuthState>,
}

impl GrpcAuthLayer {
    pub fn new(auth_state: Option<AuthState>) -> Self {
        Self { auth_state }
    }
}

impl<S> Layer<S> for GrpcAuthLayer {
    type Service = GrpcAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GrpcAuthService {
            inner,
            auth_state: self.auth_state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GrpcAuthService<S> {
    inner: S,
    auth_state: Option<AuthState>,
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
        let path = req.uri().path().to_string();

        Box::pin(async move {
            if let Some(auth_state) = auth_state {
                let token = match extract_bearer_token(&req) {
                    Some(token) => token,
                    None => {
                        return Ok(unauthenticated_response(
                            "SYS_AUTH_MISSING_TOKEN",
                            "Authorization metadata with Bearer token is required",
                        ));
                    }
                };

                let claims = match auth_state.verifier.verify_token(&token).await {
                    Ok(claims) => claims,
                    Err(_) => {
                        return Ok(unauthenticated_response(
                            "SYS_AUTH_TOKEN_INVALID",
                            "Token validation failed",
                        ));
                    }
                };

                let action = required_action(&path);
                if !authorize_claims(&claims, action) {
                    return Ok(permission_denied_response(
                        "SYS_AUTH_PERMISSION_DENIED",
                        &format!(
                            "Insufficient permissions: action '{}' is not allowed for gRPC method '{}'.",
                            action, path
                        ),
                    ));
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
    let token = auth_str.strip_prefix("Bearer ")?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn required_action(path: &str) -> &'static str {
    match grpc_method_name(path) {
        "ListWorkflows" | "GetWorkflow" | "GetInstance" | "ListInstances" | "ListTasks" => "read",
        "DeleteWorkflow" | "CancelInstance" => "admin",
        _ => "write",
    }
}

fn grpc_method_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn authorize_claims(claims: &Claims, action: &str) -> bool {
    check_system_permission(claims.realm_roles(), "workflows", action)
}

fn unauthenticated_response(code: &str, message: &str) -> Response<BoxBody> {
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
    headers.insert(
        "x-error-code",
        http::HeaderValue::from_str(code)
            .unwrap_or_else(|_| http::HeaderValue::from_static("SYS_AUTH_TOKEN_INVALID")),
    );
    response
}

fn permission_denied_response(code: &str, message: &str) -> Response<BoxBody> {
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
    headers.insert(
        "x-error-code",
        http::HeaderValue::from_str(code)
            .unwrap_or_else(|_| http::HeaderValue::from_static("SYS_AUTH_PERMISSION_DENIED")),
    );
    response
}

#[cfg(test)]
mod tests {
    use k1s0_auth::claims::{Audience, RealmAccess};

    use super::*;

    fn make_claims(role_names: &[&str]) -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: Audience(vec!["k1s0-api".to_string()]),
            exp: 9999999999,
            iat: 1000000000,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: Some("taro.yamada".to_string()),
            email: Some("taro@example.com".to_string()),
            realm_access: Some(RealmAccess {
                roles: role_names.iter().map(|s| s.to_string()).collect(),
            }),
            resource_access: None,
            tier_access: None,
            tenant_id: String::new(),
        }
    }

    #[test]
    fn extracts_method_name() {
        assert_eq!(
            grpc_method_name("/k1s0.system.workflow.v1.WorkflowService/ListWorkflows"),
            "ListWorkflows"
        );
    }

    #[test]
    fn maps_grpc_method_to_action() {
        assert_eq!(
            required_action("/k1s0.system.workflow.v1.WorkflowService/ListWorkflows"),
            "read"
        );
        assert_eq!(
            required_action("/k1s0.system.workflow.v1.WorkflowService/CancelInstance"),
            "admin"
        );
        assert_eq!(
            required_action("/k1s0.system.workflow.v1.WorkflowService/ApproveTask"),
            "write"
        );
    }

    #[test]
    fn authorizes_using_same_role_mapping_as_rest() {
        assert!(authorize_claims(&make_claims(&["sys_auditor"]), "read"));
        assert!(!authorize_claims(&make_claims(&["sys_auditor"]), "write"));
        assert!(authorize_claims(&make_claims(&["sys_operator"]), "write"));
        assert!(!authorize_claims(&make_claims(&["sys_operator"]), "admin"));
        assert!(authorize_claims(&make_claims(&["sys_admin"]), "admin"));
    }
}
