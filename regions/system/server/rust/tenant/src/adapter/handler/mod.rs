pub mod health;
pub mod tenant_handler;

pub use tenant_handler::AppState;

use axum::routing::get;
use axum::Router;

/// REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(tenant_handler::healthz))
        .route("/readyz", get(tenant_handler::readyz))
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
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::domain::repository::tenant_repository::MockTenantRepository;
    use std::sync::Arc;

    fn make_app_state(mock: MockTenantRepository) -> AppState {
        let repo = Arc::new(mock);
        AppState {
            create_tenant_uc: Arc::new(crate::usecase::CreateTenantUseCase::new(repo.clone())),
            get_tenant_uc: Arc::new(crate::usecase::GetTenantUseCase::new(repo.clone())),
            list_tenants_uc: Arc::new(crate::usecase::ListTenantsUseCase::new(repo)),
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
