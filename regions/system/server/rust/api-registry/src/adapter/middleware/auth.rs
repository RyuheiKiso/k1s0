use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    Json,
};

/// Auth middleware that checks for Bearer token presence.
/// In production, this would verify the JWT token against a JWKS endpoint.
/// For now, it only checks that the Authorization header is present.
pub async fn auth_middleware(request: Request, next: Next) -> impl IntoResponse {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth_header {
        Some(_token) => {
            // TODO: Verify token using k1s0-auth JwksVerifier when auth config is provided
            next.run(request).await.into_response()
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_APIREG_UNAUTHORIZED",
                    "message": "Missing Authorization header"
                }
            })),
        )
            .into_response(),
    }
}
