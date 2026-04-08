use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// ヘルスチェックエンドポイント: サーバーが起動しているかを返す
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// レディネスチェックエンドポイント: ADR-0068 準拠形式で返す（MED-007/MED-008 監査対応）。
/// in-memory バックエンド使用時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
/// MED-001 対応: postgres バックエンドの場合は DB 接続を実際に確認し、
///              エラー詳細を `tracing::error`! でログ出力する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068 標準: healthy / degraded / unhealthy の3値を使用する
    // MED-001 対応: バックエンド種別に応じて実際の接続確認を実施する
    let (status, db_check) = if state.backend_kind == "in-memory" {
        // in-memory バックエンドは外部接続不要のため degraded（機能制限あり）を返す
        ("degraded", "not_configured")
    } else if let Some(pool) = &state.db_pool {
        // postgres バックエンドは DB への実際の疎通確認を行う
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => ("healthy", "ok"),
            Err(e) => {
                tracing::error!(error = %e, "readyz: DB health check failed");
                ("unhealthy", "error")
            }
        }
    } else {
        // backend_kind が postgres だが db_pool が None（設定ミス）
        tracing::error!("readyz: backend_kind is '{}' but db_pool is None", state.backend_kind);
        ("unhealthy", "misconfigured")
    };

    let status_code = if status == "unhealthy" {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    (
        status_code,
        Json(serde_json::json!({
            "status": status,
            "checks": {
                "rule_engine_backend": state.backend_kind,
                "database": db_check
            },
            "timestamp": timestamp
        })),
    )
}
