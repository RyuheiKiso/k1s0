use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use k1s0_auth::Claims;

pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    Request<Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |req: Request<Body>, next: Next| Box::pin(rbac_check(req, next, resource, action))
}

fn check_system_permission(roles: &[String], _resource: &str, action: &str) -> bool {
    for role in roles {
        match role.as_str() {
            "sys_admin" => return true,
            "sys_operator" => {
                if action == "read" || action == "write" {
                    return true;
                }
            }
            "sys_auditor" => {
                if action == "read" {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

async fn rbac_check(req: Request<Body>, next: Next, resource: &str, action: &str) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SYS_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    let roles = claims.realm_roles().to_vec();

    if check_system_permission(&roles, resource, action) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_AUTH_PERMISSION_DENIED",
                    "message": format!(
                        "Insufficient permissions: action '{}' on resource '{}' is not allowed for the current roles.",
                        action, resource
                    )
                }
            })),
        )
            .into_response()
    }
}
