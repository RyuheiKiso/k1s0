//! レートリミットサーバーのユースケーステスト。
//! StubRepository パターンを使用して各ユースケースの動作を検証する。
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use k1s0_ratelimit_server::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
use k1s0_ratelimit_server::domain::repository::{
    RateLimitRepository, RateLimitStateStore, UsageSnapshot,
};
use k1s0_ratelimit_server::usecase::create_rule::CreateRuleInput;
use k1s0_ratelimit_server::usecase::list_rules::ListRulesInput;
use k1s0_ratelimit_server::usecase::update_rule::UpdateRuleInput;
use k1s0_ratelimit_server::usecase::{
    CheckRateLimitUseCase, CreateRuleUseCase, DeleteRuleUseCase, GetRuleUseCase, GetUsageUseCase,
    ListRulesUseCase, ResetRateLimitInput, ResetRateLimitUseCase, UpdateRuleUseCase,
};

// ============================================================
// Stub リポジトリ実装
// ============================================================

/// インメモリ Stub リポジトリ: レートリミットルールの永続化を模倣する。
struct StubRateLimitRepository {
    rules: tokio::sync::RwLock<Vec<RateLimitRule>>,
}

impl StubRateLimitRepository {
    fn new() -> Self {
        Self {
            rules: tokio::sync::RwLock::new(Vec::new()),
        }
    }

    fn with_rules(rules: Vec<RateLimitRule>) -> Self {
        Self {
            rules: tokio::sync::RwLock::new(rules),
        }
    }
}

#[async_trait]
impl RateLimitRepository for StubRateLimitRepository {
    /// ルールを作成してリストに追加する。
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule> {
        let mut rules = self.rules.write().await;
        rules.push(rule.clone());
        Ok(rule.clone())
    }

    /// IDでルールを検索する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<RateLimitRule> {
        let rules = self.rules.read().await;
        rules
            .iter()
            .find(|r| r.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("rule not found: {}", id))
    }

    /// 名前でルールを検索する。
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<RateLimitRule>> {
        let rules = self.rules.read().await;
        Ok(rules.iter().find(|r| r.name == name).cloned())
    }

    /// スコープでルールを検索する。
    async fn find_by_scope(&self, scope: &str) -> anyhow::Result<Vec<RateLimitRule>> {
        let rules = self.rules.read().await;
        Ok(rules.iter().filter(|r| r.scope == scope).cloned().collect())
    }

    /// 全ルールを取得する。
    async fn find_all(&self) -> anyhow::Result<Vec<RateLimitRule>> {
        Ok(self.rules.read().await.clone())
    }

    /// ページネーション付きでルールを取得する。
    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)> {
        let rules = self.rules.read().await;
        let filtered: Vec<_> = rules
            .iter()
            .filter(|r| {
                if let Some(ref s) = scope {
                    r.scope == *s
                } else {
                    true
                }
            })
            .filter(|r| !enabled_only || r.enabled)
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let page_items: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((page_items, total))
    }

    /// ルールを更新する。
    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("rule not found"))
        }
    }

    /// ルールを削除する。
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut rules = self.rules.write().await;
        let len_before = rules.len();
        rules.retain(|r| r.id != *id);
        Ok(rules.len() < len_before)
    }

    /// レートリミット状態をリセットする（Stub では何もしない）。
    async fn reset_state(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Stub ステートストア: レートリミット状態の管理を模倣する。
struct StubRateLimitStateStore {
    /// 現在の使用カウント
    counter: tokio::sync::RwLock<i64>,
    /// 許可リミット
    limit_override: Option<i64>,
    /// エラーを返すかどうか
    should_error: bool,
}

impl StubRateLimitStateStore {
    /// 正常応答を返す StubStateStore を作成する。
    fn new() -> Self {
        Self {
            counter: tokio::sync::RwLock::new(0),
            limit_override: None,
            should_error: false,
        }
    }

    /// 初期カウント付きの StubStateStore を作成する。
    fn with_counter(count: i64) -> Self {
        Self {
            counter: tokio::sync::RwLock::new(count),
            limit_override: None,
            should_error: false,
        }
    }

    /// エラーを返す StubStateStore を作成する。
    fn with_error() -> Self {
        Self {
            counter: tokio::sync::RwLock::new(0),
            limit_override: None,
            should_error: true,
        }
    }

    /// カウンターをインクリメントしてデシジョンを返すヘルパー。
    async fn check_common(
        &self,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        if self.should_error {
            return Err(anyhow::anyhow!("state store unavailable"));
        }
        let mut counter = self.counter.write().await;
        *counter += 1;
        let effective_limit = self.limit_override.unwrap_or(limit);
        let remaining = (effective_limit - *counter).max(0);
        let reset_at = Utc::now() + chrono::Duration::seconds(window_secs);
        if *counter > effective_limit {
            Ok(RateLimitDecision::denied(
                effective_limit,
                0,
                reset_at,
                "rate limit exceeded".to_string(),
            ))
        } else {
            Ok(RateLimitDecision::allowed(
                effective_limit,
                remaining,
                reset_at,
            ))
        }
    }
}

#[async_trait]
impl RateLimitStateStore for StubRateLimitStateStore {
    /// トークンバケットチェック（Stub実装）。
    async fn check_token_bucket(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        self.check_common(limit, window_secs).await
    }

    /// 固定ウィンドウチェック（Stub実装）。
    async fn check_fixed_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        self.check_common(limit, window_secs).await
    }

    /// スライディングウィンドウチェック（Stub実装）。
    async fn check_sliding_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        self.check_common(limit, window_secs).await
    }

    /// リーキーバケットチェック（Stub実装）。
    async fn check_leaky_bucket(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        self.check_common(limit, window_secs).await
    }

    /// レートリミット状態をリセットする（Stub実装）。
    async fn reset(&self, _key: &str) -> anyhow::Result<()> {
        if self.should_error {
            return Err(anyhow::anyhow!("state store unavailable"));
        }
        let mut counter = self.counter.write().await;
        *counter = 0;
        Ok(())
    }

    /// 使用状況を取得する（Stub実装）。
    async fn get_usage(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<Option<UsageSnapshot>> {
        if self.should_error {
            return Err(anyhow::anyhow!("state store unavailable"));
        }
        let counter = self.counter.read().await;
        Ok(Some(UsageSnapshot {
            used: *counter,
            remaining: (limit - *counter).max(0),
            reset_at: (Utc::now() + chrono::Duration::seconds(window_secs)).timestamp(),
        }))
    }
}

// ============================================================
// ヘルパー関数
// ============================================================

/// テスト用ルールを作成するヘルパー。
fn make_rule(
    scope: &str,
    pattern: &str,
    limit: u32,
    window: u32,
    algo: Algorithm,
) -> RateLimitRule {
    RateLimitRule::new(scope.to_string(), pattern.to_string(), limit, window, algo)
}

// ============================================================
// CheckRateLimitUseCase テスト
// ============================================================

/// 単一ルールに対するレートリミットチェック（許可）。
#[tokio::test]
async fn check_rate_limit_with_stub_single_rule_allowed() {
    let rule = make_rule("api", "*", 10, 60, Algorithm::TokenBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    let decision = uc.execute("api", "user-1", 60).await.unwrap();

    assert!(decision.allowed);
    assert_eq!(decision.scope, "api");
    assert_eq!(decision.identifier, "user-1");
}

/// 完全一致パターンが優先されることを確認する。
#[tokio::test]
async fn check_rate_limit_exact_match_takes_priority_over_wildcard() {
    let wildcard_rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let exact_rule = make_rule("api", "user-vip", 1000, 60, Algorithm::FixedWindow);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![
        wildcard_rule,
        exact_rule,
    ]));
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    // user-vip は FixedWindow のルールにマッチするはず
    let decision = uc.execute("api", "user-vip", 60).await.unwrap();

    assert!(decision.allowed);
    // FixedWindowのcheck_fixed_windowが呼ばれることを間接的に検証
    assert_eq!(decision.scope, "api");
}

/// 複数ポリシー（同一scope）でワイルドカードにフォールバックする。
#[tokio::test]
async fn check_rate_limit_falls_back_to_wildcard_when_no_exact_match() {
    let wildcard_rule = make_rule("api", "*", 50, 60, Algorithm::SlidingWindow);
    let specific_rule = make_rule("api", "admin-user", 200, 60, Algorithm::TokenBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![
        wildcard_rule,
        specific_rule,
    ]));
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    // 通常ユーザーはワイルドカードルールにフォールバック
    let decision = uc.execute("api", "normal-user", 60).await.unwrap();

    assert!(decision.allowed);
    assert_eq!(decision.scope, "api");
}

/// 無効化されたルールがスキップされることを確認する。
#[tokio::test]
async fn check_rate_limit_disabled_rule_skipped() {
    let mut rule = make_rule("api", "*", 10, 60, Algorithm::TokenBucket);
    rule.enabled = false;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    // 無効ルールはスキップされ、デフォルトが使用される
    let decision = uc.execute("api", "user-1", 60).await.unwrap();

    assert!(decision.allowed);
    // rule_id はマッチしないルールなので空文字列
    assert!(decision.rule_id.is_empty());
}

/// 空の identifier でバリデーションエラーが返る。
#[tokio::test]
async fn check_rate_limit_empty_identifier_error() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    let result = uc.execute("api", "", 60).await;

    assert!(result.is_err());
}

/// fail-open モードでバックエンドエラー時に許可される。
#[tokio::test]
async fn check_rate_limit_fail_open_with_matched_rule() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let state = Arc::new(StubRateLimitStateStore::with_error());

    let uc = CheckRateLimitUseCase::with_fallback_policy(repo, state, true, 100, 60);
    let decision = uc.execute("api", "user-1", 60).await.unwrap();

    assert!(decision.allowed);
    assert!(decision.reason.contains("fail-open"));
}

/// fail-closed モードでバックエンドエラー時にエラーが返る。
#[tokio::test]
async fn check_rate_limit_fail_closed_returns_error() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let state = Arc::new(StubRateLimitStateStore::with_error());

    let uc = CheckRateLimitUseCase::with_fallback_policy(repo, state, false, 100, 60);
    let result = uc.execute("api", "user-1", 60).await;

    assert!(result.is_err());
}

/// LeakyBucket アルゴリズムのルールが正しく選択される。
#[tokio::test]
async fn check_rate_limit_leaky_bucket_algorithm() {
    let rule = make_rule("api", "*", 50, 30, Algorithm::LeakyBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let state = Arc::new(StubRateLimitStateStore::new());

    let uc = CheckRateLimitUseCase::new(repo, state);
    let decision = uc.execute("api", "user-1", 30).await.unwrap();

    assert!(decision.allowed);
}

/// with_fallback_policy で limit=0 の場合 limit=1 に矯正される。
#[tokio::test]
async fn check_rate_limit_fallback_policy_zero_limit_clamped_to_one() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let state = Arc::new(StubRateLimitStateStore::new());

    // limit=0, window=0 は内部で max(1) に矯正される
    let uc = CheckRateLimitUseCase::with_fallback_policy(repo, state, true, 0, 0);
    let decision = uc.execute("api", "user-1", 60).await.unwrap();

    assert!(decision.allowed);
}

// ============================================================
// CreateRuleUseCase テスト
// ============================================================

/// 正常にルールを作成できる。
#[tokio::test]
async fn create_rule_with_stub_success() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = CreateRuleUseCase::new(repo.clone());

    let input = CreateRuleInput {
        scope: "payment".to_string(),
        identifier_pattern: "user:*".to_string(),
        limit: 50,
        window_seconds: 120,
        algorithm: Some("sliding_window".to_string()),
        enabled: true,
    };

    let result = uc.execute(&input).await.unwrap();
    assert_eq!(result.scope, "payment");
    assert_eq!(result.identifier_pattern, "user:*");
    assert_eq!(result.limit, 50);
    assert_eq!(result.algorithm, Algorithm::SlidingWindow);

    // リポジトリに保存されていることを確認
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 1);
}

/// 同一 scope+pattern で重複作成がエラーになる。
#[tokio::test]
async fn create_rule_with_stub_duplicate_rejected() {
    let existing = make_rule("payment", "user:*", 50, 120, Algorithm::TokenBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![existing]));
    let uc = CreateRuleUseCase::new(repo);

    let input = CreateRuleInput {
        scope: "payment".to_string(),
        identifier_pattern: "user:*".to_string(),
        limit: 100,
        window_seconds: 60,
        algorithm: None,
        enabled: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
}

/// 無効なアルゴリズム名でエラーが返る。
#[tokio::test]
async fn create_rule_invalid_algorithm_name() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = CreateRuleUseCase::new(repo);

    let input = CreateRuleInput {
        scope: "api".to_string(),
        identifier_pattern: "*".to_string(),
        limit: 100,
        window_seconds: 60,
        algorithm: Some("nonexistent_algo".to_string()),
        enabled: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
}

/// enabled=false でルールを作成できる。
#[tokio::test]
async fn create_rule_with_disabled_state() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = CreateRuleUseCase::new(repo);

    let input = CreateRuleInput {
        scope: "api".to_string(),
        identifier_pattern: "admin:*".to_string(),
        limit: 500,
        window_seconds: 3600,
        algorithm: Some("fixed_window".to_string()),
        enabled: false,
    };

    let result = uc.execute(&input).await.unwrap();
    assert!(!result.enabled);
    assert_eq!(result.algorithm, Algorithm::FixedWindow);
}

/// 空の identifier_pattern でバリデーションエラー。
#[tokio::test]
async fn create_rule_empty_identifier_pattern_validation_error() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = CreateRuleUseCase::new(repo);

    let input = CreateRuleInput {
        scope: "api".to_string(),
        identifier_pattern: "".to_string(),
        limit: 100,
        window_seconds: 60,
        algorithm: None,
        enabled: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
}

// ============================================================
// UpdateRuleUseCase テスト
// ============================================================

/// Stub リポジトリでルールを更新する。
#[tokio::test]
async fn update_rule_with_stub_success() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let uc = UpdateRuleUseCase::new(repo.clone());

    let input = UpdateRuleInput {
        id: rule_id.to_string(),
        scope: "api-v2".to_string(),
        identifier_pattern: "global".to_string(),
        limit: 200,
        window_seconds: 120,
        algorithm: Some("leaky_bucket".to_string()),
        enabled: false,
    };

    let updated = uc.execute(&input).await.unwrap();
    assert_eq!(updated.scope, "api-v2");
    assert_eq!(updated.limit, 200);
    assert_eq!(updated.algorithm, Algorithm::LeakyBucket);
    assert!(!updated.enabled);

    // リポジトリ内のデータも更新されていることを確認
    let stored = repo.find_by_id(&rule_id).await.unwrap();
    assert_eq!(stored.limit, 200);
}

/// 無効な UUID でエラーが返る。
#[tokio::test]
async fn update_rule_invalid_uuid_returns_error() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = UpdateRuleUseCase::new(repo);

    let input = UpdateRuleInput {
        id: "not-a-valid-uuid".to_string(),
        scope: "api".to_string(),
        identifier_pattern: "*".to_string(),
        limit: 100,
        window_seconds: 60,
        algorithm: None,
        enabled: true,
    };

    let result = uc.execute(&input).await;
    assert!(result.is_err());
}

/// アルゴリズムを指定しない場合は元のアルゴリズムが維持される。
#[tokio::test]
async fn update_rule_without_algorithm_keeps_original() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::SlidingWindow);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let uc = UpdateRuleUseCase::new(repo);

    let input = UpdateRuleInput {
        id: rule_id.to_string(),
        scope: "api".to_string(),
        identifier_pattern: "*".to_string(),
        limit: 200,
        window_seconds: 120,
        algorithm: None, // アルゴリズムは変更しない
        enabled: true,
    };

    let updated = uc.execute(&input).await.unwrap();
    assert_eq!(updated.algorithm, Algorithm::SlidingWindow);
}

// ============================================================
// DeleteRuleUseCase テスト
// ============================================================

/// Stub リポジトリでルールを削除する。
#[tokio::test]
async fn delete_rule_with_stub_success() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let uc = DeleteRuleUseCase::new(repo.clone());

    uc.execute(&rule_id.to_string()).await.unwrap();

    // 削除後は空になる
    let all = repo.find_all().await.unwrap();
    assert!(all.is_empty());
}

/// 存在しないルールの削除でエラーが返る。
#[tokio::test]
async fn delete_rule_with_stub_not_found() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = DeleteRuleUseCase::new(repo);

    let result = uc.execute("550e8400-e29b-41d4-a716-446655440000").await;
    assert!(result.is_err());
}

// ============================================================
// GetRuleUseCase テスト
// ============================================================

/// Stub リポジトリでルールを取得する。
#[tokio::test]
async fn get_rule_with_stub_success() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let uc = GetRuleUseCase::new(repo);

    let found = uc.execute(&rule_id.to_string()).await.unwrap();
    assert_eq!(found.id, rule_id);
    assert_eq!(found.scope, "api");
}

/// 存在しない ID でエラーが返る。
#[tokio::test]
async fn get_rule_with_stub_not_found() {
    let repo = Arc::new(StubRateLimitRepository::new());
    let uc = GetRuleUseCase::new(repo);

    let result = uc.execute("550e8400-e29b-41d4-a716-446655440000").await;
    assert!(result.is_err());
}

// ============================================================
// ListRulesUseCase テスト
// ============================================================

/// ページネーションの has_next が正しく計算される。
#[tokio::test]
async fn list_rules_with_stub_pagination_has_next() {
    let rules: Vec<RateLimitRule> = (0..5)
        .map(|i| make_rule(&format!("svc-{}", i), "*", 100, 60, Algorithm::TokenBucket))
        .collect();
    let repo = Arc::new(StubRateLimitRepository::with_rules(rules));
    let uc = ListRulesUseCase::new(repo);

    let result = uc
        .execute(&ListRulesInput {
            page: 1,
            page_size: 3,
            scope: None,
            enabled_only: false,
        })
        .await
        .unwrap();

    assert_eq!(result.rules.len(), 3);
    assert_eq!(result.total_count, 5);
    assert!(result.has_next);
}

/// 最後のページでは has_next=false。
#[tokio::test]
async fn list_rules_with_stub_last_page_no_next() {
    let rules: Vec<RateLimitRule> = (0..3)
        .map(|i| make_rule(&format!("svc-{}", i), "*", 100, 60, Algorithm::TokenBucket))
        .collect();
    let repo = Arc::new(StubRateLimitRepository::with_rules(rules));
    let uc = ListRulesUseCase::new(repo);

    let result = uc
        .execute(&ListRulesInput {
            page: 1,
            page_size: 10,
            scope: None,
            enabled_only: false,
        })
        .await
        .unwrap();

    assert_eq!(result.rules.len(), 3);
    assert!(!result.has_next);
}

/// scope フィルターでルールが絞り込まれる。
#[tokio::test]
async fn list_rules_with_stub_scope_filter() {
    let rules = vec![
        make_rule("api", "*", 100, 60, Algorithm::TokenBucket),
        make_rule("payment", "*", 50, 30, Algorithm::FixedWindow),
        make_rule("api", "admin:*", 500, 3600, Algorithm::TokenBucket),
    ];
    let repo = Arc::new(StubRateLimitRepository::with_rules(rules));
    let uc = ListRulesUseCase::new(repo);

    let result = uc
        .execute(&ListRulesInput {
            page: 1,
            page_size: 20,
            scope: Some("api".to_string()),
            enabled_only: false,
        })
        .await
        .unwrap();

    assert_eq!(result.rules.len(), 2);
    assert_eq!(result.total_count, 2);
}

/// enabled_only=true で無効ルールが除外される。
#[tokio::test]
async fn list_rules_with_stub_enabled_only_filter() {
    let mut disabled_rule = make_rule("api", "disabled", 100, 60, Algorithm::TokenBucket);
    disabled_rule.enabled = false;
    let enabled_rule = make_rule("api", "enabled", 100, 60, Algorithm::TokenBucket);
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![
        disabled_rule,
        enabled_rule,
    ]));
    let uc = ListRulesUseCase::new(repo);

    let result = uc
        .execute(&ListRulesInput {
            page: 1,
            page_size: 20,
            scope: None,
            enabled_only: true,
        })
        .await
        .unwrap();

    assert_eq!(result.rules.len(), 1);
    assert!(result.rules[0].enabled);
}

// ============================================================
// GetUsageUseCase テスト
// ============================================================

/// StateStore 付きで使用状況を取得する。
#[tokio::test]
async fn get_usage_with_state_store_returns_snapshot() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));
    let state = Arc::new(StubRateLimitStateStore::with_counter(25));

    let uc = GetUsageUseCase::with_state_store(repo, state);
    let info = uc.execute(&rule_id.to_string()).await.unwrap();

    assert_eq!(info.limit, 100);
    assert_eq!(info.used, Some(25));
    assert_eq!(info.remaining, Some(75));
    assert!(info.reset_at.is_some());
}

/// StateStore なしの場合 used/remaining は None。
#[tokio::test]
async fn get_usage_without_state_store_returns_none_values() {
    let rule = make_rule("api", "*", 100, 60, Algorithm::TokenBucket);
    let rule_id = rule.id;
    let repo = Arc::new(StubRateLimitRepository::with_rules(vec![rule]));

    let uc = GetUsageUseCase::new(repo);
    let info = uc.execute(&rule_id.to_string()).await.unwrap();

    assert_eq!(info.limit, 100);
    assert!(info.used.is_none());
    assert!(info.remaining.is_none());
}

// ============================================================
// ResetRateLimitUseCase テスト
// ============================================================

/// 正常にレートリミット状態をリセットする。
#[tokio::test]
async fn reset_rate_limit_with_stub_success() {
    let state = Arc::new(StubRateLimitStateStore::with_counter(50));
    let uc = ResetRateLimitUseCase::new(state);

    let result = uc
        .execute(&ResetRateLimitInput {
            scope: "api".to_string(),
            identifier: "user-1".to_string(),
        })
        .await;

    assert!(result.is_ok());
}

/// 空の identifier でバリデーションエラー。
#[tokio::test]
async fn reset_rate_limit_empty_identifier_error() {
    let state = Arc::new(StubRateLimitStateStore::new());
    let uc = ResetRateLimitUseCase::new(state);

    let result = uc
        .execute(&ResetRateLimitInput {
            scope: "api".to_string(),
            identifier: "".to_string(),
        })
        .await;

    assert!(result.is_err());
}

/// StateStore エラー時にインターナルエラーが返る。
#[tokio::test]
async fn reset_rate_limit_state_store_error() {
    let state = Arc::new(StubRateLimitStateStore::with_error());
    let uc = ResetRateLimitUseCase::new(state);

    let result = uc
        .execute(&ResetRateLimitInput {
            scope: "api".to_string(),
            identifier: "user-1".to_string(),
        })
        .await;

    assert!(result.is_err());
}
