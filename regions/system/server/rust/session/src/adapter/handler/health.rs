use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::adapter::handler::session_handler::AppState;
use crate::error::SessionError;
use crate::usecase::get_session::GetSessionInput;

/// ヘルスチェックエンドポイント: DB 接続確認を行い、障害時は 503 を返す
///
/// INFRA-03 監査対応: metadata_repo.health_check() で PostgreSQL 接続を確認する。
/// Noop リポジトリ（in-memory 動作時）は常に Ok を返すため影響なし。
pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    // INFRA-03 監査対応: DB 接続確認を追加し、DB 障害時は 503 を返す
    match state.metadata_repo.health_check().await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!(error = %e, "DB ヘルスチェックに失敗しました");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"status": "error", "service": "session", "detail": "database unavailable"})),
            )
                .into_response();
        }
    }
    Json(serde_json::json!({"status": "ok", "service": "session"})).into_response()
}

/// レディネスチェックエンドポイント: 各バックエンドの疎通確認結果を返す。
/// Redis 未構成（in-memory フォールバック）時は degraded ステータスを返して
/// 永続化バックエンド未構成であることを運用チームに明示する。
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // Redis 疎通確認: ダミーの get を発行してエラーの種類で判定
    let redis_check = state
        .get_uc
        .execute(&GetSessionInput {
            id: Some(format!("readyz-{}", uuid::Uuid::new_v4())),
            token: None,
        })
        .await;
    let redis_ok = !matches!(redis_check, Err(SessionError::Internal(_)));
    let redis_status = if redis_ok { "ok" } else { "error" };

    // PostgreSQL メタデータリポジトリの疎通確認
    let db_ok = state.metadata_repo.health_check().await.is_ok();
    let db_status = if db_ok { "ok" } else { "error" };

    // Kafka イベントパブリッシャーの疎通確認
    let kafka_ok = state.event_publisher.health_check().await.is_ok();
    let kafka_status = if kafka_ok { "ok" } else { "error" };

    let all_backends_ok = redis_ok && db_ok && kafka_ok;

    // ADR-0068 対応: "ready"/"not_ready" から "healthy"/"unhealthy" に統一する
    // in-memory フォールバック時は degraded を返す（ADR-0068 仕様上 200 OK を維持）
    let status = if !state.redis_configured {
        "degraded"
    } else if all_backends_ok {
        "healthy"
    } else {
        "unhealthy"
    };

    // in-memory 時も 200 を返す（サービス自体は動作可能）。バックエンド障害時は 503。
    let code = if !state.redis_configured || all_backends_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    // バックエンド種別を応答に含める
    let backend = if state.redis_configured {
        "redis"
    } else {
        "in-memory"
    };

    // ADR-0068: UTC タイムスタンプを ISO 8601 形式で返す
    let timestamp = chrono::Utc::now().to_rfc3339();
    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "session",
            "backend": backend,
            "checks": {
                "redis": redis_status,
                "postgresql": db_status,
                "kafka": kafka_status
            },
            "timestamp": timestamp
        })),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::SessionRepository;
    use crate::infrastructure::kafka_producer::NoopSessionEventPublisher;
    use crate::usecase::{
        CreateSessionUseCase, GetSessionUseCase, ListUserSessionsUseCase, RefreshSessionUseCase,
        RevokeAllSessionsUseCase, RevokeSessionUseCase,
    };

    /// テスト用の in-memory セッションリポジトリ
    struct InMemoryRepo;

    #[async_trait]
    impl SessionRepository for InMemoryRepo {
        async fn save(&self, _session: &Session) -> Result<(), SessionError> {
            Ok(())
        }
        async fn find_by_id(&self, _id: &str) -> Result<Option<Session>, SessionError> {
            Ok(None)
        }
        async fn find_by_token(&self, _token: &str) -> Result<Option<Session>, SessionError> {
            Ok(None)
        }
        async fn find_by_user_id(&self, _user_id: &str) -> Result<Vec<Session>, SessionError> {
            Ok(vec![])
        }
        async fn delete(&self, _id: &str) -> Result<(), SessionError> {
            Ok(())
        }
    }

    /// テスト用の AppState を構築する（in-memory バックエンド）
    fn test_state() -> AppState {
        let repo: Arc<dyn SessionRepository> = Arc::new(InMemoryRepo);
        let publisher = Arc::new(NoopSessionEventPublisher);
        AppState {
            create_uc: Arc::new(CreateSessionUseCase::new(
                repo.clone(),
                Arc::new(NoopSessionMetadataRepository),
                publisher.clone(),
                3600,
                86400,
            )),
            get_uc: Arc::new(GetSessionUseCase::new(repo.clone())),
            refresh_uc: Arc::new(RefreshSessionUseCase::new(repo.clone(), 86400)),
            revoke_uc: Arc::new(RevokeSessionUseCase::new(
                repo.clone(),
                Arc::new(NoopSessionMetadataRepository),
                publisher.clone(),
            )),
            list_uc: Arc::new(ListUserSessionsUseCase::new(repo.clone())),
            revoke_all_uc: Arc::new(RevokeAllSessionsUseCase::new(
                repo,
                Arc::new(NoopSessionMetadataRepository),
            )),
            metadata_repo: Arc::new(NoopSessionMetadataRepository),
            event_publisher: publisher,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("session-test")),
            auth_state: None,
            redis_configured: false,
        }
    }

    #[tokio::test]
    async fn healthz_check() {
        // INFRA-03 対応: healthz が State を受け取るようになったため with_state を追加
        let app = Router::new()
            .route("/healthz", get(super::healthz))
            .with_state(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "session");
    }

    #[tokio::test]
    async fn readyz_check_in_memory_returns_degraded() {
        let app = Router::new()
            .route("/readyz", get(super::readyz))
            .with_state(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // in-memory バックエンド時は 200 OK だが degraded ステータス
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["service"], "session");
        assert_eq!(json["backend"], "in-memory");
        assert_eq!(json["checks"]["redis"], "ok");
        assert_eq!(json["checks"]["postgresql"], "ok");
        assert_eq!(json["checks"]["kafka"], "ok");
    }
}
