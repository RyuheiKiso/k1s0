pub mod health;
pub mod vault_handler;

pub use vault_handler::AppState;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::require_permission;
use crate::adapter::middleware::spiffe::spiffe_auth_middleware;

/// REST API router.
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティング
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET/metadata/audit -> secrets/read
        let read_routes = Router::new()
            .route(
                "/api/v1/secrets",
                get(vault_handler::list_secrets),
            )
            .route(
                "/api/v1/secrets/:key",
                get(vault_handler::get_secret),
            )
            .route(
                "/api/v1/secrets/:key/metadata",
                get(vault_handler::get_secret_metadata),
            )
            .route("/api/v1/audit/logs", get(vault_handler::list_audit_logs))
            .route_layer(axum::middleware::from_fn(require_permission(
                "secrets", "read",
            )));

        // POST/PUT/rotate -> secrets/write
        let write_routes = Router::new()
            .route(
                "/api/v1/secrets",
                post(vault_handler::create_secret),
            )
            .route(
                "/api/v1/secrets/:key",
                axum::routing::put(vault_handler::update_secret),
            )
            .route(
                "/api/v1/secrets/:key/rotate",
                post(vault_handler::rotate_secret),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "secrets", "write",
            )));

        // DELETE -> secrets/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/secrets/:key",
                axum::routing::delete(vault_handler::delete_secret),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "secrets", "admin",
            )));

        let merged = Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes);

        // SPIFFE ミドルウェアを RBAC の後、auth の前に適用
        // axum のレイヤー順序: 最後に追加 = リクエスト時に最初に実行
        // 実行順: auth → SPIFFE → RBAC → handler
        let merged = if let Some(ref spiffe_state) = state.spiffe_state {
            merged.layer(axum::middleware::from_fn_with_state(
                spiffe_state.clone(),
                spiffe_auth_middleware,
            ))
        } else {
            merged
        };

        merged.layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）
        Router::new()
            .route(
                "/api/v1/secrets",
                get(vault_handler::list_secrets).post(vault_handler::create_secret),
            )
            .route(
                "/api/v1/secrets/:key",
                get(vault_handler::get_secret)
                    .put(vault_handler::update_secret)
                    .delete(vault_handler::delete_secret),
            )
            .route(
                "/api/v1/secrets/:key/metadata",
                get(vault_handler::get_secret_metadata),
            )
            .route(
                "/api/v1/secrets/:key/rotate",
                post(vault_handler::rotate_secret),
            )
            .route("/api/v1/audit/logs", get(vault_handler::list_audit_logs))
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
