use async_trait::async_trait;

use crate::domain::entity::notification_template::NotificationTemplate;

/// テンプレートリポジトリトレイト。全メソッドで tenant_id を受け取り RLS テナント分離を強制する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationTemplateRepository: Send + Sync {
    /// テナントスコープで ID によるテンプレート検索を行う
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<NotificationTemplate>>;
    /// テナントスコープで全テンプレートを取得する
    #[allow(dead_code)]
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<NotificationTemplate>>;
    /// テナントスコープでページネーション付きテンプレート一覧を取得する
    async fn find_all_paginated(
        &self,
        tenant_id: &str,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)>;
    /// テンプレートを作成する（template.tenant_id を使用して RLS コンテキストを設定する）
    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    /// テンプレートを更新する（template.tenant_id を使用して RLS コンテキストを設定する）
    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()>;
    /// テナントスコープでテンプレートを削除する
    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool>;
}
