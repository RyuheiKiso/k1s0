# system-scheduler-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/jobs

```json
{
  "jobs": [
    {
      "id": "job_01JABCDEF1234567890",
      "name": "日次レポート生成",
      "description": "毎日 0 時に日次レポートを生成する",
      "cron_expression": "0 0 * * *",
      "timezone": "Asia/Tokyo",
      "target_type": "kafka",
      "target": "k1s0.business.report.generate.v1",
      "status": "active",
      "next_run_at": "2026-02-24T00:00:00.000+09:00",
      "last_run_at": "2026-02-23T00:00:00.000+09:00",
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 10,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### GET /api/v1/jobs/:id

**レスポンス（200 OK）**

```json
{
  "id": "job_01JABCDEF1234567890",
  "name": "日次レポート生成",
  "description": "毎日 0 時に日次レポートを生成する",
  "cron_expression": "0 0 * * *",
  "timezone": "Asia/Tokyo",
  "target_type": "kafka",
  "target": "k1s0.business.report.generate.v1",
  "payload": {
    "report_type": "daily"
  },
  "status": "active",
  "next_run_at": "2026-02-24T00:00:00.000+09:00",
  "last_run_at": "2026-02-23T00:00:00.000+09:00",
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SCHED_NOT_FOUND",
    "message": "scheduler job not found: job_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/jobs

**リクエスト**

```json
{
  "name": "日次レポート生成",
  "description": "毎日 0 時に日次レポートを生成する",
  "cron_expression": "0 0 * * *",
  "timezone": "Asia/Tokyo",
  "target_type": "kafka",
  "target": "k1s0.business.report.generate.v1",
  "payload": {
    "report_type": "daily"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "job_01JABCDEF1234567890",
  "name": "日次レポート生成",
  "description": "毎日 0 時に日次レポートを生成する",
  "cron_expression": "0 0 * * *",
  "timezone": "Asia/Tokyo",
  "target_type": "kafka",
  "target": "k1s0.business.report.generate.v1",
  "payload": {
    "report_type": "daily"
  },
  "status": "active",
  "next_run_at": "2026-02-24T00:00:00.000+09:00",
  "last_run_at": null,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SCHED_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "cron_expression", "message": "invalid cron expression"},
      {"field": "timezone", "message": "unknown timezone: Invalid/Zone"}
    ]
  }
}
```

### PUT /api/v1/jobs/:id

**リクエスト**

```json
{
  "name": "日次レポート生成（更新）",
  "description": "毎日 1 時に日次レポートを生成する",
  "cron_expression": "0 1 * * *",
  "timezone": "Asia/Tokyo",
  "target_type": "kafka",
  "target": "k1s0.business.report.generate.v1",
  "payload": {
    "report_type": "daily"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "id": "job_01JABCDEF1234567890",
  "name": "日次レポート生成（更新）",
  "description": "毎日 1 時に日次レポートを生成する",
  "cron_expression": "0 1 * * *",
  "timezone": "Asia/Tokyo",
  "target_type": "kafka",
  "target": "k1s0.business.report.generate.v1",
  "payload": {
    "report_type": "daily"
  },
  "status": "active",
  "next_run_at": "2026-02-24T01:00:00.000+09:00",
  "last_run_at": "2026-02-23T00:00:00.000+09:00",
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T15:00:00.000+00:00"
}
```

### DELETE /api/v1/jobs/:id

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "scheduler job job_01JABCDEF1234567890 deleted"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SCHED_JOB_RUNNING",
    "message": "cannot delete a running job: job_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/jobs/:id/trigger

**レスポンス（202 Accepted）**

```json
{
  "execution_id": "exec_01JABCDEF1234567890",
  "job_id": "job_01JABCDEF1234567890",
  "status": "running",
  "triggered_at": "2026-02-23T12:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SCHED_JOB_RUNNING",
    "message": "job is already running: job_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/jobs/:id/executions

```json
{
  "executions": [
    {
      "id": "exec_01JABCDEF1234567890",
      "job_id": "job_01JABCDEF1234567890",
      "status": "succeeded",
      "triggered_by": "scheduler",
      "started_at": "2026-02-23T00:00:00.000+00:00",
      "finished_at": "2026-02-23T00:00:01.500+00:00",
      "duration_ms": 1500,
      "error_message": null
    }
  ],
  "pagination": {
    "total_count": 30,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (scheduler_handler.rs)      │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_jobs / get_job / create_job        │   │
                    │  │  update_job / delete_job                 │   │
                    │  │  trigger_job / pause_job / resume_job    │   │
                    │  │  list_executions                         │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (scheduler_grpc.rs)         │   │
                    │  │  TriggerJob / GetJobExecution            │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ CronSchedulerEngine                      │   │
                    │  │  tokio cron scheduling loop              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateJobUsecase / UpdateJobUsecase /          │
                    │  DeleteJobUsecase / GetJobUsecase /             │
                    │  ListJobsUsecase / TriggerJobUsecase /          │
                    │  PauseJobUsecase / ResumeJobUsecase /           │
                    │  ListExecutionsUsecase / GetExecutionUsecase    │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  SchedulerJob,  │              │ SchedulerJobRepository     │   │
    │  JobExecution   │              │ JobExecutionRepository     │   │
    └────────────────┘              │ (trait)                    │   │
              │                     └──────────┬─────────────────┘   │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ Scheduler      │            │                     │
                 │ DomainService  │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ SchedulerJobPostgres   │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ JobExecutionPostgres   │  │
                    │  │ Distributed  │  │ Repository             │  │
                    │  │ Lock         │  └────────────────────────┘  │
                    │  │ (Postgres)   │  ┌────────────────────────┐  │
                    │  └──────────────┘  │ Database               │  │
                    │  ┌──────────────┐  │ Config                 │  │
                    │  │ Config       │  └────────────────────────┘  │
                    │  │ Loader       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```
