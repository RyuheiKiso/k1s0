use std::sync::atomic::Ordering;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use super::AppState;

/// liveness probe: プロセスが起動していれば常に ok を返す。
/// CRITICAL-003 監査対応: DB 確認は readyz に移動し、healthz は liveness のみとする。
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

/// readiness probe: DB 接続確認および cron タスク健全性チェックを行い、サービスがリクエスト受付可能かを返す。
/// CRITICAL-003 監査対応: Docker Compose の healthcheck および Kubernetes readinessProbe として使用する。
/// ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一し、timestamp フィールドを追加する。
/// H-006 監査対応: cron リセットタスクの健全性フラグも確認し、停止時は 503 を返してプロセス再起動を促す。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();

    // H-006 監査対応: cron リセットタスクの健全性を Relaxed 順序で読み取る
    let cron_ok = state.cron_healthy.load(Ordering::Relaxed);

    // DB 接続確認（CRITICAL-003 対応）
    let db_check: Result<&str, String> = if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => Ok("ok"),
            Err(e) => Err(format!("error: {}", e)),
        }
    } else {
        Ok("not_configured")
    };

    // DB と cron の両方が正常な場合のみ healthy とみなす
    let overall_healthy = db_check.is_ok() && cron_ok;
    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let db_value = match &db_check {
        Ok(s) => s.to_string(),
        Err(e) => e.clone(),
    };

    (
        status_code,
        Json(serde_json::json!({
            // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
            "status": if overall_healthy { "healthy" } else { "unhealthy" },
            "checks": {
                "database": db_value,
                // H-006 監査対応: cron リセットタスクの健全性チェック結果
                "cron_reset": if cron_ok { "ok" } else { "stopped" }
            },
            "timestamp": timestamp
        })),
    )
        .into_response()
}
