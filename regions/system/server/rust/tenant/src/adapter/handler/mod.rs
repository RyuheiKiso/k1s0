pub mod health;
pub mod tenant_handler;

pub use tenant_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::require_permission;

/// REST API router.
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(tenant_handler::healthz))
        .route("/readyz", get(tenant_handler::readyz))
        .route("/metrics", get(metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> tenants/read
        let read_routes = Router::new()
            .route(
                "/api/v1/tenants",
                get(tenant_handler::list_tenants),
            )
            .route(
                "/api/v1/tenants/:id",
                get(tenant_handler::get_tenant),
            )
            .route(
                "/api/v1/tenants/:id/members",
                get(tenant_handler::list_members),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "tenants", "read",
            )));

        // POST/PUT/members -> tenants/write
        let write_routes = Router::new()
            .route(
                "/api/v1/tenants",
                post(tenant_handler::create_tenant),
            )
            .route(
                "/api/v1/tenants/:id",
                axum::routing::put(tenant_handler::update_tenant),
            )
            .route(
                "/api/v1/tenants/:id/members",
                post(tenant_handler::add_member),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "tenants", "write",
            )));

        // DELETE/suspend/activate -> tenants/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/tenants/:id",
                delete(tenant_handler::delete_tenant),
            )
            .route(
                "/api/v1/tenants/:id/suspend",
                post(tenant_handler::suspend_tenant),
            )
            .route(
                "/api/v1/tenants/:id/activate",
                post(tenant_handler::activate_tenant),
            )
            .route(
                "/api/v1/tenants/:id/members/:user_id",
                delete(tenant_handler::remove_member),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "tenants", "admin",
            )));

        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）
        Router::new()
            .route(
                "/api/v1/tenants",
                get(tenant_handler::list_tenants).post(tenant_handler::create_tenant),
            )
            .route(
                "/api/v1/tenants/:id",
                get(tenant_handler::get_tenant)
                    .put(tenant_handler::update_tenant)
                    .delete(tenant_handler::delete_tenant),
            )
            .route(
                "/api/v1/tenants/:id/suspend",
                post(tenant_handler::suspend_tenant),
            )
            .route(
                "/api/v1/tenants/:id/activate",
                post(tenant_handler::activate_tenant),
            )
            .route(
                "/api/v1/tenants/:id/members",
                get(tenant_handler::list_members).post(tenant_handler::add_member),
            )
            .route(
                "/api/v1/tenants/:id/members/:user_id",
                delete(tenant_handler::remove_member),
            )
    };

    public_routes
        .merge(api_routes)
        .with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::domain::repository::member_repository::MockMemberRepository;
    use crate::domain::repository::tenant_repository::MockTenantRepository;
    use std::sync::Arc;

    fn make_app_state(mock: MockTenantRepository) -> AppState {
        let repo = Arc::new(mock);
        let member_repo = Arc::new(MockMemberRepository::new());
        AppState {
            create_tenant_uc: Arc::new(crate::usecase::CreateTenantUseCase::new(repo.clone())),
            get_tenant_uc: Arc::new(crate::usecase::GetTenantUseCase::new(repo.clone())),
            list_tenants_uc: Arc::new(crate::usecase::ListTenantsUseCase::new(repo.clone())),
            update_tenant_uc: Arc::new(crate::usecase::UpdateTenantUseCase::new(repo.clone())),
            delete_tenant_uc: Arc::new(crate::usecase::DeleteTenantUseCase::new(repo.clone())),
            suspend_tenant_uc: Arc::new(crate::usecase::SuspendTenantUseCase::new(repo.clone())),
            activate_tenant_uc: Arc::new(crate::usecase::ActivateTenantUseCase::new(repo)),
            list_members_uc: Arc::new(crate::usecase::ListMembersUseCase::new(member_repo.clone())),
            add_member_uc: Arc::new(crate::usecase::AddMemberUseCase::new(member_repo.clone())),
            remove_member_uc: Arc::new(crate::usecase::RemoveMemberUseCase::new(member_repo)),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-tenant-server-test")),
            auth_state: None,
        }
    }

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(MockTenantRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
