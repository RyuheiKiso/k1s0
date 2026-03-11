use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::{AppState, ErrorResponse};
use crate::domain::entity::health::HealthStatus;

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/health",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 200, description = "Health status", body = HealthStatus),
        (status = 404, description = "No health status found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_health(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.health_status_uc.get(id).await {
        Ok(Some(health)) => {
            (StatusCode::OK, Json(serde_json::to_value(health).unwrap())).into_response()
        }
        Ok(None) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "No health status found for this service");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/services/{id}/health",
    params(("id" = Uuid, Path, description = "Service ID")),
    request_body = HealthStatus,
    responses(
        (status = 204, description = "Health status reported"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn report_health(
    State(state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(health): Json<HealthStatus>,
) -> impl IntoResponse {
    match state.health_status_uc.report(&health).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
