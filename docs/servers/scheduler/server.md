# system-scheduler-server 設計

cron 式によるジョブスケジューリング・分散実行管理サーバー。PostgreSQL 分散ロック・実行履歴管理を提供。

> **ガイド**: 実装例・依存関係図は [server.guide.md](./server.guide.md) を参照。

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

#### GET /api/v1/jobs/:id

ID 指定でジョブの詳細を取得する。

#### POST /api/v1/jobs

新しいスケジューラージョブを作成する。`target_type` は `kafka` または `http` を指定する。作成後すぐにスケジューリングが開始される。

#### PUT /api/v1/jobs/:id

既存のジョブを更新する。更新後は次回実行時刻が再計算される。

#### DELETE /api/v1/jobs/:id

ジョブを削除する。実行中のジョブは削除できない。

#### POST /api/v1/jobs/:id/trigger

ジョブをスケジュールに関係なく即時実行する。分散ロックを取得し重複実行を防止する。

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

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SCHED_NOT_FOUND` | 404 | 指定されたジョブが見つからない |
| `SYS_SCHED_ALREADY_EXISTS` | 409 | 同一名のジョブが既に存在する |
| `SYS_SCHED_JOB_RUNNING` | 409 | ジョブが実行中のため操作できない |
| `SYS_SCHED_INVALID_STATUS` | 409 | 操作に対してジョブのステータスが不正 |
| `SYS_SCHED_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_SCHED_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.scheduler.v1;

service SchedulerService {
  rpc TriggerJob(TriggerJobRequest) returns (TriggerJobResponse);
  rpc GetJobExecution(GetJobExecutionRequest) returns (GetJobExecutionResponse);
}

message TriggerJobRequest {
  string job_id = 1;
}

message TriggerJobResponse {
  string execution_id = 1;
  string job_id = 2;
  string status = 3;
  string triggered_at = 4;
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
  string status = 3;
  string triggered_by = 4;
  string started_at = 5;
  optional string finished_at = 6;
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

## 詳細設計ドキュメント

- [system-scheduler-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-scheduler-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。
