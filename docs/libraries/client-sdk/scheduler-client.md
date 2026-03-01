# k1s0-scheduler-client ライブラリ設計

## 概要

system-scheduler-server（ポート 8093）へのジョブスケジューリングクライアントライブラリ。ジョブの登録（cron 式または one-shot・インターバル）・キャンセル・一時停止・再開・実行状態と実行履歴の取得・ジョブ実行完了イベントの購読（Kafka トピック `k1s0.system.scheduler.job_completed.v1`）を統一インターフェースで提供する。全 Tier のサービスから共通利用し、定期バッチ・遅延実行・一時停止が必要なあらゆる非同期処理のスケジューリング基盤となる。

> **ポート注記**: ポート `8093` は Docker Compose 環境でのホスト側ポートである。本番環境では Kubernetes Service 経由（`scheduler-server:8080`）で接続する。

**配置先**: `regions/system/library/rust/scheduler-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SchedulerClient` | トレイト（`src/client.rs`） | ジョブスケジューリング操作インターフェース |
| `GrpcSchedulerClient` | 構造体 | gRPC 経由の scheduler-server 接続実装 |
| `Job` | 構造体 | ジョブ情報（ID・名称・スケジュール・状態・ペイロード）|
| `JobRequest` | 構造体 | ジョブ登録リクエスト（名称・スケジュール・ペイロード・最大リトライ・タイムアウト）|
| `JobFilter` | 構造体 | ジョブ一覧取得フィルター（状態・名称プレフィックス）|
| `JobExecution` | 構造体 | 実行履歴（実行 ID・ジョブ ID・開始時刻・終了時刻・結果・エラー詳細）|
| `Schedule` | enum | `Cron(String)` / `OneShot(DateTime<Utc>)` / `Interval(Duration)` |
| `JobStatus` | enum | `Pending`・`Running`・`Completed`・`Failed`・`Paused`・`Cancelled` |
| `JobCompletedEvent` | 構造体 | Kafka から購読するジョブ完了イベント |
| `SchedulerError` | enum | `JobNotFound(String)`・`InvalidSchedule(String)`・`ServerError(String)`・`Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-scheduler-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]
kafka = ["rdkafka"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["sync"] }
tokio-stream = "0.1"
tonic = { version = "0.12", optional = true }
rdkafka = { version = "0.37", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**依存追加**: `k1s0-scheduler-client = { path = "../../system/library/rust/scheduler-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
scheduler-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # SchedulerClient トレイト
│   ├── grpc.rs         # GrpcSchedulerClient
│   ├── job.rs          # Job・JobRequest・JobFilter・JobExecution・JobStatus・Schedule・JobCompletedEvent
│   └── error.rs        # SchedulerError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_scheduler_client::{
    GrpcSchedulerClient, JobFilter, JobRequest, JobStatus, Schedule, SchedulerClient,
};
use chrono::{Duration, Utc};
use serde_json::json;

// クライアントの構築
let client = GrpcSchedulerClient::new("http://scheduler-server:8080").await?;

// Cron ジョブの登録（毎日午前 2 時に実行）
let job = client.create_job(JobRequest {
    name: "daily-report".to_string(),
    schedule: Schedule::Cron("0 2 * * *".to_string()),
    payload: json!({ "report_type": "daily", "tenant_id": "TENANT-001" }),
    max_retries: 3,
    timeout_secs: 300,
}).await?;
tracing::info!(job_id = %job.id, "Cron ジョブ登録完了");

// One-shot ジョブの登録（5 分後に一度だけ実行）
let one_shot = client.create_job(JobRequest {
    name: "send-reminder".to_string(),
    schedule: Schedule::OneShot(Utc::now() + Duration::minutes(5)),
    payload: json!({ "user_id": "USR-001", "message": "リマインダー" }),
    max_retries: 1,
    timeout_secs: 30,
}).await?;

// インターバルジョブの登録（10 分ごとに実行）
let interval_job = client.create_job(JobRequest {
    name: "health-sync".to_string(),
    schedule: Schedule::Interval(std::time::Duration::from_secs(600)),
    payload: json!({}),
    max_retries: 0,
    timeout_secs: 60,
}).await?;

// ジョブの一時停止・再開
client.pause_job(&job.id).await?;
client.resume_job(&job.id).await?;

// ジョブのキャンセル
client.cancel_job(&one_shot.id).await?;

// ジョブ情報の取得
let fetched = client.get_job(&job.id).await?;
tracing::info!(status = ?fetched.status, "ジョブ状態");

// ジョブ一覧の取得（実行中のみ）
let filter = JobFilter::new().status(JobStatus::Running);
let running_jobs = client.list_jobs(filter).await?;
tracing::info!(count = running_jobs.len(), "実行中ジョブ一覧");

// 実行履歴の取得
let executions = client.get_executions(&job.id).await?;
for exec in &executions {
    tracing::info!(
        execution_id = %exec.id,
        started_at = %exec.started_at,
        result = %exec.result,
        "実行履歴"
    );
}
```

## Go 実装

**配置先**: `regions/system/library/go/scheduler-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/segmentio/kafka-go v0.4`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type SchedulerClient interface {
    CreateJob(ctx context.Context, req JobRequest) (Job, error)
    CancelJob(ctx context.Context, jobID string) error
    PauseJob(ctx context.Context, jobID string) error
    ResumeJob(ctx context.Context, jobID string) error
    GetJob(ctx context.Context, jobID string) (Job, error)
    ListJobs(ctx context.Context, filter JobFilter) ([]Job, error)
    GetExecutions(ctx context.Context, jobID string) ([]JobExecution, error)
}

type Schedule struct {
    Type     string // "cron", "one_shot", "interval"
    Cron     string
    OneShot  *time.Time
    Interval *time.Duration
}

type JobRequest struct {
    Name        string
    Schedule    Schedule
    Payload     json.RawMessage
    MaxRetries  uint32
    TimeoutSecs uint64
}

type JobStatus string

const (
    JobStatusPending   JobStatus = "pending"
    JobStatusRunning   JobStatus = "running"
    JobStatusCompleted JobStatus = "completed"
    JobStatusFailed    JobStatus = "failed"
    JobStatusPaused    JobStatus = "paused"
    JobStatusCancelled JobStatus = "cancelled"
)

type Job struct {
    ID          string
    Name        string
    Schedule    Schedule
    Status      JobStatus
    Payload     json.RawMessage
    MaxRetries  uint32
    TimeoutSecs uint64
    CreatedAt   time.Time
    NextRunAt   *time.Time
}

type JobFilter struct {
    Status         *JobStatus
    NamePrefix     *string
}

type JobExecution struct {
    ID         string
    JobID      string
    StartedAt  time.Time
    FinishedAt *time.Time
    Result     string
    Error      string // 必須（ゼロ値 "" でエラーなしを表現）
}

type JobCompletedEvent struct {
    JobID       string
    ExecutionID string
    CompletedAt time.Time
    Result      string
}

// InMemoryClient はテスト用インメモリ実装
type InMemoryClient struct{ /* ... */ }

func NewInMemoryClient() *InMemoryClient
func (c *InMemoryClient) CreateJob(ctx context.Context, req JobRequest) (Job, error)
func (c *InMemoryClient) CancelJob(ctx context.Context, jobID string) error
func (c *InMemoryClient) PauseJob(ctx context.Context, jobID string) error
func (c *InMemoryClient) ResumeJob(ctx context.Context, jobID string) error
func (c *InMemoryClient) GetJob(ctx context.Context, jobID string) (Job, error)
func (c *InMemoryClient) ListJobs(ctx context.Context, filter JobFilter) ([]Job, error)
func (c *InMemoryClient) GetExecutions(ctx context.Context, jobID string) ([]JobExecution, error)
func (c *InMemoryClient) Jobs() map[string]Job // テスト用ヘルパー（SchedulerClient インターフェース外）

type GrpcSchedulerClient struct{ /* ... */ }

func NewGrpcSchedulerClient(addr string) (*GrpcSchedulerClient, error)
func (c *GrpcSchedulerClient) CreateJob(ctx context.Context, req JobRequest) (Job, error)
func (c *GrpcSchedulerClient) CancelJob(ctx context.Context, jobID string) error
func (c *GrpcSchedulerClient) PauseJob(ctx context.Context, jobID string) error
func (c *GrpcSchedulerClient) ResumeJob(ctx context.Context, jobID string) error
func (c *GrpcSchedulerClient) GetJob(ctx context.Context, jobID string) (Job, error)
func (c *GrpcSchedulerClient) ListJobs(ctx context.Context, filter JobFilter) ([]Job, error)
func (c *GrpcSchedulerClient) GetExecutions(ctx context.Context, jobID string) ([]JobExecution, error)
```

**使用例**:

```go
client, err := NewGrpcSchedulerClient("scheduler-server:8080")
if err != nil {
    log.Fatal(err)
}

// Cron ジョブ登録
job, err := client.CreateJob(ctx, JobRequest{
    Name: "daily-report",
    Schedule: Schedule{Type: "cron", Cron: "0 2 * * *"},
    Payload:  json.RawMessage(`{"report_type":"daily"}`),
    MaxRetries: 3,
    TimeoutSecs: 300,
})
if err != nil {
    return err
}
log.Printf("ジョブ登録完了: %s", job.ID)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/scheduler-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'paused' | 'cancelled';

export type Schedule =
  | { type: 'cron'; expression: string }
  | { type: 'one_shot'; runAt: Date }
  | { type: 'interval'; intervalMs: number };

export interface JobRequest {
  name: string;
  schedule: Schedule;
  payload: unknown;
  maxRetries?: number;
  timeoutSecs?: number;
}

export interface Job {
  id: string;
  name: string;
  schedule: Schedule;
  status: JobStatus;
  payload: unknown;
  maxRetries: number;
  timeoutSecs: number;
  createdAt: Date;
  nextRunAt?: Date;
}

export interface JobFilter {
  status?: JobStatus;
  namePrefix?: string;
}

export interface JobExecution {
  id: string;
  jobId: string;
  startedAt: Date;
  finishedAt?: Date;
  result: string;
  error?: string;
}

export interface JobCompletedEvent {
  jobId: string;
  executionId: string;
  completedAt: Date;
  result: string;
}

export interface SchedulerClient {
  createJob(req: JobRequest): Promise<Job>;
  cancelJob(jobId: string): Promise<void>;
  pauseJob(jobId: string): Promise<void>;
  resumeJob(jobId: string): Promise<void>;
  getJob(jobId: string): Promise<Job>;
  listJobs(filter?: JobFilter): Promise<Job[]>;
  getExecutions(jobId: string): Promise<JobExecution[]>;
}

export class InMemorySchedulerClient implements SchedulerClient {
  createJob(req: JobRequest): Promise<Job>;
  cancelJob(jobId: string): Promise<void>;
  pauseJob(jobId: string): Promise<void>;
  resumeJob(jobId: string): Promise<void>;
  getJob(jobId: string): Promise<Job>;
  listJobs(filter?: JobFilter): Promise<Job[]>;
  getExecutions(jobId: string): Promise<JobExecution[]>;
  getAll(): Job[]; // テスト用ヘルパー（SchedulerClient インターフェース外）
}

export class GrpcSchedulerClient implements SchedulerClient {
  constructor(serverUrl: string);
  createJob(req: JobRequest): Promise<Job>;
  cancelJob(jobId: string): Promise<void>;
  pauseJob(jobId: string): Promise<void>;
  resumeJob(jobId: string): Promise<void>;
  getJob(jobId: string): Promise<Job>;
  listJobs(filter?: JobFilter): Promise<Job[]>;
  getExecutions(jobId: string): Promise<JobExecution[]>;
  close(): Promise<void>;
}

export class SchedulerError extends Error {
  constructor(
    message: string,
    public readonly code: 'JOB_NOT_FOUND' | 'INVALID_SCHEDULE' | 'SERVER_ERROR' | 'TIMEOUT'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/scheduler_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  grpc: ^4.0.0
  protobuf: ^3.1.0
```

**主要 API**:

```dart
// --- 型定義 ---

enum JobStatus { pending, running, completed, failed, paused, cancelled }

/// Schedule: sealed class（パターンマッチ対応）
sealed class Schedule {
  factory Schedule.cron(String expression) = CronSchedule;
  factory Schedule.oneShot(DateTime runAt) = OneShotSchedule;
  factory Schedule.interval(Duration interval) = IntervalSchedule;
}

class CronSchedule extends Schedule {
  final String expression;
}

class OneShotSchedule extends Schedule {
  final DateTime runAt;
}

class IntervalSchedule extends Schedule {
  final Duration interval;
}

class JobRequest {
  final String name;
  final Schedule schedule;
  final Map<String, dynamic> payload;
  final int maxRetries;
  final int timeoutSecs;
}

class Job {
  final String id;
  final String name;
  final Schedule schedule;
  final JobStatus status;
  final Map<String, dynamic> payload;
  final int maxRetries;
  final int timeoutSecs;
  final DateTime createdAt;
  final DateTime? nextRunAt;

  Job copyWith({JobStatus? status});
}

class JobFilter {
  final JobStatus? status;
  final String? namePrefix;
}

class JobExecution {
  final String id;
  final String jobId;
  final DateTime startedAt;
  final DateTime? finishedAt;
  final String result;
  final String? error;
}

class JobCompletedEvent {
  final String jobId;
  final String executionId;
  final DateTime completedAt;
  final String result;
}

/// SchedulerError: implements Exception
class SchedulerError implements Exception {
  final String message;
  final String code; // 'JOB_NOT_FOUND' | 'INVALID_SCHEDULE' | 'SERVER_ERROR' | 'TIMEOUT'

  @override
  String toString();
}

// --- インターフェース ---

abstract class SchedulerClient {
  Future<Job> createJob(JobRequest request);
  Future<void> cancelJob(String jobId);
  Future<void> pauseJob(String jobId);
  Future<void> resumeJob(String jobId);
  Future<Job> getJob(String jobId);
  Future<List<Job>> listJobs(JobFilter filter);
  Future<List<JobExecution>> getExecutions(String jobId);
}

// --- テスト用インメモリ実装 ---

class InMemorySchedulerClient implements SchedulerClient {
  Future<Job> createJob(JobRequest request);
  Future<void> cancelJob(String jobId);
  Future<void> pauseJob(String jobId);
  Future<void> resumeJob(String jobId);
  Future<Job> getJob(String jobId);
  Future<List<Job>> listJobs(JobFilter filter);
  Future<List<JobExecution>> getExecutions(String jobId);
  Map<String, Job> get jobs; // テスト用ヘルパー（SchedulerClient インターフェース外）
}
```

**使用例**:

```dart
import 'package:k1s0_scheduler_client/scheduler_client.dart';

final client = GrpcSchedulerClient('scheduler-server:8080');

// Cron ジョブ登録
final job = await client.createJob(JobRequest(
  name: 'daily-report',
  schedule: Schedule.cron('0 2 * * *'),
  payload: {'report_type': 'daily', 'tenant_id': 'TENANT-001'},
  maxRetries: 3,
  timeoutSecs: 300,
));
print('ジョブ登録完了: ${job.id}');

// ジョブのキャンセル・一時停止・再開
await client.cancelJob(job.id);
await client.pauseJob(job.id);
await client.resumeJob(job.id);

// ジョブ情報の取得
final fetched = await client.getJob(job.id);
print('ジョブ状態: ${fetched.status}');

// ジョブ一覧の取得（状態フィルター）
final runningJobs = await client.listJobs(JobFilter(status: JobStatus.running));
print('実行中ジョブ数: ${runningJobs.length}');

// 実行履歴
final executions = await client.getExecutions(job.id);
for (final exec in executions) {
  print('実行 ${exec.id}: ${exec.result}');
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_schedule_cron_variant() {
        let schedule = Schedule::Cron("0 2 * * *".to_string());
        assert!(matches!(schedule, Schedule::Cron(_)));
    }

    #[test]
    fn test_schedule_one_shot_variant() {
        let at = Utc::now();
        let schedule = Schedule::OneShot(at);
        assert!(matches!(schedule, Schedule::OneShot(_)));
    }

    #[test]
    fn test_job_status_transitions() {
        let status = JobStatus::Pending;
        assert!(matches!(status, JobStatus::Pending));
    }

    #[test]
    fn test_scheduler_error_job_not_found() {
        let err = SchedulerError::JobNotFound("job-999".to_string());
        assert!(matches!(err, SchedulerError::JobNotFound(_)));
    }

    #[test]
    fn test_scheduler_error_invalid_schedule() {
        let err = SchedulerError::InvalidSchedule("invalid cron expression".to_string());
        assert!(matches!(err, SchedulerError::InvalidSchedule(_)));
    }
}
```

### 統合テスト

- `testcontainers` で scheduler-server コンテナを起動して実際の create/get/cancel フローを検証
- Cron・OneShot・Interval の各スケジュール種別でジョブが正しく登録されることを確認
- pause/resume のライフサイクルが正しく動作することを確認
- 不正な cron 式で `InvalidSchedule` エラーが返ることを確認
- `get_executions` で過去の実行履歴が取得できることを確認（Kafka イベント連携含む）

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestSchedulerClient {}
    #[async_trait]
    impl SchedulerClient for TestSchedulerClient {
        async fn create_job(&self, req: JobRequest) -> Result<Job, SchedulerError>;
        async fn cancel_job(&self, job_id: &str) -> Result<(), SchedulerError>;
        async fn pause_job(&self, job_id: &str) -> Result<(), SchedulerError>;
        async fn resume_job(&self, job_id: &str) -> Result<(), SchedulerError>;
        async fn get_job(&self, job_id: &str) -> Result<Job, SchedulerError>;
        async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<Job>, SchedulerError>;
        async fn get_executions(&self, job_id: &str) -> Result<Vec<JobExecution>, SchedulerError>;
    }
}

#[tokio::test]
async fn test_report_service_registers_daily_cron_on_startup() {
    let mut mock = MockTestSchedulerClient::new();
    mock.expect_create_job()
        .withf(|req| {
            req.name == "daily-report"
                && matches!(req.schedule, Schedule::Cron(_))
        })
        .once()
        .returning(|req| Ok(Job {
            id: "job-001".to_string(),
            name: req.name.clone(),
            schedule: req.schedule.clone(),
            status: JobStatus::Pending,
            payload: req.payload.clone(),
            max_retries: req.max_retries,
            timeout_secs: req.timeout_secs,
            created_at: Utc::now(),
            next_run_at: None,
        }));

    let service = ReportService::new(Arc::new(mock));
    service.start().await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-scheduler-server設計](../../servers/scheduler/server.md) — スケジューラーサーバー設計
- [system-library-kafka設計](../messaging/kafka.md) — Kafka コンシューマー（`k1s0.system.scheduler.job_completed.v1`）
- [system-library-eventstore設計](../data/eventstore.md) — イベント永続化（ジョブ完了イベントの永続化）
- [system-library-retry設計](../resilience/retry.md) — k1s0-retry ライブラリ（ジョブリトライロジック）
