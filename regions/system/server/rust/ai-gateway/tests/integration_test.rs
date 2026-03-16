// AI Gateway サーバーの統合テスト
// router 初期化と healthz エンドポイントの smoke test を行う

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use k1s0_ai_gateway_server::adapter::handler::ai_handler::AppState;
use k1s0_ai_gateway_server::adapter::handler::router;
use k1s0_ai_gateway_server::domain::entity::model::AiModel;
use k1s0_ai_gateway_server::domain::entity::routing_rule::RoutingRule;
use k1s0_ai_gateway_server::domain::entity::usage_record::UsageRecord;
use k1s0_ai_gateway_server::domain::repository::{
    ModelRepository, RoutingRuleRepository, UsageRepository,
};
use k1s0_ai_gateway_server::domain::service::guardrail_service::GuardrailService;
use k1s0_ai_gateway_server::domain::service::routing_service::RoutingService;
use k1s0_ai_gateway_server::infrastructure::llm_client::LlmClient;
use k1s0_ai_gateway_server::usecase::{
    CompleteUseCase, EmbedUseCase, GetUsageUseCase, ListModelsUseCase,
};

// --- テストダブル ---

/// スタブ: モデルリポジトリ（空の結果を返す）
struct StubModelRepository;

#[async_trait::async_trait]
impl ModelRepository for StubModelRepository {
    async fn find_all(&self) -> Vec<AiModel> {
        vec![]
    }

    async fn find_by_id(&self, _id: &str) -> Option<AiModel> {
        None
    }
}

/// スタブ: ルーティングルールリポジトリ（空の結果を返す）
struct StubRoutingRuleRepository;

#[async_trait::async_trait]
impl RoutingRuleRepository for StubRoutingRuleRepository {
    async fn find_active_rule(&self, _model_id: &str) -> Option<RoutingRule> {
        None
    }
}

/// スタブ: 使用量リポジトリ（空の結果を返す）
struct StubUsageRepository;

#[async_trait::async_trait]
impl UsageRepository for StubUsageRepository {
    async fn save(&self, _record: &UsageRecord) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_tenant(
        &self,
        _tenant_id: &str,
        _start: &str,
        _end: &str,
    ) -> Vec<UsageRecord> {
        vec![]
    }
}

/// テスト用の AppState とルーターを構築する
fn make_test_app() -> axum::Router {
    // リポジトリのスタブを作成
    let model_repo: Arc<dyn ModelRepository> = Arc::new(StubModelRepository);
    let routing_rule_repo: Arc<dyn RoutingRuleRepository> = Arc::new(StubRoutingRuleRepository);
    let usage_repo: Arc<dyn UsageRepository> = Arc::new(StubUsageRepository);

    // ドメインサービスを生成
    let guardrail = Arc::new(GuardrailService::new());
    let routing = Arc::new(RoutingService::new(
        model_repo.clone(),
        routing_rule_repo,
    ));

    // LLMクライアント（テスト用のダミーURL）
    let llm_client = Arc::new(LlmClient::new(
        "http://localhost:0".to_string(),
        "test-api-key".to_string(),
    ));

    // 各ユースケースを生成
    let complete_uc = Arc::new(CompleteUseCase::new(
        guardrail,
        routing,
        llm_client.clone(),
        usage_repo.clone(),
    ));
    let embed_uc = Arc::new(EmbedUseCase::new(llm_client));
    let list_models_uc = Arc::new(ListModelsUseCase::new(model_repo));
    let get_usage_uc = Arc::new(GetUsageUseCase::new(usage_repo));

    // メトリクスを生成
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("ai-gateway-test"));

    // AppState を構築（認証なし = テストモード）
    let state = AppState {
        complete_uc,
        embed_uc,
        list_models_uc,
        get_usage_uc,
        metrics,
        auth_state: None,
    };

    // ルーターを構築（メトリクス無効）
    router(state, false, "/metrics")
}

// --- 統合テスト ---

/// /healthz と /readyz エンドポイントが 200 OK を返すことを確認する
#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz のテスト
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/healthz は 200 を返すべき");

    // /readyz のテスト
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "/readyz は 200 を返すべき");
}
