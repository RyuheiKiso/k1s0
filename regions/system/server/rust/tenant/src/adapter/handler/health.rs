use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use super::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: state.app_name.clone(),
        version: state.version.clone(),
    })
}
