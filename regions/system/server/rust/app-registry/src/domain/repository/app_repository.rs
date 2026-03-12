use async_trait::async_trait;

use crate::domain::entity::app::App;

/// AppRepository はアプリ情報取得のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AppRepository: Send + Sync {
    /// アプリ一覧を取得する。カテゴリや検索キーワードでフィルタ可能。
    async fn list(&self, category: Option<String>, search: Option<String>) -> anyhow::Result<Vec<App>>;

    /// アプリ ID でアプリ情報を取得する。
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<App>>;
}
