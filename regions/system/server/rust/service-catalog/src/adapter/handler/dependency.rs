use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::{AppState, ErrorResponse};
use crate::domain::entity::dependency::Dependency;

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/dependencies",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 200, description = "Dependencies list", body = Vec<Dependency>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_dependencies(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.manage_deps_uc.list(id).await {
        Ok(deps) => (StatusCode::OK, Json(serde_json::to_value(deps).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/services/{id}/dependencies",
    params(("id" = Uuid, Path, description = "Service ID")),
    request_body = Vec<Dependency>,
    responses(
        (status = 204, description = "Dependencies set"),
        (status = 409, description = "Dependency cycle detected"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_dependencies(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(deps): Json<Vec<Dependency>>,
) -> impl IntoResponse {
    match state.manage_deps_uc.set(id, deps).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::manage_dependencies::ManageDependenciesError::CycleDetected(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_004", "Dependency cycle detected");
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
