use async_trait::async_trait;

use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;

/// VersionRepository はアプリバージョン管理のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VersionRepository: Send + Sync {
    /// 指定アプリの全バージョンを取得する。
    async fn list_by_app(&self, app_id: &str) -> anyhow::Result<Vec<AppVersion>>;

    /// 新しいバージョンを作成する。
    async fn create(&self, version: &AppVersion) -> anyhow::Result<AppVersion>;

    /// 指定バージョンを削除する。
    async fn delete(
        &self,
        app_id: &str,
        version: &str,
        platform: &Platform,
        arch: &str,
    ) -> anyhow::Result<()>;
}
