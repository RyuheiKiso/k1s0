// バージョン履歴 REST ハンドラ。
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use super::{error::map_domain_error, AppState};

pub async fn list_versions(
    State(state): State<AppState>,
    Path(status_id): Path<Uuid>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .get_versions_uc
        .list(status_id, 50, 0)
        .await
        .map(|(versions, _total)| Json(versions))
        .map_err(map_domain_error)
}
