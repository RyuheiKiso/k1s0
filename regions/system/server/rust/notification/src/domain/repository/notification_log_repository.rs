use async_trait::async_trait;

use crate::domain::entity::notification_log::NotificationLog;

/// 通知ログリポジトリトレイト。全メソッドで tenant_id を受け取り RLS テナント分離を強制する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationLogRepository: Send + Sync {
    /// テナントスコープで ID による通知ログ検索を行う
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<NotificationLog>>;
    /// テナントスコープでチャンネル ID による通知ログ検索を行う
    #[allow(dead_code)]
    async fn find_by_channel_id(&self, channel_id: &str, tenant_id: &str) -> anyhow::Result<Vec<NotificationLog>>;
    /// テナントスコープでページネーション付き通知ログ一覧を取得する
    async fn find_all_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)>;
    /// 通知ログを作成する（log.tenant_id を使用して RLS コンテキストを設定する）
    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()>;
    /// 通知ログを更新する（log.tenant_id を使用して RLS コンテキストを設定する）
    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()>;
}
