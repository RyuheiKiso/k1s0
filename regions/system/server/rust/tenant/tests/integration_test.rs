// テナントサーバーの統合テスト
// router 初期化の smoke test として、ヘルスチェックと認証なしアクセスを検証する

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

// テナントサーバーのクレートから必要な型をインポート
use k1s0_tenant_server::adapter::handler::{router, AppState};
use k1s0_tenant_server::domain::repository::member_repository::MockMemberRepository;
use k1s0_tenant_server::domain::repository::tenant_repository::MockTenantRepository;
use k1s0_tenant_server::usecase;

// --- テスト用アプリケーション構築 ---

/// テスト用の AppState を構築し、router を返すヘルパー関数。
/// 全リポジトリにモックを使用し、認証は無効化する。
fn make_test_app() -> axum::Router {
    // モックリポジトリの生成
    let tenant_repo = Arc::new(MockTenantRepository::new());
    let member_repo = Arc::new(MockMemberRepository::new());

    // AppState の構築（認証なし、DB なし）
    let state = AppState {
        create_tenant_uc: Arc::new(usecase::CreateTenantUseCase::new(tenant_repo.clone())),
        get_tenant_uc: Arc::new(usecase::GetTenantUseCase::new(tenant_repo.clone())),
        list_tenants_uc: Arc::new(usecase::ListTenantsUseCase::new(tenant_repo.clone())),
        update_tenant_uc: Arc::new(usecase::UpdateTenantUseCase::new(tenant_repo.clone())),
        delete_tenant_uc: Arc::new(usecase::DeleteTenantUseCase::new(tenant_repo.clone())),
        suspend_tenant_uc: Arc::new(usecase::SuspendTenantUseCase::new(tenant_repo.clone())),
        activate_tenant_uc: Arc::new(usecase::ActivateTenantUseCase::new(tenant_repo.clone())),
        list_members_uc: Arc::new(usecase::ListMembersUseCase::new(member_repo.clone())),
        add_member_uc: Arc::new(usecase::AddMemberUseCase::new(member_repo.clone())),
        remove_member_uc: Arc::new(usecase::RemoveMemberUseCase::new(member_repo.clone())),
        update_member_role_uc: Arc::new(usecase::UpdateMemberRoleUseCase::new(
            member_repo,
            tenant_repo,
        )),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-tenant-server-test",
        )),
        auth_state: None,
        db_pool: None,
        kafka_brokers: None,
        keycloak_health_url: None,
        http_client: reqwest::Client::new(),
    };

    router(state)
}

// --- ヘルスチェックテスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz へのリクエスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz へのリクエスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- 認証なしアクセステスト ---

/// 認証が無効な状態で保護エンドポイントにアクセスすると正常にルーティングされることを確認する。
/// auth_state が None の場合、認証ミドルウェアはスキップされるため 200 以外のステータスが返る可能性がある。
/// ここではルーターが正しく初期化されていること（パニックしないこと）を検証する。
#[tokio::test]
async fn test_api_routes_are_reachable() {
    let app = make_test_app();

    // 認証なしモードでは /api/v1/tenants にアクセスできる（リポジトリが空なので結果は空）
    let req = Request::builder()
        .uri("/api/v1/tenants?page=1&page_size=10")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // ルーターが正常に応答すること（500 でないこと）を確認
    assert_ne!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
