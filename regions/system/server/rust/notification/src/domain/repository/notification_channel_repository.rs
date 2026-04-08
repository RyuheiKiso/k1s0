use async_trait::async_trait;

use crate::domain::entity::notification_channel::NotificationChannel;

/// MEDIUM-RUST-001 監査対応: チャンネルリポジトリトレイト
/// 全取得・削除系メソッドに `tenant_id` を追加してテナント分離を強制する。
/// RLS の `set_config` に渡す `tenant_id` を呼び出し元（ユースケース）から受け取る設計。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationChannelRepository: Send + Sync {
    /// 指定テナントのチャンネルを ID で検索する
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<NotificationChannel>>;
    /// 指定テナントの全チャンネルを取得する
    #[allow(dead_code)]
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<NotificationChannel>>;
    /// 指定テナントのチャンネルをページネーション付きで取得する
    async fn find_all_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)>;
    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()>;
    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()>;
    /// 指定テナントのチャンネルを削除する
    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool>;
}
