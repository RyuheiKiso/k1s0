use async_trait::async_trait;

use crate::domain::entity::app::App;

/// `AppRepository` はアプリ情報取得のためのリポジトリトレイト。
/// CRIT-004 監査対応: RLS テナント分離のため全メソッドに `tenant_id` を追加する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// テナントスコープでアプリ一覧を取得する。カテゴリや検索キーワードでフィルタ可能。
    async fn list(
        &self,
        tenant_id: &str,
        category: Option<String>,
        search: Option<String>,
    ) -> anyhow::Result<Vec<App>>;

    /// テナントスコープでアプリ ID でアプリ情報を取得する。
    async fn find_by_id(&self, tenant_id: &str, id: &str) -> anyhow::Result<Option<App>>;

    /// テナントスコープでアプリを新規登録する。
    async fn create(&self, tenant_id: &str, app: &App) -> anyhow::Result<App>;

    /// テナントスコープでアプリ情報を更新する。
    async fn update(&self, tenant_id: &str, app: &App) -> anyhow::Result<App>;

    /// テナントスコープでアプリを削除する。削除成功なら true を返す。
    async fn delete(&self, tenant_id: &str, id: &str) -> anyhow::Result<bool>;
}
