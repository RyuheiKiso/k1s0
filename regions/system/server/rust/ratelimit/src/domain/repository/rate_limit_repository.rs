use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::{RateLimitDecision, RateLimitRule};

/// `RateLimitRepository` はルールの永続化を担当する（PostgreSQL）。
/// CRIT-005 対応: RLS テナント分離のため全クエリに `tenant_id` を渡す。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RateLimitRepository: Send + Sync {
    /// ルールを作成する。テナント ID はルールエンティティ内の `tenant_id` フィールドから取得する。
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら ID でルールを取得する。
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<RateLimitRule>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら name でルールを取得する。
    #[allow(dead_code)]
    async fn find_by_name(
        &self,
        name: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Option<RateLimitRule>>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら scope でルールを取得する。
    async fn find_by_scope(
        &self,
        scope: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<RateLimitRule>>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら全ルールを取得する。
    #[allow(dead_code)]
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<RateLimitRule>>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながら条件付きのページネーションでルールを取得する。
    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)>;

    /// ルールを更新する。テナント ID はルールエンティティ内の `tenant_id` フィールドから取得する。
    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()>;

    /// CRIT-005 対応: `tenant_id` を渡して RLS でテナント分離しながらルールを削除する。削除された場合 true を返す。
    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool>;

    /// レートリミット状態をリセットする。
    #[allow(dead_code)]
    async fn reset_state(&self, key: &str) -> anyhow::Result<()>;
}

/// `UsageSnapshot` はレートリミットの現在の使用状況スナップショット。
#[derive(Debug, Clone)]
pub struct UsageSnapshot {
    pub used: i64,
    pub remaining: i64,
    pub reset_at: i64,
}

/// `RateLimitStateStore` はレートリミット状態の管理を担当する（Redis）。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RateLimitStateStore: Send + Sync {
    /// トークンバケットアルゴリズムでレートリミットをチェックする。
    async fn check_token_bucket(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision>;

    /// 固定ウィンドウアルゴリズムでレートリミットをチェックする。
    async fn check_fixed_window(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision>;

    /// スライディングウィンドウアルゴリズムでレートリミットをチェックする。
    async fn check_sliding_window(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision>;

    /// リーキーバケットアルゴリズムでレートリミットをチェックする。
    async fn check_leaky_bucket(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision>;

    /// レートリミット状態をリセットする。
    async fn reset(&self, key: &str) -> anyhow::Result<()>;

    /// 指定キーの現在の使用状況を取得する。
    async fn get_usage(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<Option<UsageSnapshot>>;
}
