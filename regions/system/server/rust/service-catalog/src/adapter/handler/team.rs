use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::{AppState, ErrorResponse};
#[allow(unused_imports)]
use crate::domain::entity::service::Service;
#[allow(unused_imports)]
use crate::domain::entity::team::Team;
use crate::domain::repository::service_repository::ServiceListFilters;

#[utoipa::path(
    get,
    path = "/api/v1/teams",
    responses(
        (status = 200, description = "Team list", body = Vec<Team>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_teams(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_teams_uc.execute().await {
        Ok(teams) => (StatusCode::OK, Json(serde_json::to_value(teams).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/teams/{team_id}/services",
    params(("team_id" = Uuid, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Services for team", body = Vec<Service>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_team_services(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    let filters = ServiceListFilters {
        team_id: Some(team_id),
        ..Default::default()
    };

    match state.list_services_uc.execute(filters).await {
        Ok(services) => (StatusCode::OK, Json(serde_json::to_value(services).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
