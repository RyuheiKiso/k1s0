# system-scheduler-server 設計

cron 式によるジョブスケジューリング・分散実行管理サーバー。PostgreSQL 分散ロック・実行履歴管理を提供。

## 概要

system tier のスケジューラーサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ジョブ定義管理 | cron 式・ターゲット URL / Kafka トピック・タイムゾーン設定の CRUD |
| 分散実行制御 | 複数インスタンス間での重複実行防止（PostgreSQL による分散ロック） |
| 実行履歴管理 | 実行状態・実行時間・エラー内容を PostgreSQL に記録し一覧取得を提供 |
| ジョブ手動トリガー | REST API による即時実行 |
| 一時停止・再開 | ジョブの一時停止と再開 API |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/scheduler/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| スケジューリング | cron 式をパースして tokio による非同期タイマーで次回実行時刻を計算。起動時に全有効ジョブをロードしタイマーを設定 |
| 分散ロック | PostgreSQL の `SELECT FOR UPDATE SKIP LOCKED` による分散ロックで重複実行防止 |
| トリガー方式 | ジョブ実行時に Kafka `k1s0.system.scheduler.triggered.v1` を発行。ターゲットが HTTP URL の場合は直接 POST も実行可能 |
| DB | PostgreSQL の `scheduler` スキーマ（scheduler_jobs, scheduler_executions テーブル） |
| Kafka | プロデューサー（`k1s0.system.scheduler.triggered.v1`）のみ。設定がない場合は HTTP コールバックのみ動作 |
| タイムゾーン | ジョブごとにタイムゾーンを設定可能（デフォルト UTC）。DST を考慮した次回実行時刻を計算 |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SCHED_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/jobs` | ジョブ一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/jobs/:id` | ジョブ詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/jobs` | ジョブ作成 | `sys_operator` 以上 |
| PUT | `/api/v1/jobs/:id` | ジョブ更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/jobs/:id` | ジョブ削除 | `sys_admin` のみ |
| POST | `/api/v1/jobs/:id/trigger` | ジョブ手動実行 | `sys_operator` 以上 |
| PUT | `/api/v1/jobs/:id/pause` | ジョブ一時停止 | `sys_operator` 以上 |
| PUT | `/api/v1/jobs/:id/resume` | ジョブ再開 | `sys_operator` 以上 |
| GET | `/api/v1/jobs/:id/executions` | 実行履歴一覧 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/jobs

ジョブ一覧をページネーション付きで取得する。`status` クエリパラメータでフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `status` | string | No | - | ジョブ状態でフィルタ（active/paused） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス例（200 OK）**

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

#### GET /api/v1/jobs/:id

ID 指定でジョブの詳細を取得する。

**レスポンス例（200 OK）**

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

**レスポンス例（404 Not Found）**

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

#### POST /api/v1/jobs

新しいスケジューラージョブを作成する。`target_type` は `kafka` または `http` を指定する。作成後すぐにスケジューリングが開始される。

**リクエスト例**

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

**レスポンス例（201 Created）**

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

**レスポンス例（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SCHED_INVALID_CRON",
    "message": "invalid cron expression: <expr>",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### PUT /api/v1/jobs/:id

既存のジョブを更新する。更新後は次回実行時刻が再計算される。

**リクエスト例**

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

**レスポンス例（200 OK）**

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

#### DELETE /api/v1/jobs/:id

ジョブを削除する。

**レスポンス例（200 OK）**

```json
{
  "success": true,
  "message": "job job_01JABCDEF1234567890 deleted"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SCHED_NOT_FOUND",
    "message": "job not found: job_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/jobs/:id/trigger

ジョブをスケジュールに関係なく即時実行する。分散ロックを取得し重複実行を防止する。

**レスポンス例（200 OK）**

```json
{
  "execution_id": "exec_01JABCDEF1234567890",
  "job_id": "job_01JABCDEF1234567890",
  "status": "running",
  "triggered_at": "2026-02-23T12:00:00.000+00:00"
}
```

**レスポンス例（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SCHED_NOT_ACTIVE",
    "message": "job is not active: job_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/jobs/:id/executions

指定ジョブの実行履歴一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `status` | string | No | - | 実行状態でフィルタ（running/succeeded/failed） |
| `from` | string | No | - | 開始日時（RFC3339） |
| `to` | string | No | - | 終了日時（RFC3339） |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス例（200 OK）**

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

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SCHED_NOT_FOUND` | 404 | 指定されたジョブが見つからない |
| `SYS_SCHED_ALREADY_EXISTS` | 409 | 同一名のジョブが既に存在する |
| `SYS_SCHED_NOT_ACTIVE` | 409 | ジョブがアクティブでないため実行できない |
| `SYS_SCHED_INVALID_STATUS` | 409 | 操作に対してジョブのステータスが不正 |
| `SYS_SCHED_INVALID_CRON` | 400 | cron 式が不正 |
| `SYS_SCHED_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.scheduler.v1;

import "k1s0/system/common/v1/types.proto";

service SchedulerService {
  rpc TriggerJob(TriggerJobRequest) returns (TriggerJobResponse);
  // GetJobExecution は現在 Unimplemented を返す（TODO: 実行リポジトリ実装後に完成）
  rpc GetJobExecution(GetJobExecutionRequest) returns (GetJobExecutionResponse);
}

message TriggerJobRequest {
  string job_id = 1;
}

message TriggerJobResponse {
  string execution_id = 1;
  string job_id = 2;
  // 実行状態（running / succeeded / failed）
  string status = 3;
  // 現在 null 固定（TODO: triggered_at を返すよう実装予定）
  k1s0.system.common.v1.Timestamp triggered_at = 4;
}

message GetJobExecutionRequest {
  string execution_id = 1;
}

message GetJobExecutionResponse {
  JobExecution execution = 1;
}

message JobExecution {
  string id = 1;
  string job_id = 2;
  // 実行状態（running / succeeded / failed）
  string status = 3;
  // 実行トリガー（scheduler / manual）
  string triggered_by = 4;
  k1s0.system.common.v1.Timestamp started_at = 5;
  optional k1s0.system.common.v1.Timestamp finished_at = 6;
  // 実行時間（ミリ秒）
  optional uint64 duration_ms = 7;
  optional string error_message = 8;
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `SchedulerJob`, `JobExecution` | エンティティ定義 |
| domain/repository | `SchedulerJobRepository`, `JobExecutionRepository` | リポジトリトレイト |
| domain/service | `SchedulerDomainService` | cron 式解析・次回実行時刻計算・分散ロック判定 |
| usecase | `CreateJobUsecase`, `UpdateJobUsecase`, `DeleteJobUsecase`, `GetJobUsecase`, `ListJobsUsecase`, `TriggerJobUsecase`, `PauseJobUsecase`, `ResumeJobUsecase`, `ListExecutionsUsecase`, `GetExecutionUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| adapter/scheduler | `CronSchedulerEngine` | tokio による cron スケジューリングループ |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `SchedulerJobPostgresRepository`, `JobExecutionPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/lock | `DistributedLockPostgres` | PostgreSQL 分散ロック実装 |
| infrastructure/messaging | `JobTriggeredKafkaProducer` | Kafka プロデューサー（ジョブトリガー通知） |

### ドメインモデル

#### SchedulerJob

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ジョブの一意識別子 |
| `name` | String | ジョブの表示名 |
| `description` | String | ジョブの説明 |
| `cron_expression` | String | cron 式（例: `0 0 * * *`） |
| `timezone` | String | タイムゾーン（例: `Asia/Tokyo`） |
| `target_type` | String | ターゲット種別（kafka / http） |
| `target` | String | Kafka トピック名または HTTP URL |
| `payload` | Option\<JSON\> | ジョブ実行時に渡すペイロード |
| `status` | String | ジョブ状態（active / paused） |
| `next_run_at` | Option\<DateTime\<Utc\>\> | 次回実行予定日時 |
| `last_run_at` | Option\<DateTime\<Utc\>\> | 最終実行日時 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### JobExecution

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | 実行ログの一意識別子 |
| `job_id` | String | 対象ジョブ ID |
| `status` | String | 実行状態（running / succeeded / failed） |
| `triggered_by` | String | 実行トリガー（scheduler / manual） |
| `started_at` | DateTime\<Utc\> | 実行開始日時 |
| `finished_at` | Option\<DateTime\<Utc\>\> | 実行完了日時 |
| `duration_ms` | Option\<u64\> | 実行時間（ミリ秒） |
| `error_message` | Option\<String\> | エラーメッセージ（失敗時） |

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

---

## 詳細設計ドキュメント

- [system-scheduler-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-scheduler-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。
