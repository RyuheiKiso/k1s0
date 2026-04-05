use async_trait::async_trait;

use crate::domain::entity::scheduler_job::SchedulerJob;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchedulerJobRepository: Send + Sync {
    /// テナント ID でフィルタリングしてジョブを ID 検索する（CRIT-005 対応）
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<SchedulerJob>>;
    /// テナント ID でフィルタリングして全ジョブを取得する（CRIT-005 対応）
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<SchedulerJob>>;
    /// ジョブを作成する。SchedulerJob.tenant_id で RLS 設定を行う（CRIT-005 対応）
    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()>;
    /// ジョブを更新する。SchedulerJob.tenant_id で RLS 設定を行う（CRIT-005 対応）
    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()>;
    /// ジョブを削除する。tenant_id で RLS 設定を行う（CRIT-005 対応）
    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool>;
    /// アクティブなジョブを全テナント横断で取得する（スケジューラー内部用途）
    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>>;
}
