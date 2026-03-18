// k1s0-rule-engine-server の router 初期化 smoke test。
// healthz/readyz の疎通確認と、認証なしでの保護エンドポイントアクセスを検証する。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, Utc};
use tower::ServiceExt;
use uuid::Uuid;

use k1s0_rule_engine_server::adapter::handler::{router, AppState};
use k1s0_rule_engine_server::domain::entity::rule::{EvaluationLog, Rule, RuleSet, RuleSetVersion};
use k1s0_rule_engine_server::domain::repository::{
    EvaluationLogRepository, RuleRepository, RuleSetRepository, RuleSetVersionRepository,
};
use k1s0_rule_engine_server::usecase::{
    CreateRuleSetUseCase, CreateRuleUseCase, DeleteRuleSetUseCase, DeleteRuleUseCase,
    EvaluateUseCase, GetRuleSetUseCase, GetRuleUseCase, ListEvaluationLogsUseCase,
    ListRuleSetsUseCase, ListRulesUseCase, PublishRuleSetUseCase, RollbackRuleSetUseCase,
    UpdateRuleSetUseCase, UpdateRuleUseCase,
};

// --- テストダブル: RuleRepository のスタブ実装 ---

struct StubRuleRepository;

#[async_trait]
impl RuleRepository for StubRuleRepository {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<Rule>> {
        Ok(None)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Rule>> {
        Ok(vec![])
    }

    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _rule_set_id: Option<Uuid>,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)> {
        Ok((vec![], 0))
    }

    async fn create(&self, _rule: &Rule) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update(&self, _rule: &Rule) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn exists_by_name(&self, _name: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn find_by_ids(&self, _ids: &[Uuid]) -> anyhow::Result<Vec<Rule>> {
        Ok(vec![])
    }
}

// --- テストダブル: RuleSetRepository のスタブ実装 ---

struct StubRuleSetRepository;

#[async_trait]
impl RuleSetRepository for StubRuleSetRepository {
    async fn find_by_id(&self, _id: &Uuid) -> anyhow::Result<Option<RuleSet>> {
        Ok(None)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>> {
        Ok(vec![])
    }

    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)> {
        Ok((vec![], 0))
    }

    async fn find_by_domain_and_name(
        &self,
        _domain: &str,
        _name: &str,
    ) -> anyhow::Result<Option<RuleSet>> {
        Ok(None)
    }

    async fn create(&self, _rule_set: &RuleSet) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update(&self, _rule_set: &RuleSet) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _id: &Uuid) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn exists_by_name(&self, _name: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

// --- テストダブル: RuleSetVersionRepository のスタブ実装 ---

struct StubRuleSetVersionRepository;

#[async_trait]
impl RuleSetVersionRepository for StubRuleSetVersionRepository {
    async fn find_by_rule_set_id_and_version(
        &self,
        _rule_set_id: &Uuid,
        _version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        Ok(None)
    }

    async fn find_latest_by_rule_set_id(
        &self,
        _rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        Ok(None)
    }

    async fn create(&self, _version: &RuleSetVersion) -> anyhow::Result<()> {
        Ok(())
    }
}

// --- テストダブル: EvaluationLogRepository のスタブ実装 ---

struct StubEvaluationLogRepository;

#[async_trait]
impl EvaluationLogRepository for StubEvaluationLogRepository {
    async fn create(&self, _log: &EvaluationLog) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_all_paginated(
        &self,
        _page: u32,
        _page_size: u32,
        _rule_set_name: Option<String>,
        _domain: Option<String>,
        _from: Option<DateTime<Utc>>,
        _to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)> {
        Ok((vec![], 0))
    }
}

// --- テストアプリケーション構築 ---

/// スタブリポジトリを使って AppState と Router を構築するヘルパー。
fn make_test_app() -> axum::Router {
    let rule_repo: Arc<dyn RuleRepository> = Arc::new(StubRuleRepository);
    let rule_set_repo: Arc<dyn RuleSetRepository> = Arc::new(StubRuleSetRepository);
    let version_repo: Arc<dyn RuleSetVersionRepository> = Arc::new(StubRuleSetVersionRepository);
    let eval_log_repo: Arc<dyn EvaluationLogRepository> = Arc::new(StubEvaluationLogRepository);

    let state = AppState {
        create_rule_uc: Arc::new(CreateRuleUseCase::new(rule_repo.clone())),
        get_rule_uc: Arc::new(GetRuleUseCase::new(rule_repo.clone())),
        list_rules_uc: Arc::new(ListRulesUseCase::new(rule_repo.clone())),
        update_rule_uc: Arc::new(UpdateRuleUseCase::new(rule_repo.clone())),
        delete_rule_uc: Arc::new(DeleteRuleUseCase::new(rule_repo.clone())),
        create_rule_set_uc: Arc::new(CreateRuleSetUseCase::new(rule_set_repo.clone())),
        get_rule_set_uc: Arc::new(GetRuleSetUseCase::new(rule_set_repo.clone())),
        list_rule_sets_uc: Arc::new(ListRuleSetsUseCase::new(rule_set_repo.clone())),
        update_rule_set_uc: Arc::new(UpdateRuleSetUseCase::new(rule_set_repo.clone())),
        delete_rule_set_uc: Arc::new(DeleteRuleSetUseCase::new(rule_set_repo.clone())),
        publish_rule_set_uc: Arc::new(PublishRuleSetUseCase::new(
            rule_set_repo.clone(),
            version_repo.clone(),
        )),
        rollback_rule_set_uc: Arc::new(RollbackRuleSetUseCase::new(rule_set_repo, version_repo)),
        evaluate_uc: Arc::new(EvaluateUseCase::new(
            Arc::new(StubRuleSetRepository),
            rule_repo,
            eval_log_repo.clone(),
        )),
        list_evaluation_logs_uc: Arc::new(ListEvaluationLogsUseCase::new(eval_log_repo)),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
            "k1s0-rule-engine-server-test",
        )),
        auth_state: None,
    };
    router(state)
}

// --- テスト: /healthz と /readyz が 200 を返す ---

#[tokio::test]
async fn test_healthz_and_readyz() {
    let app = make_test_app();

    // /healthz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // /readyz への GET リクエストで 200 OK を確認
    let req = Request::builder()
        .uri("/readyz")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- テスト: 認証なしモード（auth_state=None）では保護エンドポイントに直接アクセス可能 ---

#[tokio::test]
async fn test_api_accessible_without_auth() {
    let app = make_test_app();

    // /api/v1/rules への GET が 200 を返す（認証なしモード）
    let req = Request::builder()
        .uri("/api/v1/rules")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
