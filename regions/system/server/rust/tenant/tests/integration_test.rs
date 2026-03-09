use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_tenant_server::adapter::handler::{self, AppState};
use k1s0_tenant_server::domain::repository::member_repository::MockMemberRepository;
use k1s0_tenant_server::domain::repository::tenant_repository::MockTenantRepository;
use k1s0_tenant_server::usecase;

fn make_app_state() -> AppState {
    let repo = Arc::new(MockTenantRepository::new());
    let member_repo = Arc::new(MockMemberRepository::new());
    AppState {
        create_tenant_uc: Arc::new(usecase::CreateTenantUseCase::new(repo.clone())),
        get_tenant_uc: Arc::new(usecase::GetTenantUseCase::new(repo.clone())),
        list_tenants_uc: Arc::new(usecase::ListTenantsUseCase::new(repo.clone())),
        update_tenant_uc: Arc::new(usecase::UpdateTenantUseCase::new(repo.clone())),
        delete_tenant_uc: Arc::new(usecase::DeleteTenantUseCase::new(repo.clone())),
        suspend_tenant_uc: Arc::new(usecase::SuspendTenantUseCase::new(repo.clone())),
        activate_tenant_uc: Arc::new(usecase::ActivateTenantUseCase::new(repo)),
        list_members_uc: Arc::new(usecase::ListMembersUseCase::new(member_repo.clone())),
        add_member_uc: Arc::new(usecase::AddMemberUseCase::new(member_repo.clone())),
        remove_member_uc: Arc::new(usecase::RemoveMemberUseCase::new(member_repo)),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-tenant-server-test",
        )),
        auth_state: None,
        db_pool: None,
        kafka_brokers: None,
        keycloak_health_url: None,
        http_client: reqwest::Client::new(),
    }
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let state = make_app_state();
    let app = handler::router(state);

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_returns_status() {
    let state = make_app_state();
    let app = handler::router(state);

    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
