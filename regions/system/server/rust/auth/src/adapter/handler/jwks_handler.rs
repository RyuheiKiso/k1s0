use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use super::{AppState, ErrorResponse};

#[utoipa::path(
    get,
    path = "/jwks",
    responses(
        (status = 200, description = "JWKS"),
        (status = 503, description = "JWKS provider unavailable", body = ErrorResponse)
    )
)]
pub async fn jwks(State(state): State<AppState>) -> impl IntoResponse {
    let Some(provider) = state.jwks_provider.clone() else {
        let err = ErrorResponse::new(
            "SYS_AUTH_JWKS_UNAVAILABLE",
            "JWKS provider is not configured (keycloak config missing)",
        );
        return (StatusCode::SERVICE_UNAVAILABLE, Json(err)).into_response();
    };

    match provider.get().await {
        Ok(value) => (StatusCode::OK, Json(value)).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "failed to fetch JWKS");
            let err = ErrorResponse::new("SYS_AUTH_JWKS_FETCH_FAILED", "Failed to fetch JWKS");
            (StatusCode::SERVICE_UNAVAILABLE, Json(err)).into_response()
        }
    }
}
