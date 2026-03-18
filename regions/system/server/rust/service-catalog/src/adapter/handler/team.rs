use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{AppState, ErrorResponse};
#[allow(unused_imports)]
use crate::domain::entity::service::Service;
#[allow(unused_imports)]
use crate::domain::entity::team::Team;
use crate::domain::repository::service_repository::ServiceListFilters;
use crate::usecase::{
    CreateTeamError, CreateTeamInput, DeleteTeamError, GetTeamError, UpdateTeamError,
    UpdateTeamInput,
};

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub slack_channel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub contact_email: Option<String>,
    pub slack_channel: Option<String>,
}

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
        Ok(teams) => (StatusCode::OK, Json(teams)).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
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
        Ok(services) => (
            StatusCode::OK,
            Json(services),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/teams/{team_id}
pub async fn get_team(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_team_uc.execute(team_id).await {
        Ok(team) => (StatusCode::OK, Json(team)).into_response(),
        Err(GetTeamError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_006", "team not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(GetTeamError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_005", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/teams
pub async fn create_team(
    State(state): State<AppState>,
    Json(req): Json<CreateTeamRequest>,
) -> impl IntoResponse {
    let input = CreateTeamInput {
        name: req.name,
        description: req.description,
        contact_email: req.contact_email,
        slack_channel: req.slack_channel,
    };

    match state.create_team_uc.execute(input).await {
        Ok(team) => (
            StatusCode::CREATED,
            Json(team),
        )
            .into_response(),
        Err(CreateTeamError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_004", msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(CreateTeamError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_005", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// PUT /api/v1/teams/{team_id}
pub async fn update_team(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
    Json(req): Json<UpdateTeamRequest>,
) -> impl IntoResponse {
    let input = UpdateTeamInput {
        id: team_id,
        name: req.name,
        description: req.description,
        contact_email: req.contact_email,
        slack_channel: req.slack_channel,
    };

    match state.update_team_uc.execute(input).await {
        Ok(team) => (StatusCode::OK, Json(team)).into_response(),
        Err(UpdateTeamError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_006", "team not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(UpdateTeamError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_004", msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(UpdateTeamError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_005", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// DELETE /api/v1/teams/{team_id}
pub async fn delete_team(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.delete_team_uc.execute(team_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteTeamError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_006", "team not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeleteTeamError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_005", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
