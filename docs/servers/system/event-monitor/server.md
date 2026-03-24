# system-event-monitor-server 設計

> **認可モデル注記**: 実装では `resource/action`（例: `event_monitor/read`, `event_monitor/write`, `event_monitor/admin`）で判定し、ロール `sys_admin` / `sys_operator` / `sys_auditor` は middleware でそれぞれ `admin` / `write` / `read` にマッピングされます。


業務イベントフロー全体を可視化し、correlation-id ベースの業務トランザクション追跡・KPI ダッシュボード・障害イベントの選択的リプレイを提供するモニタリングサーバー。tier1 の可観測性スタック（Prometheus/Loki/Jaeger）がインフラ視点であるのに対し、本サーバーは業務視点のモニタリングに特化する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | event_monitor/read |
| sys_operator 以上 | event_monitor/write |
| sys_admin のみ | event_monitor/admin |


system tier の業務イベントモニタリングサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| イベントフロー可視化 | Kafka + event-store のイベントを集約し、業務フロー DAG をリアルタイム構築・提供 |
| トランザクション追跡 | correlation-id ベースで業務トランザクションの全イベントを時系列追跡 |
| フロー定義管理 | 業務フロー定義（期待されるイベントチェーン）の登録・管理 |
| 業務 KPI ダッシュボード | フロー別の完了率・平均処理時間・ボトルネック検出のリアルタイム集計 |
| SLO 監視 | フロー別の SLO 定義・違反検出・業務インパクト推定 |
| 選択的リプレイ | dlq-manager と連携した障害イベントの選択的リプレイ（影響範囲事前確認付き） |
| アラート連携 | SLO 違反・フロー異常検出時に notification-server 経由で業務担当者に通知 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| イベント集約 | Kafka コンシューマー（マルチトピック購読） |
| 時系列集計 | PostgreSQL ウィンドウ関数 + インメモリ集計 |
| キャッシュ | moka v0.12（KPI 集計結果キャッシュ） |

### 配置パス

配置: `regions/system/server/rust/event-monitor/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

### gRPC ポート

proto ファイルおよびサーバー実装のデフォルト: **50051**（config.yaml で上書き可能）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| dlq-manager との違い | dlq-manager は DLQ メッセージの管理・再処理に特化する。event-monitor は正常フロー含む全イベントフローの業務視点モニタリングに特化する |
| 可観測性スタックとの違い | Prometheus/Jaeger はインフラメトリクス・技術トレースを提供する。event-monitor は「注文→出荷→請求のフロー全体で今どこが詰まっているか」を業務担当者が確認する手段を提供する |
| イベント集約方式 | Kafka コンシューマーが `k1s0.*.*.*.v1` パターンの全ドメインイベントを購読し、メタデータ（correlation_id, event_type, timestamp）を抽出して DB に永続化。ペイロード本体は保存しない（容量節約） |
| フロー定義 | 業務フロー（期待されるイベントチェーン）を JSON で定義。例: `TaskCreated → BoardReserved → ActivityProcessed → TaskCompleted` |
| SLO 定義 | フロー別に「完了までの目標時間」「許容エラー率」を定義。バーンレート計算で違反を早期検出 |
| correlation-id | k1s0-correlation ライブラリの correlation_id をキーとして、フロー横断のイベントチェーンを構築 |
| DB スキーマ | PostgreSQL の `event_monitor` スキーマ（event_records, flow_definitions, flow_instances, flow_slos テーブル） |
| Kafka | コンシューマー（全ドメインイベント購読）。プロデューサーなし |
| ポート | 8112（REST）/ 50051（gRPC） |

---

## フロー定義形式

### FlowDefinition

```json
{
  "name": "task_assignment",
  "description": "タスクアサインメントフロー",
  "domain": "service.task",
  "steps": [
    {
      "event_type": "TaskCreated",
      "source": "task-service",
      "timeout_seconds": 0,
      "description": "タスク作成"
    },
    {
      "event_type": "BoardReserved",
      "source": "board-service",
      "timeout_seconds": 30,
      "description": "ボード割当（タスク作成から30秒以内）"
    },
    {
      "event_type": "ActivityProcessed",
      "source": "activity-service",
      "timeout_seconds": 60,
      "description": "アクティビティ処理（ボード割当から60秒以内）"
    },
    {
      "event_type": "TaskCompleted",
      "source": "task-service",
      "timeout_seconds": 10,
      "description": "タスク完了（アクティビティ処理から10秒以内）"
    }
  ],
  "slo": {
    "target_completion_seconds": 120,
    "target_success_rate": 0.995,
    "alert_on_violation": true
  }
}
```

| フィールド | 説明 |
| --- | --- |
| `steps[].timeout_seconds` | 前ステップからの許容時間。0 はフロー開始イベント |
| `slo.target_completion_seconds` | フロー全体の完了目標時間 |
| `slo.target_success_rate` | フロー完了率の目標（0.0〜1.0） |
| `slo.alert_on_violation` | SLO 違反時に notification-server へ通知するか |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_EVMON_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/events` | イベント一覧取得（フィルタ付き） | `sys_auditor` 以上 |
| GET | `/api/v1/events/trace/{correlation_id}` | correlation-id によるイベントチェーン取得 | `sys_auditor` 以上 |
| GET | `/api/v1/flows` | フロー定義一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/flows/{id}` | フロー定義詳細取得 | `sys_auditor` 以上 |
| POST | `/api/v1/flows` | フロー定義作成 | `sys_operator` 以上 |
| PUT | `/api/v1/flows/{id}` | フロー定義更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/flows/{id}` | フロー定義削除 | `sys_admin` のみ |
| GET | `/api/v1/flows/{id}/instances` | フローインスタンス一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/flows/{id}/instances/{instance_id}` | フローインスタンス詳細取得 | `sys_auditor` 以上 |
| GET | `/api/v1/flows/{id}/kpi` | フロー別 KPI 取得 | `sys_auditor` 以上 |
| GET | `/api/v1/kpi/summary` | 全フロー KPI サマリー | `sys_auditor` 以上 |
| GET | `/api/v1/slo/status` | SLO ステータス一覧 | `sys_auditor` 以上 |
| GET | `/api/v1/slo/{flow_id}/burn-rate` | フロー別バーンレート取得 | `sys_auditor` 以上 |
| POST | `/api/v1/replay/preview` | リプレイ影響範囲プレビュー | `sys_operator` 以上 |
| POST | `/api/v1/replay/execute` | リプレイ実行（dlq-manager 連携） | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/events

イベント一覧をフィルタ付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |
| `domain` | string | No | - | ドメインでフィルタ（例: `service.task`） |
| `event_type` | string | No | - | イベントタイプでフィルタ |
| `source` | string | No | - | イベントソースでフィルタ |
| `from` | string | No | - | 開始日時（ISO 8601） |
| `to` | string | No | - | 終了日時（ISO 8601） |
| `status` | string | No | - | `normal` / `timeout` / `error` でフィルタ |

**レスポンス例（200 OK）**

```json
{
  "events": [
    {
      "id": "evt-001",
      "correlation_id": "corr-12345",
      "event_type": "TaskCreated",
      "source": "task-service",
      "domain": "service.task",
      "trace_id": "abc123def456",
      "timestamp": "2026-03-05T10:00:00.000+00:00",
      "flow_id": "flow-001",
      "flow_step_index": 0,
      "status": "normal"
    }
  ],
  "pagination": {
    "total_count": 15230,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/events/trace/{correlation_id}

correlation-id に紐づく全イベントを時系列順に取得し、フローの進行状況を可視化する。

**レスポンス例（200 OK）**

```json
{
  "correlation_id": "corr-12345",
  "flow": {
    "id": "flow-001",
    "name": "task_assignment",
    "status": "in_progress",
    "started_at": "2026-03-05T10:00:00.000+00:00",
    "elapsed_seconds": 45
  },
  "events": [
    {
      "id": "evt-001",
      "event_type": "TaskCreated",
      "source": "task-service",
      "timestamp": "2026-03-05T10:00:00.000+00:00",
      "step_index": 0,
      "status": "completed",
      "duration_from_previous_ms": 0
    },
    {
      "id": "evt-002",
      "event_type": "BoardReserved",
      "source": "board-service",
      "timestamp": "2026-03-05T10:00:15.000+00:00",
      "step_index": 1,
      "status": "completed",
      "duration_from_previous_ms": 15000
    },
    {
      "id": "evt-003",
      "event_type": "ActivityProcessed",
      "source": "activity-service",
      "timestamp": "2026-03-05T10:00:45.000+00:00",
      "step_index": 2,
      "status": "completed",
      "duration_from_previous_ms": 30000
    }
  ],
  "pending_steps": [
    {
      "event_type": "TaskCompleted",
      "source": "task-service",
      "step_index": 3,
      "timeout_seconds": 10,
      "waiting_since_seconds": 0
    }
  ]
}
```

#### GET /api/v1/flows/{id}/kpi

フロー別の KPI を取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `period` | string | No | `1h` | 集計期間（`1h`, `6h`, `24h`, `7d`, `30d`） |

**レスポンス例（200 OK）**

```json
{
  "flow_id": "flow-001",
  "flow_name": "task_assignment",
  "period": "24h",
  "kpi": {
    "total_started": 1250,
    "total_completed": 1200,
    "total_failed": 30,
    "total_in_progress": 20,
    "completion_rate": 0.96,
    "avg_duration_seconds": 85.3,
    "p50_duration_seconds": 72.0,
    "p95_duration_seconds": 145.0,
    "p99_duration_seconds": 210.0,
    "bottleneck_step": {
      "event_type": "ActivityProcessed",
      "step_index": 2,
      "avg_duration_seconds": 42.5,
      "timeout_rate": 0.02
    }
  },
  "slo_status": {
    "target_completion_seconds": 120,
    "target_success_rate": 0.995,
    "current_success_rate": 0.96,
    "is_violated": true,
    "burn_rate": 1.8,
    "estimated_budget_exhaustion_hours": 12.5
  }
}
```

#### GET /api/v1/kpi/summary

全フローの KPI サマリーを取得する。

**レスポンス例（200 OK）**

```json
{
  "period": "24h",
  "flows": [
    {
      "flow_id": "flow-001",
      "flow_name": "task_assignment",
      "domain": "service.task",
      "total_started": 1250,
      "completion_rate": 0.96,
      "avg_duration_seconds": 85.3,
      "slo_violated": true
    },
    {
      "flow_id": "flow-002",
      "flow_name": "project_processing",
      "domain": "business.taskmanagement",
      "total_started": 340,
      "completion_rate": 0.998,
      "avg_duration_seconds": 12.1,
      "slo_violated": false
    }
  ],
  "summary": {
    "total_flows": 8,
    "flows_with_slo_violation": 1,
    "overall_completion_rate": 0.985
  }
}
```

#### GET /api/v1/slo/{flow_id}/burn-rate

SLO バーンレートを取得する。バーンレートが 1.0 を超えるとエラーバジェットが想定より速く消費されていることを意味する。

**レスポンス例（200 OK）**

```json
{
  "flow_id": "flow-001",
  "flow_name": "task_assignment",
  "windows": [
    {
      "window": "1h",
      "burn_rate": 2.5,
      "error_budget_remaining": 0.72
    },
    {
      "window": "6h",
      "burn_rate": 1.8,
      "error_budget_remaining": 0.72
    },
    {
      "window": "24h",
      "burn_rate": 1.2,
      "error_budget_remaining": 0.72
    },
    {
      "window": "30d",
      "burn_rate": 0.95,
      "error_budget_remaining": 0.72
    }
  ],
  "alert_status": "firing",
  "alert_fired_at": "2026-03-05T14:30:00.000+00:00"
}
```

#### POST /api/v1/replay/preview

リプレイ対象のイベントと影響範囲をプレビューする。実際のリプレイは実行しない。

**リクエスト例**

```json
{
  "correlation_ids": ["corr-12345", "corr-12346"],
  "from_step_index": 2,
  "include_downstream": true
}
```

**レスポンス例（200 OK）**

```json
{
  "preview": {
    "total_events_to_replay": 6,
    "affected_services": ["activity-service", "task-service"],
    "affected_flows": [
      {
        "correlation_id": "corr-12345",
        "flow_name": "task_assignment",
        "replay_from_step": 2,
        "events_to_replay": 3
      },
      {
        "correlation_id": "corr-12346",
        "flow_name": "task_assignment",
        "replay_from_step": 2,
        "events_to_replay": 3
      }
    ],
    "dlq_messages_found": 2,
    "estimated_duration_seconds": 15
  }
}
```

#### POST /api/v1/replay/execute

dlq-manager と連携してリプレイを実行する。

**リクエスト例**

```json
{
  "correlation_ids": ["corr-12345"],
  "from_step_index": 2,
  "include_downstream": true,
  "dry_run": false
}
```

**レスポンス例（202 Accepted）**

```json
{
  "replay_id": "replay-001",
  "status": "in_progress",
  "total_events": 3,
  "replayed_events": 0,
  "started_at": "2026-03-05T16:00:00.000+00:00"
}
```

**レスポンス例（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_EVMON_CORRELATION_NOT_FOUND",
    "message": "no events found for correlation_id: corr-99999",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_EVMON_NOT_FOUND` | 404 | 指定されたリソースが見つからない |
| `SYS_EVMON_FLOW_NOT_FOUND` | 404 | 指定されたフロー定義が見つからない |
| `SYS_EVMON_CORRELATION_NOT_FOUND` | 404 | 指定された correlation_id のイベントが見つからない |
| `SYS_EVMON_ALREADY_EXISTS` | 409 | 同一名のフロー定義が既に存在する |
| `SYS_EVMON_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_EVMON_REPLAY_IN_PROGRESS` | 409 | 同一 correlation_id に対するリプレイが既に実行中 |
| `SYS_EVMON_REPLAY_FAILED` | 500 | リプレイ実行の失敗（dlq-manager 連携エラー） |
| `SYS_EVMON_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

gRPC ポート: **50051**

```protobuf
syntax = "proto3";
package k1s0.system.event_monitor.v1;

import "k1s0/system/common/v1/types.proto";

service EventMonitorService {
  rpc ListEvents(ListEventsRequest) returns (ListEventsResponse);
  rpc TraceByCorrelation(TraceByCorrelationRequest) returns (TraceByCorrelationResponse);
  rpc ListFlows(ListFlowsRequest) returns (ListFlowsResponse);
  rpc GetFlow(GetFlowRequest) returns (GetFlowResponse);
  rpc CreateFlow(CreateFlowRequest) returns (CreateFlowResponse);
  rpc UpdateFlow(UpdateFlowRequest) returns (UpdateFlowResponse);
  rpc DeleteFlow(DeleteFlowRequest) returns (DeleteFlowResponse);
  rpc GetFlowInstance(GetFlowInstanceRequest) returns (GetFlowInstanceResponse);
  rpc ListFlowInstances(ListFlowInstancesRequest) returns (ListFlowInstancesResponse);
  rpc GetFlowKpi(GetFlowKpiRequest) returns (GetFlowKpiResponse);
  rpc GetKpiSummary(GetKpiSummaryRequest) returns (GetKpiSummaryResponse);
  rpc GetSloStatus(GetSloStatusRequest) returns (GetSloStatusResponse);
  rpc GetSloBurnRate(GetSloBurnRateRequest) returns (GetSloBurnRateResponse);
  rpc PreviewReplay(PreviewReplayRequest) returns (PreviewReplayResponse);
  rpc ExecuteReplay(ExecuteReplayRequest) returns (ExecuteReplayResponse);
}

// --- Event messages ---

message EventRecord {
  string id = 1;
  string correlation_id = 2;
  string event_type = 3;
  string source_service = 4;
  string domain = 5;
  string trace_id = 6;
  k1s0.system.common.v1.Timestamp timestamp = 7;
  optional string flow_id = 8;
  optional int32 flow_step_index = 9;
  string status = 10;
}

message ListEventsRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  optional string domain = 2;
  optional string event_type = 3;
  optional string source = 4;
  optional k1s0.system.common.v1.Timestamp from = 5;
  optional k1s0.system.common.v1.Timestamp to = 6;
  optional string status = 7;
}

message ListEventsResponse {
  repeated EventRecord events = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message TraceByCorrelationRequest {
  string correlation_id = 1;
}

message TraceEvent {
  string id = 1;
  string event_type = 2;
  string source = 3;
  k1s0.system.common.v1.Timestamp timestamp = 4;
  int32 step_index = 5;
  string status = 6;
  int64 duration_from_previous_ms = 7;
}

message PendingStep {
  string event_type = 1;
  string source = 2;
  int32 step_index = 3;
  int32 timeout_seconds = 4;
  int64 waiting_since_seconds = 5;
}

message FlowSummary {
  string id = 1;
  string name = 2;
  string status = 3;
  k1s0.system.common.v1.Timestamp started_at = 4;
  int64 elapsed_seconds = 5;
}

message TraceByCorrelationResponse {
  string correlation_id = 1;
  FlowSummary flow = 2;
  repeated TraceEvent events = 3;
  repeated PendingStep pending_steps = 4;
}

// --- Flow Definition messages ---

message FlowStep {
  string event_type = 1;
  string source = 2;
  int32 timeout_seconds = 3;
  string description = 4;
}

message FlowSlo {
  int32 target_completion_seconds = 1;
  double target_success_rate = 2;
  bool alert_on_violation = 3;
}

message FlowDefinition {
  string id = 1;
  string name = 2;
  string description = 3;
  string domain = 4;
  repeated FlowStep steps = 5;
  FlowSlo slo = 6;
  bool enabled = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
  k1s0.system.common.v1.Timestamp updated_at = 9;
}

message ListFlowsRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  optional string domain = 2;
}

message ListFlowsResponse {
  repeated FlowDefinition flows = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message GetFlowRequest {
  string id = 1;
}

message GetFlowResponse {
  FlowDefinition flow = 1;
}

message CreateFlowRequest {
  string name = 1;
  string description = 2;
  string domain = 3;
  repeated FlowStep steps = 4;
  FlowSlo slo = 5;
}

message CreateFlowResponse {
  FlowDefinition flow = 1;
}

message UpdateFlowRequest {
  string id = 1;
  optional string description = 2;
  repeated FlowStep steps = 3;
  optional FlowSlo slo = 4;
  optional bool enabled = 5;
}

message UpdateFlowResponse {
  FlowDefinition flow = 1;
}

message DeleteFlowRequest {
  string id = 1;
}

message DeleteFlowResponse {
  bool success = 1;
  string message = 2;
}

// --- FlowInstance messages ---

message GetFlowInstanceRequest {
  string instance_id = 1;
}

message GetFlowInstanceResponse {
  FlowInstance instance = 1;
}

message ListFlowInstancesRequest {
  string flow_id = 1;
  k1s0.system.common.v1.Pagination pagination = 2;
  optional string status = 3;
}

message ListFlowInstancesResponse {
  repeated FlowInstance instances = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message FlowInstance {
  string id = 1;
  string flow_definition_id = 2;
  string correlation_id = 3;
  string status = 4;
  int32 current_step_index = 5;
  k1s0.system.common.v1.Timestamp started_at = 6;
  optional k1s0.system.common.v1.Timestamp completed_at = 7;
  optional int64 duration_ms = 8;
}

// --- KPI messages ---

message BottleneckStep {
  string event_type = 1;
  int32 step_index = 2;
  double avg_duration_seconds = 3;
  double timeout_rate = 4;
}

message SloStatus {
  int32 target_completion_seconds = 1;
  double target_success_rate = 2;
  double current_success_rate = 3;
  bool is_violated = 4;
  double burn_rate = 5;
  double estimated_budget_exhaustion_hours = 6;
}

message FlowKpi {
  int64 total_started = 1;
  int64 total_completed = 2;
  int64 total_failed = 3;
  int64 total_in_progress = 4;
  double completion_rate = 5;
  double avg_duration_seconds = 6;
  double p50_duration_seconds = 7;
  double p95_duration_seconds = 8;
  double p99_duration_seconds = 9;
  BottleneckStep bottleneck_step = 10;
}

message GetFlowKpiRequest {
  string flow_id = 1;
  optional string period = 2;
}

message GetFlowKpiResponse {
  string flow_id = 1;
  string flow_name = 2;
  string period = 3;
  FlowKpi kpi = 4;
  SloStatus slo_status = 5;
}

message FlowKpiSummary {
  string flow_id = 1;
  string flow_name = 2;
  string domain = 3;
  int64 total_started = 4;
  double completion_rate = 5;
  double avg_duration_seconds = 6;
  bool slo_violated = 7;
}

message GetKpiSummaryRequest {
  optional string period = 1;
}

message GetKpiSummaryResponse {
  string period = 1;
  repeated FlowKpiSummary flows = 2;
  int32 total_flows = 3;
  int32 flows_with_slo_violation = 4;
  double overall_completion_rate = 5;
}

// --- SLO messages ---

message GetSloStatusRequest {}

message SloFlowStatus {
  string flow_id = 1;
  string flow_name = 2;
  bool is_violated = 3;
  double burn_rate = 4;
  double error_budget_remaining = 5;
}

message GetSloStatusResponse {
  repeated SloFlowStatus flows = 1;
}

message BurnRateWindow {
  string window = 1;
  double burn_rate = 2;
  double error_budget_remaining = 3;
}

message GetSloBurnRateRequest {
  string flow_id = 1;
}

message GetSloBurnRateResponse {
  string flow_id = 1;
  string flow_name = 2;
  repeated BurnRateWindow windows = 3;
  string alert_status = 4;
  optional k1s0.system.common.v1.Timestamp alert_fired_at = 5;
}

// --- Replay messages ---

message ReplayFlowPreview {
  string correlation_id = 1;
  string flow_name = 2;
  int32 replay_from_step = 3;
  int32 events_to_replay = 4;
}

message PreviewReplayRequest {
  repeated string correlation_ids = 1;
  int32 from_step_index = 2;
  bool include_downstream = 3;
}

message PreviewReplayResponse {
  int32 total_events_to_replay = 1;
  repeated string affected_services = 2;
  repeated ReplayFlowPreview affected_flows = 3;
  int32 dlq_messages_found = 4;
  int32 estimated_duration_seconds = 5;
}

message ExecuteReplayRequest {
  repeated string correlation_ids = 1;
  int32 from_step_index = 2;
  bool include_downstream = 3;
  bool dry_run = 4;
}

message ExecuteReplayResponse {
  string replay_id = 1;
  string status = 2;
  int32 total_events = 3;
  int32 replayed_events = 4;
  k1s0.system.common.v1.Timestamp started_at = 5;
}
```

---

## Kafka コンシューマー設計

### 全ドメインイベント購読

event-monitor は Kafka コンシューマーをバックグラウンドタスクとして起動し、全ドメインイベントのメタデータを収集する。

| 設定項目 | 値 |
| --- | --- |
| トピックパターン | `k1s0.*.*.*.v1`（config.yaml の `kafka.event_topic_pattern`） |
| 除外パターン | `*.dlq.v1`（DLQ トピックは dlq-manager が管理） |
| コンシューマーグループ | `event-monitor.default` |
| auto.offset.reset | `latest`（過去イベントの全取り込みは不要、起動時点以降を対象） |
| enable.auto.commit | `true` |

### メッセージ処理フロー

```
1. Kafka コンシューマーがメッセージを受信
2. メッセージヘッダーから correlation_id, trace_id を抽出
3. メッセージキーから event_type を抽出
4. トピック名から domain を推定（k1s0.{tier}.{domain}.{event_type}.v1）
5. EventRecord エンティティを作成
6. フロー定義とマッチング（domain + event_type でフローステップを特定）
7. フローインスタンスの状態を更新（新規作成 or ステップ進行）
8. タイムアウト検出（前ステップからの経過時間 > timeout_seconds）
9. DB に永続化
10. KPI 集計キャッシュを更新
```

### タイムアウト検出ジョブ

scheduler-server にジョブを登録し、定期的にタイムアウトしたフローインスタンスを検出する。

| 設定項目 | 値 |
| --- | --- |
| ジョブ名 | `event-monitor.check-timeouts` |
| 実行間隔 | 30 秒 |
| 処理内容 | `in_progress` 状態のフローインスタンスを走査し、次ステップの `timeout_seconds` を超過したものを `timeout` ステータスに遷移。SLO 違反の場合は notification-server へ通知 |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `EventRecord`, `FlowDefinition`, `FlowStep`, `FlowInstance`, `FlowSlo`, `FlowKpi`, `SloStatus` | エンティティ定義 |
| domain/repository | `EventRecordRepository`, `FlowDefinitionRepository`, `FlowInstanceRepository` | リポジトリトレイト |
| domain/service | `FlowMatchingService`, `KpiAggregationService`, `SloCalculationService`, `TimeoutDetectionService` | フローマッチング・KPI 集計・SLO 計算・タイムアウト検出 |
| usecase | `ListEventsUseCase`, `TraceByCorrelationUseCase`, `CreateFlowUseCase`, `UpdateFlowUseCase`, `DeleteFlowUseCase`, `GetFlowInstanceUseCase`, `ListFlowInstancesUseCase`, `GetFlowKpiUseCase`, `GetKpiSummaryUseCase`, `GetSloStatusUseCase`, `PreviewReplayUseCase`, `ExecuteReplayUseCase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `EventRecordPostgresRepository`, `FlowDefinitionPostgresRepository`, `FlowInstancePostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/cache | `KpiCacheService` | moka キャッシュ実装（KPI 集計結果） |
| infrastructure/kafka | `EventKafkaConsumer` | Kafka コンシューマー（全ドメインイベント購読） |
| infrastructure/dlq | `DlqManagerClient` | dlq-manager サーバーへの gRPC クライアント（リプレイ実行） |
| infrastructure/notification | `NotificationSender` | notification-server へのアラート送信 |

### ドメインモデル

#### EventRecord

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | イベント記録の一意識別子 |
| `correlation_id` | String | 業務トランザクション相関 ID |
| `event_type` | String | イベントタイプ（例: `TaskCreated`） |
| `source` | String | イベントソース（例: `task-service`） |
| `domain` | String | ドメイン（例: `service.task`） |
| `trace_id` | String | 分散トレース ID |
| `timestamp` | DateTime\<Utc\> | イベント発生日時 |
| `flow_id` | Option\<UUID\> | マッチしたフロー定義 ID |
| `flow_step_index` | Option\<i32\> | フロー内のステップインデックス |
| `status` | String | `normal` / `timeout` / `error` |
| `received_at` | DateTime\<Utc\> | event-monitor がイベントを受信した日時 |

#### FlowDefinition

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | フロー定義の一意識別子 |
| `name` | String | フロー名（例: `task_assignment`） |
| `description` | String | フローの説明 |
| `domain` | String | 業務領域（例: `service.task`） |
| `steps` | Vec\<FlowStep\> | フローステップ一覧 |
| `slo` | FlowSlo | SLO 定義 |
| `enabled` | bool | フロー定義の有効/無効 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### FlowInstance

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | フローインスタンスの一意識別子 |
| `flow_definition_id` | UUID | フロー定義 ID |
| `correlation_id` | String | 業務トランザクション相関 ID |
| `status` | FlowInstanceStatus | `in_progress` / `completed` / `failed` / `timeout` |
| `current_step_index` | i32 | 現在のステップインデックス |
| `started_at` | DateTime\<Utc\> | フロー開始日時 |
| `completed_at` | Option\<DateTime\<Utc\>\> | フロー完了日時 |
| `duration_ms` | Option\<i64\> | フロー完了までの所要時間（ミリ秒） |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (event_monitor_handler.rs)   │   │
                    │  │  healthz / readyz / metrics               │   │
                    │  │  list_events / trace_by_correlation       │   │
                    │  │  list_flows / get_flow / create_flow      │   │
                    │  │  get_flow_kpi / get_kpi_summary           │   │
                    │  │  get_slo_status / get_slo_burn_rate       │   │
                    │  │  preview_replay / execute_replay          │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (tonic_service.rs)           │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  ListEventsUseCase / TraceByCorrelationUseCase  │
                    │  CreateFlowUseCase / GetFlowKpiUseCase /       │
                    │  GetKpiSummaryUseCase / GetSloStatusUseCase /  │
                    │  PreviewReplayUseCase / ExecuteReplayUseCase   │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  EventRecord,   │              │ EventRecordRepository      │   │
    │  FlowDefinition,│              │ FlowDefinitionRepository   │   │
    │  FlowInstance,  │              │ FlowInstanceRepository     │   │
    │  FlowKpi,       │              │ (trait)                    │   │
    │  SloStatus      │              └──────────┬─────────────────┘   │
    └────────────────┘                          │                     │
              │                                 │                     │
              │  ┌──────────────────┐           │                     │
              └──▶ domain/service   │           │                     │
                 │ FlowMatching     │           │                     │
                 │ KpiAggregation   │           │                     │
                 │ SloCalculation   │           │                     │
                 │ TimeoutDetection │           │                     │
                 └──────────────────┘           │                     │
                    ┌───────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌──────▼─────────────────┐  │
                    │  │ Kafka        │  │ EventRecordPostgres    │  │
                    │  │ Consumer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ FlowDefinitionPostgres │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ (KPI)        │  ├────────────────────────┤  │
                    │  └──────────────┘  │ FlowInstancePostgres   │  │
                    │  ┌──────────────┐  │ Repository             │  │
                    │  │ DlqManager   │  └────────────────────────┘  │
                    │  │ Client       │  ┌────────────────────────┐  │
                    │  └──────────────┘  │ Database               │  │
                    │  ┌──────────────┐  │ Config                 │  │
                    │  │ Notification │  └────────────────────────┘  │
                    │  │ Sender       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## DB スキーマ

PostgreSQL の `event_monitor` スキーマに以下のテーブルを配置する。

```sql
CREATE SCHEMA IF NOT EXISTS event_monitor;

CREATE TABLE event_monitor.flow_definitions (
    id                         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                       TEXT NOT NULL,
    description                TEXT NOT NULL DEFAULT '',
    domain                     TEXT NOT NULL,
    steps                      JSONB NOT NULL,
    slo_target_completion_secs INTEGER NOT NULL DEFAULT 300,
    slo_target_success_rate    NUMERIC(5, 4) NOT NULL DEFAULT 0.9950,
    slo_alert_on_violation     BOOLEAN NOT NULL DEFAULT true,
    enabled                    BOOLEAN NOT NULL DEFAULT true,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (domain, name)
);

CREATE TABLE event_monitor.event_records (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    correlation_id   TEXT NOT NULL,
    event_type       TEXT NOT NULL,
    source_service   TEXT NOT NULL,
    domain           TEXT NOT NULL,
    trace_id         TEXT NOT NULL DEFAULT '',
    timestamp        TIMESTAMPTZ NOT NULL,
    flow_id          UUID REFERENCES event_monitor.flow_definitions(id) ON DELETE SET NULL,
    flow_step_index  INTEGER,
    status           TEXT NOT NULL DEFAULT 'normal',
    received_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE event_monitor.flow_instances (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flow_definition_id UUID NOT NULL REFERENCES event_monitor.flow_definitions(id) ON DELETE CASCADE,
    correlation_id     TEXT NOT NULL UNIQUE,
    status             TEXT NOT NULL DEFAULT 'in_progress',
    current_step_index INTEGER NOT NULL DEFAULT 0,
    started_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at       TIMESTAMPTZ,
    duration_ms        BIGINT
);

CREATE INDEX idx_event_records_correlation ON event_monitor.event_records(correlation_id);
CREATE INDEX idx_event_records_domain_type ON event_monitor.event_records(domain, event_type);
CREATE INDEX idx_event_records_timestamp ON event_monitor.event_records(timestamp DESC);
CREATE INDEX idx_event_records_flow_id ON event_monitor.event_records(flow_id);
CREATE INDEX idx_flow_instances_flow_definition_id ON event_monitor.flow_instances(flow_definition_id);
CREATE INDEX idx_flow_instances_status ON event_monitor.flow_instances(status);
CREATE INDEX idx_flow_instances_started_at ON event_monitor.flow_instances(started_at DESC);
```

---

## 設定ファイル例

### config.yaml（本番）
> ※ dev環境では省略可能なセクションがあります。


```yaml
app:
  name: "event-monitor"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8112
  grpc_port: 50051

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "require"          # 開発環境では "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "event-monitor.default"
  security_protocol: "PLAINTEXT"
  event_topic_pattern: "k1s0.*.*.*.v1"
  exclude_pattern: "*.dlq.v1"

dlq_manager:
  grpc_endpoint: "dlq-manager.k1s0-system.svc.cluster.local:50051"
  timeout_ms: 5000

notification:
  endpoint: "http://notification-server.k1s0-system.svc.cluster.local:8092"
  timeout_ms: 3000

scheduler:
  endpoint: "http://scheduler-server.k1s0-system.svc.cluster.local:8089"
  timeout_check_interval_seconds: 30

cache:
  kpi_max_entries: 10000
  kpi_ttl_seconds: 30

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"
  issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
  audience: "k1s0-api"
  jwks_cache_ttl_secs: 300
```

### Helm values

```yaml
# values-event-monitor.yaml（infra/helm/services/system/event-monitor/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/event-monitor
  tag: ""

replicaCount: 2

container:
  port: 8112
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 8
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/event-monitor/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/event-monitor/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## データ保持ポリシー

イベント量が大きくなるため、以下の保持ポリシーを適用する。

| データ | 保持期間 | 削除方式 |
| --- | --- | --- |
| event_records | 90 日 | PostgreSQL パーティション（月次）+ 古いパーティションの DROP |
| flow_instances（completed/failed） | 90 日 | event_records と同期して削除 |
| flow_instances（in_progress） | 無期限 | 状態遷移するまで保持 |
| flow_definitions | 無期限 | 手動削除のみ |

### パーティション設計

```sql
-- event_records は月次パーティション
CREATE TABLE event_monitor.event_records (
    ...
) PARTITION BY RANGE (timestamp);

-- 月次パーティション自動作成は scheduler-server のジョブで実行
-- ジョブ名: event-monitor.create-monthly-partition
-- 実行タイミング: 毎月 25 日（翌月分を事前作成）
```

---

## 詳細設計ドキュメント

- [system-event-monitor-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-event-monitor-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-dlq-manager-server.md](../dlq-manager/server.md) -- DLQ 管理サーバー（リプレイ連携先）
- [system-event-store-server.md](../event-store/server.md) -- イベントストア（イベントソーシング基盤）
- [system-notification-server.md](../notification/server.md) -- 通知サーバー（SLO 違反アラート先）
- [system-scheduler-server.md](../scheduler/server.md) -- スケジューラー（タイムアウト検出ジョブ）
- [system-library-correlation.md](../../libraries/observability/correlation.md) -- 相関 ID ライブラリ
- [可観測性設計.md](../../architecture/observability/可観測性設計.md) -- インフラ視点の可観測性
- [SLO設計.md](../../architecture/observability/SLO設計.md) -- SLO 設計ガイドライン
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック設計
- [REST-API設計.md](../../architecture/api/REST-API設計.md) -- D-007 統一エラーレスポンス

---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。

### サービス固有メトリクス

| メトリクス名 | 型 | 説明 |
| --- | --- | --- |
| `event_monitor_events_received_total` | counter | 受信イベント総数（ラベル: `domain`, `event_type`, `source`） |
| `event_monitor_events_matched_total` | counter | フローにマッチしたイベント数 |
| `event_monitor_events_unmatched_total` | counter | フロー定義にマッチしなかったイベント数 |
| `event_monitor_flow_instances_active` | gauge | 現在 in_progress のフローインスタンス数 |
| `event_monitor_flow_completion_duration_seconds` | histogram | フロー完了までの所要時間（ラベル: `flow_name`） |
| `event_monitor_flow_timeout_total` | counter | タイムアウト検出数（ラベル: `flow_name`, `step_index`） |
| `event_monitor_slo_burn_rate` | gauge | SLO バーンレート（ラベル: `flow_name`, `window`） |
| `event_monitor_replay_total` | counter | リプレイ実行回数 |
| `event_monitor_kpi_cache_hits_total` | counter | KPI キャッシュヒット数 |
