use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use super::{AppState, ErrorResponse};
#[allow(unused_imports)]
use crate::domain::entity::scorecard::Scorecard;

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}/scorecard",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 200, description = "Scorecard", body = Scorecard),
        (status = 404, description = "Scorecard not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_scorecard(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_scorecard_uc.execute(id).await {
        Ok(scorecard) => (
            StatusCode::OK,
            Json(scorecard),
        )
            .into_response(),
        Err(crate::usecase::get_scorecard::GetScorecardError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "No scorecard found for this service");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
