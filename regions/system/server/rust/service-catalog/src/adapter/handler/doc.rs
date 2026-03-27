use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::{AppState, ErrorResponse};
use crate::domain::entity::service_doc::ServiceDoc;

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/docs",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 200, description = "Documents list", body = Vec<ServiceDoc>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_docs(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.manage_docs_uc.list(id).await {
        Ok(docs) => (StatusCode::OK, Json(docs)).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/services/{id}/docs",
    params(("id" = Uuid, Path, description = "Service ID")),
    request_body = Vec<ServiceDoc>,
    responses(
        (status = 204, description = "Documents set"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn set_docs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(docs): Json<Vec<ServiceDoc>>,
) -> impl IntoResponse {
    match state.manage_docs_uc.set(id, docs).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
