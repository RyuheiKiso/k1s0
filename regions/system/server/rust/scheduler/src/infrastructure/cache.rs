/// JobCache はスケジューラジョブのインメモリキャッシュ。
/// moka::future::Cache を使用し、TTL 付きでジョブをキャッシュする。
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::entity::scheduler_job::SchedulerJob;

/// キャッシュキーはジョブ ID (UUID) の文字列表現。
pub struct JobCache {
    inner: Cache<String, Arc<SchedulerJob>>,
}

impl JobCache {
    /// 新しい JobCache を作成する。
    ///
    /// # Arguments
    /// * `max_capacity` - キャッシュに保持する最大エントリ数
    /// * `ttl_secs` - エントリの有効期間（秒）
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }

    /// デフォルト設定 (max 1000, TTL 120秒) で JobCache を作成する。
    pub fn default_config() -> Self {
        Self::new(1000, 120)
    }

    /// ジョブ ID に対応するエントリを取得する。
    /// キャッシュミスの場合は None を返す。
    pub async fn get(&self, job_id: &uuid::Uuid) -> Option<Arc<SchedulerJob>> {
        let key = job_id.to_string();
        self.inner.get(&key).await
    }

    /// ジョブをキャッシュに追加する。
    /// キーはジョブの ID から自動生成する。
    pub async fn insert(&self, job: Arc<SchedulerJob>) {
        let key = job.id.to_string();
        self.inner.insert(key, job).await;
    }

    /// 特定のジョブをキャッシュから削除する。
    pub async fn invalidate(&self, job_id: &uuid::Uuid) {
        let key = job_id.to_string();
        self.inner.invalidate(&key).await;
    }

    /// すべてのキャッシュエントリを削除する。
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_job(name: &str) -> Arc<SchedulerJob> {
        Arc::new(SchedulerJob::new(
            name.to_string(),
            "* * * * *".to_string(),
            serde_json::json!({"task": name}),
        ))
    }

    #[tokio::test]
    async fn test_insert_and_get_returns_job() {
        let cache = JobCache::new(100, 60);
        let job = make_job("test-job");

        cache.insert(job.clone()).await;

        let result = cache.get(&job.id).await;
        assert!(result.is_some());
        let cached = result.unwrap();
        assert_eq!(cached.name, "test-job");
        assert_eq!(cached.cron_expression, "* * * * *");
    }

    #[tokio::test]
    async fn test_get_miss_returns_none() {
        let cache = JobCache::new(100, 60);
        let missing_id = Uuid::new_v4();

        let result = cache.get(&missing_id).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_job() {
        let cache = JobCache::new(100, 60);
        let job = make_job("invalidate-test");
        let job_id = job.id;
        cache.insert(job).await;

        // 削除前は取得できる
        assert!(cache.get(&job_id).await.is_some());

        cache.invalidate(&job_id).await;

        // 削除後は取得できない
        assert!(cache.get(&job_id).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_does_not_affect_other_jobs() {
        let cache = JobCache::new(100, 60);
        let job1 = make_job("job-1");
        let job2 = make_job("job-2");
        let id1 = job1.id;
        let id2 = job2.id;

        cache.insert(job1).await;
        cache.insert(job2).await;

        cache.invalidate(&id1).await;

        // job-1 は削除済み
        assert!(cache.get(&id1).await.is_none());
        // job-2 は残っている
        assert!(cache.get(&id2).await.is_some());
    }

    #[tokio::test]
    async fn test_invalidate_all_removes_everything() {
        let cache = JobCache::new(100, 60);
        let job1 = make_job("all-1");
        let job2 = make_job("all-2");
        let id1 = job1.id;
        let id2 = job2.id;

        cache.insert(job1).await;
        cache.insert(job2).await;

        cache.invalidate_all().await;

        assert!(cache.get(&id1).await.is_none());
        assert!(cache.get(&id2).await.is_none());
    }

    #[tokio::test]
    async fn test_insert_overwrites_existing_job() {
        let cache = JobCache::new(100, 60);

        let job_v1 = Arc::new(SchedulerJob {
            id: Uuid::new_v4(),
            name: "overwrite-job".to_string(),
            cron_expression: "* * * * *".to_string(),
            payload: serde_json::json!({"version": 1}),
            status: "active".to_string(),
            next_run_at: None,
            last_run_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });

        let job_v2 = Arc::new(SchedulerJob {
            id: job_v1.id,
            name: "overwrite-job".to_string(),
            cron_expression: "0 12 * * *".to_string(),
            payload: serde_json::json!({"version": 2}),
            status: "active".to_string(),
            next_run_at: None,
            last_run_at: None,
            created_at: job_v1.created_at,
            updated_at: Utc::now(),
        });

        cache.insert(job_v1.clone()).await;
        cache.insert(job_v2).await;

        let result = cache.get(&job_v1.id).await.unwrap();
        assert_eq!(result.cron_expression, "0 12 * * *");
        assert_eq!(result.payload, serde_json::json!({"version": 2}));
    }

    #[tokio::test]
    async fn test_default_config() {
        let cache = JobCache::default_config();
        let job = make_job("default-config-test");
        let job_id = job.id;

        cache.insert(job).await;
        assert!(cache.get(&job_id).await.is_some());
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        // TTL 1秒のキャッシュで、1秒以上待機後にエントリが消えることを確認
        let cache = JobCache::new(100, 1);
        let job = make_job("ttl-test");
        let job_id = job.id;
        cache.insert(job).await;

        // TTL 内は取得できる
        assert!(cache.get(&job_id).await.is_some());

        // TTL を超えるまで待機
        tokio::time::sleep(Duration::from_millis(1200)).await;

        // TTL 超過後はエントリが消えている
        assert!(cache.get(&job_id).await.is_none());
    }
}
