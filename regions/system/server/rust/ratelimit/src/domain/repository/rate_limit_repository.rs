use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::{RateLimitDecision, RateLimitRule};

/// RateLimitRepository はルールの永続化を担当する（PostgreSQL）。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RateLimitRepository: Send + Sync {
    /// ルールを作成する。
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule>;

    /// ID でルールを取得する。
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<RateLimitRule>;

    /// name でルールを取得する。
    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<RateLimitRule>>;

    /// scope でルールを取得する。
    async fn find_by_scope(&self, scope: &str) -> anyhow::Result<Vec<RateLimitRule>>;

    /// 全ルールを取得する。
    async fn find_all(&self) -> anyhow::Result<Vec<RateLimitRule>>;

    /// ルールを更新する。
    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()>;

    /// ルールを削除する。削除された場合 true を返す。
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;

    /// レートリミット状態をリセットする。
    async fn reset_state(&self, key: &str) -> anyhow::Result<()>;
}

/// UsageSnapshot はレートリミットの現在の使用状況スナップショット。
#[derive(Debug, Clone)]
pub struct UsageSnapshot {
    pub used: i64,
    pub remaining: i64,
    pub reset_at: i64,
}

/// RateLimitStateStore はレートリミット状態の管理を担当する（Redis）。
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

    /// レートリミット状態をリセットする。
    async fn reset(&self, key: &str) -> anyhow::Result<()>;

    /// 指定キーの現在の使用状況を取得する。
    async fn get_usage(&self, key: &str, limit: i64, window_secs: i64) -> anyhow::Result<Option<UsageSnapshot>>;
}
