use async_trait::async_trait;

use crate::domain::entity::app::App;

/// AppRepository はアプリ情報取得のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// アプリ一覧を取得する。カテゴリや検索キーワードでフィルタ可能。
    async fn list(
        &self,
        category: Option<String>,
        search: Option<String>,
    ) -> anyhow::Result<Vec<App>>;

    /// アプリ ID でアプリ情報を取得する。
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<App>>;

    /// アプリを新規登録する。
    async fn create(&self, app: &App) -> anyhow::Result<App>;

    /// アプリ情報を更新する。
    async fn update(&self, app: &App) -> anyhow::Result<App>;

    /// アプリを削除する。削除成功なら true を返す。
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}
