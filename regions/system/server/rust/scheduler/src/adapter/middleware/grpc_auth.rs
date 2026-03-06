use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response};
use tonic::body::{empty_body, BoxBody};
use tower::{Layer, Service};

use crate::adapter::middleware::auth::{claims_have_system_tier, SchedulerAuthState};
use crate::adapter::middleware::rbac::check_system_permission;

#[derive(Clone)]
pub struct GrpcAuthLayer {
    auth_state: Option<SchedulerAuthState>,
}

impl GrpcAuthLayer {
    pub fn new(auth_state: Option<SchedulerAuthState>) -> Self {
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
    auth_state: Option<SchedulerAuthState>,
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

                if !claims_have_system_tier(&claims) {
                    return Ok(permission_denied_response(
                        "SYS_AUTH_TIER_FORBIDDEN",
                        "Token does not include required tier access: system",
                    ));
                }

                let action = required_action(&path);
                if !check_system_permission(claims.realm_roles(), "scheduler", action) {
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
        "ListJobs" | "GetJob" | "GetJobExecution" | "ListExecutions" => "read",
        "DeleteJob" => "admin",
        _ => "write",
    }
}

fn grpc_method_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
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
