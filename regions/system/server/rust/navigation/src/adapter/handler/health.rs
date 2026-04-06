use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

// ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
// timestamp フィールドを追加して監視ダッシュボードでの時系列確認を可能にする
// MEDIUM-RUST-002 監査対応: readyz エンドポイントで navigation.yaml のロード可否を確認する。
// navigation サービスは DB を持たないため、設定ファイル（navigation.yaml）のロードをヘルスチェックに使用する。
// ロード失敗（ファイル不在・パース失敗）は SERVICE_UNAVAILABLE を返し、
// Kubernetes readinessProbe がトラフィックを遮断するようにする。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();

    // MEDIUM-RUST-002 監査対応: navigation 設定ファイルのロード可否を確認する。
    // DB を持たないサービスのため、YAML 設定ファイルの可読性をレディネスチェックとして使用する。
    match state.get_navigation_uc.check_config_loadable() {
        Ok(_) => Json(serde_json::json!({
            "status": "healthy",
            "checks": {
                "navigation_config": "ok"
            },
            "timestamp": timestamp
        }))
        .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "checks": {
                    "navigation_config": e.to_string()
                },
                "timestamp": timestamp
            })),
        )
            .into_response(),
    }
}
