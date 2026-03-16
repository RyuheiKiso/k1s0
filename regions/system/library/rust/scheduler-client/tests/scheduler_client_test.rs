// scheduler-client の外部結合テスト。
// InMemorySchedulerClient のジョブライフサイクル（作成→取得→更新→削除）を検証する。

use std::time::Duration;

use k1s0_scheduler_client::{
    InMemorySchedulerClient, JobFilter, JobRequest, JobStatus, Schedule, SchedulerClient,
    SchedulerError,
};

// テスト用のジョブリクエストを生成するヘルパー関数。
fn make_job_request(name: &str, schedule: Schedule) -> JobRequest {
    JobRequest {
        name: name.to_string(),
        schedule,
        payload: serde_json::json!({"type": "test"}),
        max_retries: 3,
        timeout_secs: 60,
    }
}

// --- ジョブ作成テスト ---

// ジョブを作成して正しいフィールドが設定されることを確認する。
#[tokio::test]
async fn test_create_job() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "daily-report",
            Schedule::Cron("0 2 * * *".to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(job.id, "job-001");
    assert_eq!(job.name, "daily-report");
    assert_eq!(job.status, JobStatus::Pending);
    assert_eq!(job.max_retries, 3);
    assert_eq!(job.timeout_secs, 60);
}

// 複数のジョブを作成するとシーケンシャルな ID が振られることを確認する。
#[tokio::test]
async fn test_create_multiple_jobs() {
    let client = InMemorySchedulerClient::new();

    let job1 = client
        .create_job(make_job_request(
            "job-a",
            Schedule::Interval(Duration::from_secs(300)),
        ))
        .await
        .unwrap();
    let job2 = client
        .create_job(make_job_request(
            "job-b",
            Schedule::Cron("*/5 * * * *".to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(job1.id, "job-001");
    assert_eq!(job2.id, "job-002");
}

// Interval スケジュールで作成したジョブに next_run_at が設定されることを確認する。
#[tokio::test]
async fn test_create_job_interval_has_next_run() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "interval-job",
            Schedule::Interval(Duration::from_secs(600)),
        ))
        .await
        .unwrap();

    assert!(job.next_run_at.is_some());
}

// Cron スケジュールで作成したジョブの next_run_at が None であることを確認する。
#[tokio::test]
async fn test_create_job_cron_no_next_run() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "cron-job",
            Schedule::Cron("0 * * * *".to_string()),
        ))
        .await
        .unwrap();

    // Cron の場合は next_run_at が None（InMemorySchedulerClient の実装による）
    assert!(job.next_run_at.is_none());
}

// --- ジョブ取得テスト ---

// 作成したジョブを ID で取得できることを確認する。
#[tokio::test]
async fn test_get_job() {
    let client = InMemorySchedulerClient::new();
    let created = client
        .create_job(make_job_request(
            "test-job",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    let job = client.get_job(&created.id).await.unwrap();
    assert_eq!(job.id, created.id);
    assert_eq!(job.name, "test-job");
}

// 存在しないジョブ ID で取得すると JobNotFound エラーが返ることを確認する。
#[tokio::test]
async fn test_get_job_not_found() {
    let client = InMemorySchedulerClient::new();
    let result = client.get_job("nonexistent").await;
    assert!(matches!(result, Err(SchedulerError::JobNotFound(_))));
}

// --- ジョブ状態更新テスト ---

// ジョブを一時停止すると Paused ステータスになることを確認する。
#[tokio::test]
async fn test_pause_job() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "pause-test",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    client.pause_job(&job.id).await.unwrap();
    let paused = client.get_job(&job.id).await.unwrap();
    assert_eq!(paused.status, JobStatus::Paused);
}

// 一時停止したジョブを再開すると Pending ステータスになることを確認する。
#[tokio::test]
async fn test_resume_job() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "resume-test",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    client.pause_job(&job.id).await.unwrap();
    client.resume_job(&job.id).await.unwrap();
    let resumed = client.get_job(&job.id).await.unwrap();
    assert_eq!(resumed.status, JobStatus::Pending);
}

// ジョブをキャンセルすると Cancelled ステータスになることを確認する。
#[tokio::test]
async fn test_cancel_job() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "cancel-test",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    client.cancel_job(&job.id).await.unwrap();
    let cancelled = client.get_job(&job.id).await.unwrap();
    assert_eq!(cancelled.status, JobStatus::Cancelled);
}

// 存在しないジョブの一時停止が JobNotFound エラーを返すことを確認する。
#[tokio::test]
async fn test_pause_nonexistent_job() {
    let client = InMemorySchedulerClient::new();
    let result = client.pause_job("missing").await;
    assert!(matches!(result, Err(SchedulerError::JobNotFound(_))));
}

// 存在しないジョブの再開が JobNotFound エラーを返すことを確認する。
#[tokio::test]
async fn test_resume_nonexistent_job() {
    let client = InMemorySchedulerClient::new();
    let result = client.resume_job("missing").await;
    assert!(matches!(result, Err(SchedulerError::JobNotFound(_))));
}

// 存在しないジョブのキャンセルが JobNotFound エラーを返すことを確認する。
#[tokio::test]
async fn test_cancel_nonexistent_job() {
    let client = InMemorySchedulerClient::new();
    let result = client.cancel_job("missing").await;
    assert!(matches!(result, Err(SchedulerError::JobNotFound(_))));
}

// --- ジョブ一覧テスト ---

// フィルタなしで全ジョブを一覧取得できることを確認する。
#[tokio::test]
async fn test_list_jobs_no_filter() {
    let client = InMemorySchedulerClient::new();
    client
        .create_job(make_job_request(
            "job-a",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();
    client
        .create_job(make_job_request(
            "job-b",
            Schedule::Interval(Duration::from_secs(60)),
        ))
        .await
        .unwrap();

    let jobs = client.list_jobs(JobFilter::new()).await.unwrap();
    assert_eq!(jobs.len(), 2);
}

// ステータスフィルタで特定ステータスのジョブのみ取得できることを確認する。
#[tokio::test]
async fn test_list_jobs_status_filter() {
    let client = InMemorySchedulerClient::new();
    let job1 = client
        .create_job(make_job_request(
            "active",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();
    client
        .create_job(make_job_request(
            "paused",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    client.pause_job(&job1.id).await.unwrap();

    let paused_jobs = client
        .list_jobs(JobFilter::new().status(JobStatus::Paused))
        .await
        .unwrap();
    assert_eq!(paused_jobs.len(), 1);
    assert_eq!(paused_jobs[0].name, "active"); // job1 was paused

    let pending_jobs = client
        .list_jobs(JobFilter::new().status(JobStatus::Pending))
        .await
        .unwrap();
    assert_eq!(pending_jobs.len(), 1);
    assert_eq!(pending_jobs[0].name, "paused"); // job2 is still pending
}

// 名前プレフィックスフィルタで特定のジョブのみ取得できることを確認する。
#[tokio::test]
async fn test_list_jobs_name_prefix_filter() {
    let client = InMemorySchedulerClient::new();
    client
        .create_job(make_job_request(
            "daily-report",
            Schedule::Cron("0 2 * * *".to_string()),
        ))
        .await
        .unwrap();
    client
        .create_job(make_job_request(
            "daily-backup",
            Schedule::Cron("0 3 * * *".to_string()),
        ))
        .await
        .unwrap();
    client
        .create_job(make_job_request(
            "hourly-check",
            Schedule::Cron("0 * * * *".to_string()),
        ))
        .await
        .unwrap();

    let daily_jobs = client
        .list_jobs(JobFilter::new().name_prefix("daily"))
        .await
        .unwrap();
    assert_eq!(daily_jobs.len(), 2);

    let hourly_jobs = client
        .list_jobs(JobFilter::new().name_prefix("hourly"))
        .await
        .unwrap();
    assert_eq!(hourly_jobs.len(), 1);
}

// --- 実行履歴テスト ---

// 新規作成したジョブの実行履歴が空であることを確認する。
#[tokio::test]
async fn test_get_executions_empty() {
    let client = InMemorySchedulerClient::new();
    let job = client
        .create_job(make_job_request(
            "new-job",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    let executions = client.get_executions(&job.id).await.unwrap();
    assert!(executions.is_empty());
}

// 存在しないジョブの実行履歴取得が JobNotFound エラーを返すことを確認する。
#[tokio::test]
async fn test_get_executions_not_found() {
    let client = InMemorySchedulerClient::new();
    let result = client.get_executions("missing").await;
    assert!(matches!(result, Err(SchedulerError::JobNotFound(_))));
}

// --- スナップショットテスト ---

// jobs() メソッドが現在の全ジョブのスナップショットを返すことを確認する。
#[tokio::test]
async fn test_jobs_snapshot() {
    let client = InMemorySchedulerClient::new();
    client
        .create_job(make_job_request(
            "snap-job",
            Schedule::Cron("* * * * *".to_string()),
        ))
        .await
        .unwrap();

    let snapshot = client.jobs().await;
    assert_eq!(snapshot.len(), 1);
    assert!(snapshot.contains_key("job-001"));
}

// --- フルライフサイクルテスト ---

// ジョブの作成→取得→一時停止→再開→キャンセルの完全なライフサイクルを確認する。
#[tokio::test]
async fn test_full_job_lifecycle() {
    let client = InMemorySchedulerClient::new();

    // 1. 作成
    let job = client
        .create_job(make_job_request(
            "lifecycle-test",
            Schedule::Interval(Duration::from_secs(300)),
        ))
        .await
        .unwrap();
    assert_eq!(job.status, JobStatus::Pending);

    // 2. 取得
    let fetched = client.get_job(&job.id).await.unwrap();
    assert_eq!(fetched.name, "lifecycle-test");

    // 3. 一時停止
    client.pause_job(&job.id).await.unwrap();
    assert_eq!(
        client.get_job(&job.id).await.unwrap().status,
        JobStatus::Paused
    );

    // 4. 再開
    client.resume_job(&job.id).await.unwrap();
    assert_eq!(
        client.get_job(&job.id).await.unwrap().status,
        JobStatus::Pending
    );

    // 5. キャンセル
    client.cancel_job(&job.id).await.unwrap();
    assert_eq!(
        client.get_job(&job.id).await.unwrap().status,
        JobStatus::Cancelled
    );
}

// --- エラーテスト ---

// SchedulerError の各バリアントが正しい表示文字列を持つことを確認する。
#[test]
fn test_scheduler_error_display() {
    let err = SchedulerError::JobNotFound("job-999".to_string());
    assert!(err.to_string().contains("job-999"));

    let err = SchedulerError::InvalidSchedule("bad cron".to_string());
    assert!(err.to_string().contains("bad cron"));

    let err = SchedulerError::ServerError("internal error".to_string());
    assert!(err.to_string().contains("internal error"));

    let err = SchedulerError::Timeout;
    assert!(err.to_string().contains("タイムアウト"));
}
