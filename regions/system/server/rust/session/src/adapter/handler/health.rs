use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::adapter::handler::session_handler::AppState;
use crate::error::SessionError;
use crate::usecase::get_session::GetSessionInput;

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok", "service": "session"}))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let redis_check = state
        .get_uc
        .execute(&GetSessionInput {
            id: Some(format!("readyz-{}", uuid::Uuid::new_v4())),
            token: None,
        })
        .await;
    let redis_ok = !matches!(redis_check, Err(SessionError::Internal(_)));
    let redis_status = if redis_ok { "ok" } else { "error" };

    let db_ok = state.metadata_repo.health_check().await.is_ok();
    let db_status = if db_ok { "ok" } else { "error" };

    let kafka_ok = state.event_publisher.health_check().await.is_ok();
    let kafka_status = if kafka_ok { "ok" } else { "error" };

    let ready = redis_ok && db_ok && kafka_ok;
    let status = if ready { "ready" } else { "not_ready" };
    let code = if ready {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "session",
            "checks": {
                "redis": redis_status,
                "postgresql": db_status,
                "kafka": kafka_status
            }
        })),
    )
}

#[cfg(test)]
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
            revoke_uc: Arc::new(RevokeSessionUseCase::new(repo.clone(), Arc::new(NoopSessionMetadataRepository), publisher.clone())),
            list_uc: Arc::new(ListUserSessionsUseCase::new(repo.clone())),
            revoke_all_uc: Arc::new(RevokeAllSessionsUseCase::new(repo, Arc::new(NoopSessionMetadataRepository))),
            metadata_repo: Arc::new(NoopSessionMetadataRepository),
            event_publisher: publisher,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("session-test")),
            auth_state: None,
        }
    }

    #[tokio::test]
    async fn healthz_check() {
        let app = Router::new().route("/healthz", get(super::healthz));
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
    async fn readyz_check() {
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

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
        assert_eq!(json["service"], "session");
        assert_eq!(json["checks"]["redis"], "ok");
        assert_eq!(json["checks"]["postgresql"], "ok");
        assert_eq!(json["checks"]["kafka"], "ok");
    }
}
