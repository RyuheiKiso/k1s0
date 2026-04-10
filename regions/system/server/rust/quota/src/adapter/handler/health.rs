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
/// ADR-0068 対応: "`ready"/"not_ready`" から "healthy"/"unhealthy" に統一し、timestamp フィールドを追加する。
/// MED-011 監査対応: cron リセットタスクの停止は補助機能の障害であり、サービス全体の停止には相当しない。
///   - DB 接続が正常 + cron 正常 → 200 "healthy"
///   - DB 接続が正常 + cron 停止 → 200 "degraded"（K8s は Pod を再起動しないが Prometheus でアラートを設定すること）
///   - DB 接続が失敗             → 503 "unhealthy"（K8s が Pod を再起動する）
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();

    // cron リセットタスクの健全性を Relaxed 順序で読み取る
    let cron_ok = state.cron_healthy.load(Ordering::Relaxed);

    // DB 接続確認（CRITICAL-003 対応）: DB 障害のみが真の unhealthy である
    let db_check: Result<&str, String> = if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => Ok("ok"),
            Err(e) => Err(format!("error: {e}")),
        }
    } else {
        Ok("not_configured")
    };

    let db_ok = db_check.is_ok();

    // DB 障害のみ 503（K8s Pod 再起動対象）
    // cron 停止は 200 + degraded として Prometheus アラートで監視する
    let (status_code, status_str) = match (db_ok, cron_ok) {
        (true, true) => (StatusCode::OK, "healthy"),
        (true, false) => (StatusCode::OK, "degraded"),
        (false, _) => (StatusCode::SERVICE_UNAVAILABLE, "unhealthy"),
    };

    let db_value = match &db_check {
        Ok(s) => s.to_string(),
        Err(e) => e.clone(),
    };

    (
        status_code,
        Json(serde_json::json!({
            "status": status_str,
            "checks": {
                "database": db_value,
                // cron 停止時は "stopped" を返す（degraded 状態の詳細情報）
                "cron_reset": if cron_ok { "ok" } else { "stopped" }
            },
            "timestamp": timestamp
        })),
    )
        .into_response()
}
