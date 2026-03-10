use async_trait::async_trait;

use crate::domain::entity::download_stat::DownloadStat;

/// DownloadStatsRepository はダウンロード統計情報のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DownloadStatsRepository: Send + Sync {
    /// ダウンロード統計を記録する。
    async fn record(&self, stat: &DownloadStat) -> anyhow::Result<()>;

    /// アプリ全体のダウンロード数を取得する。
    #[allow(dead_code)]
    async fn count_by_app(&self, app_id: &str) -> anyhow::Result<i64>;

    /// 特定バージョンのダウンロード数を取得する。
    #[allow(dead_code)]
    async fn count_by_version(&self, app_id: &str, version: &str) -> anyhow::Result<i64>;
}
