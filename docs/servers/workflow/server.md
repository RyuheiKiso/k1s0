# system-workflow-server 設計

system tier の人間タスク・承認フロー込みのワークフローオーケストレーションサーバー設計を定義する。BPMN 的なワークフロー定義を管理し、担当者割当・期日・承認/却下を含む人手プロセスを制御する。ワークフロー状態変化は Kafka トピック `k1s0.system.workflow.state_changed.v1` で発行し、タスク期日超過を scheduler-server で監視、notification-server で通知する。Rust での実装を定義する。

## 概要

system tier のワークフローオーケストレーションサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ワークフロー定義管理 | JSON/YAML 形式のワークフロー定義（ステップ・条件分岐・タイムアウト）の CRUD |
| インスタンス管理 | ワークフロー定義からインスタンスを起動し、状態遷移・完了・キャンセルを管理 |
| 人間タスク管理 | 担当者割当・期日設定・承認/却下/再割当の操作を提供 |
| 期日監視・通知連携 | タスク期日超過を scheduler-server でポーリングし、notification-server 経由で担当者へ通知 |
| 状態変化イベント配信 | ワークフロー・タスクの状態変化を Kafka `k1s0.system.workflow.state_changed.v1` で発行 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/workflow/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ワークフロー定義形式 | JSON/YAML で記述したステップ定義（`type: human_task` / `type: automated`）をDBに保存 |
| 状態機械 | インスタンスの状態は `pending` → `running` → `completed` / `cancelled` / `failed` で遷移。タスク単位は `pending` → `assigned` → `approved` / `rejected` |
| 人間タスク | 担当者（`assignee_id`）・期日（`due_at`）を持ち、承認/却下/再割当の操作が可能 |
| saga-server との違い | saga-server はシステム間分散トランザクションに特化する。workflow-server は「申請→承認→通知」等の人手プロセスを管理する |
| scheduler 連携 | タスク期日監視ジョブを scheduler-server に登録し、期日超過タスクを定期チェックして notification-server へ通知依頼 |
| DB | PostgreSQL の `workflow` スキーマ（workflow_definitions, workflow_instances, workflow_tasks テーブル） |
| Kafka | プロデューサー（`k1s0.system.workflow.state_changed.v1`） |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |
| ポート | ホスト側 8100（内部 8080）、gRPC 9090 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_WORKFLOW_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| GET | `/api/v1/workflows` | ワークフロー定義一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/workflows` | ワークフロー定義作成 | `sys_operator` 以上 |
| GET | `/api/v1/workflows/:id` | ワークフロー定義取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/workflows/:id` | ワークフロー定義更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/workflows/:id` | ワークフロー定義削除 | `sys_admin` のみ |
| POST | `/api/v1/workflows/:id/execute` | インスタンス起動（実行開始） | `sys_operator` 以上 |
| GET | `/api/v1/workflows/:id/status` | ワークフロー実行ステータス取得 | `sys_auditor` 以上 |
| GET | `/api/v1/instances` | インスタンス一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/instances/:id` | インスタンス取得 | `sys_auditor` 以上 |
| POST | `/api/v1/instances/:id/cancel` | インスタンスキャンセル | `sys_operator` 以上 |
| GET | `/api/v1/tasks` | 人間タスク一覧取得（担当者フィルタ可） | `sys_auditor` 以上 |
| POST | `/api/v1/tasks/:id/approve` | タスク承認 | `sys_operator` 以上 |
| POST | `/api/v1/tasks/:id/reject` | タスク却下 | `sys_operator` 以上 |
| POST | `/api/v1/tasks/:id/reassign` | タスク再割当 | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### GET /api/v1/workflows

ワークフロー定義一覧をページネーション付きで取得する。`enabled_only` クエリパラメータで有効な定義のみに絞り込める。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `enabled_only` | bool | No | false | 有効な定義のみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "workflows": [
    {
      "id": "wf_01JABCDEF1234567890",
      "name": "purchase-approval",
      "description": "購買申請承認フロー",
      "version": 2,
      "enabled": true,
      "step_count": 3,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 5,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

#### POST /api/v1/workflows

新しいワークフロー定義を作成する。`steps` はステップ定義の配列で、各ステップは `type: human_task` または `type: automated` を指定する。

**リクエスト**

```json
{
  "name": "purchase-approval",
  "description": "購買申請承認フロー",
  "enabled": true,
  "steps": [
    {
      "step_id": "step-1",
      "name": "部門長承認",
      "type": "human_task",
      "assignee_role": "dept_manager",
      "timeout_hours": 48,
      "on_approve": "step-2",
      "on_reject": "end"
    },
    {
      "step_id": "step-2",
      "name": "経理部承認",
      "type": "human_task",
      "assignee_role": "finance_approver",
      "timeout_hours": 72,
      "on_approve": "end",
      "on_reject": "step-1"
    }
  ]
}
```

**レスポンス（201 Created）**

```json
{
  "id": "wf_01JABCDEF1234567890",
  "name": "purchase-approval",
  "description": "購買申請承認フロー",
  "version": 1,
  "enabled": true,
  "steps": [
    {
      "step_id": "step-1",
      "name": "部門長承認",
      "type": "human_task",
      "assignee_role": "dept_manager",
      "timeout_hours": 48,
      "on_approve": "step-2",
      "on_reject": "end"
    },
    {
      "step_id": "step-2",
      "name": "経理部承認",
      "type": "human_task",
      "assignee_role": "finance_approver",
      "timeout_hours": 72,
      "on_approve": "end",
      "on_reject": "step-1"
    }
  ],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_WORKFLOW_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "steps[0].on_approve", "message": "referenced step_id 'step-2' does not exist"},
      {"field": "name", "message": "name is required and must be non-empty"}
    ]
  }
}
```

#### GET /api/v1/workflows/:id

ID 指定でワークフロー定義の詳細（ステップ定義を含む）を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "wf_01JABCDEF1234567890",
  "name": "purchase-approval",
  "description": "購買申請承認フロー",
  "version": 2,
  "enabled": true,
  "steps": [
    {
      "step_id": "step-1",
      "name": "部門長承認",
      "type": "human_task",
      "assignee_role": "dept_manager",
      "timeout_hours": 48,
      "on_approve": "step-2",
      "on_reject": "end"
    }
  ],
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_WORKFLOW_NOT_FOUND",
    "message": "workflow definition not found: wf_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/workflows/:id/instances

指定したワークフロー定義からインスタンスを起動する。起動後すぐに最初のステップのタスクが生成される。

**リクエスト**

```json
{
  "title": "PC購入申請（田中太郎）",
  "initiator_id": "user-001",
  "context": {
    "item": "ノートPC",
    "amount": 150000,
    "requester": "tanaka@example.com"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "inst_01JABCDEF1234567890",
  "workflow_id": "wf_01JABCDEF1234567890",
  "workflow_name": "purchase-approval",
  "title": "PC購入申請（田中太郎）",
  "initiator_id": "user-001",
  "current_step_id": "step-1",
  "status": "running",
  "context": {
    "item": "ノートPC",
    "amount": 150000,
    "requester": "tanaka@example.com"
  },
  "started_at": "2026-02-20T10:00:00.000+00:00",
  "completed_at": null,
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

#### GET /api/v1/instances

インスタンス一覧をページネーション付きで取得する。`status` / `workflow_id` / `initiator_id` でフィルタリングできる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `status` | string | No | - | インスタンス状態でフィルタ（pending/running/completed/cancelled/failed） |
| `workflow_id` | string | No | - | ワークフロー定義 ID でフィルタ |
| `initiator_id` | string | No | - | 起動者 ID でフィルタ |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "instances": [
    {
      "id": "inst_01JABCDEF1234567890",
      "workflow_id": "wf_01JABCDEF1234567890",
      "workflow_name": "purchase-approval",
      "title": "PC購入申請（田中太郎）",
      "initiator_id": "user-001",
      "current_step_id": "step-1",
      "status": "running",
      "started_at": "2026-02-20T10:00:00.000+00:00",
      "completed_at": null,
      "created_at": "2026-02-20T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 50,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### POST /api/v1/instances/:id/cancel

実行中のインスタンスをキャンセルする。`completed` / `cancelled` 済みのインスタンスには適用できない。

**リクエスト**

```json
{
  "reason": "申請内容に誤りがあったため取り消し"
}
```

**レスポンス（200 OK）**

```json
{
  "id": "inst_01JABCDEF1234567890",
  "status": "cancelled",
  "cancelled_at": "2026-02-20T15:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_WORKFLOW_INVALID_STATUS",
    "message": "cannot cancel an already completed instance: inst_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/tasks

人間タスク一覧をページネーション付きで取得する。`assignee_id` を指定すると特定担当者のタスクのみ取得できる。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `assignee_id` | string | No | - | 担当者 ID でフィルタ |
| `status` | string | No | - | タスク状態でフィルタ（pending/assigned/approved/rejected） |
| `overdue_only` | bool | No | false | 期日超過のタスクのみ取得 |
| `instance_id` | string | No | - | インスタンス ID でフィルタ |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "tasks": [
    {
      "id": "task_01JABCDEF1234567890",
      "instance_id": "inst_01JABCDEF1234567890",
      "step_id": "step-1",
      "step_name": "部門長承認",
      "assignee_id": "user-002",
      "status": "assigned",
      "due_at": "2026-02-22T10:00:00.000+00:00",
      "is_overdue": false,
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

#### POST /api/v1/tasks/:id/approve

タスクを承認する。承認後は次のステップへ遷移し、次タスクが生成される。最終ステップの承認ではインスタンスが `completed` になる。

**リクエスト**

```json
{
  "comment": "内容を確認しました。承認します。",
  "actor_id": "user-002"
}
```

**レスポンス（200 OK）**

```json
{
  "task_id": "task_01JABCDEF1234567890",
  "status": "approved",
  "next_task_id": "task_01JABCDEF9876543210",
  "instance_status": "running",
  "decided_at": "2026-02-20T14:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_WORKFLOW_TASK_NOT_FOUND",
    "message": "workflow task not found: task_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/tasks/:id/reject

タスクを却下する。`on_reject` の設定に基づいて前のステップへ差し戻すか、インスタンスを `failed` で終了する。

**リクエスト**

```json
{
  "comment": "金額が予算上限を超過しているため却下します。",
  "actor_id": "user-002"
}
```

**レスポンス（200 OK）**

```json
{
  "task_id": "task_01JABCDEF1234567890",
  "status": "rejected",
  "next_task_id": null,
  "instance_status": "failed",
  "decided_at": "2026-02-20T14:00:00.000+00:00"
}
```

#### POST /api/v1/tasks/:id/reassign

タスクの担当者を変更する。`status` が `pending` または `assigned` のタスクのみ再割当可能。

**リクエスト**

```json
{
  "new_assignee_id": "user-003",
  "reason": "担当者変更のため",
  "actor_id": "user-002"
}
```

**レスポンス（200 OK）**

```json
{
  "task_id": "task_01JABCDEF1234567890",
  "previous_assignee_id": "user-002",
  "new_assignee_id": "user-003",
  "reassigned_at": "2026-02-20T13:00:00.000+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_WORKFLOW_INVALID_STATUS",
    "message": "cannot reassign a task with status 'approved': task_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_WORKFLOW_NOT_FOUND` | 404 | 指定されたワークフロー定義が見つからない |
| `SYS_WORKFLOW_INSTANCE_NOT_FOUND` | 404 | 指定されたインスタンスが見つからない |
| `SYS_WORKFLOW_TASK_NOT_FOUND` | 404 | 指定されたタスクが見つからない |
| `SYS_WORKFLOW_ALREADY_EXISTS` | 409 | 同一名のワークフロー定義が既に存在する |
| `SYS_WORKFLOW_INVALID_STATUS` | 409 | 操作に対してインスタンスまたはタスクのステータスが不正 |
| `SYS_WORKFLOW_VALIDATION_ERROR` | 400 | リクエストまたはワークフロー定義のバリデーションエラー |
| `SYS_WORKFLOW_STEP_REF_ERROR` | 400 | ステップ定義で参照先 step_id が不正 |
| `SYS_WORKFLOW_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.workflow.v1;

service WorkflowService {
  rpc StartInstance(StartInstanceRequest) returns (StartInstanceResponse);
  rpc GetInstance(GetInstanceRequest) returns (GetInstanceResponse);
  rpc ApproveTask(ApproveTaskRequest) returns (ApproveTaskResponse);
  rpc RejectTask(RejectTaskRequest) returns (RejectTaskResponse);
}

message StartInstanceRequest {
  string workflow_id = 1;
  string title = 2;
  string initiator_id = 3;
  bytes context_json = 4;
}

message StartInstanceResponse {
  string instance_id = 1;
  string status = 2;
  string current_step_id = 3;
  string started_at = 4;
}

message GetInstanceRequest {
  string instance_id = 1;
}

message GetInstanceResponse {
  WorkflowInstance instance = 1;
}

message WorkflowInstance {
  string id = 1;
  string workflow_id = 2;
  string workflow_name = 3;
  string title = 4;
  string initiator_id = 5;
  string current_step_id = 6;
  string status = 7;
  bytes context_json = 8;
  string started_at = 9;
  optional string completed_at = 10;
}

message ApproveTaskRequest {
  string task_id = 1;
  string actor_id = 2;
  optional string comment = 3;
}

message ApproveTaskResponse {
  string task_id = 1;
  string status = 2;
  optional string next_task_id = 3;
  string instance_status = 4;
}

message RejectTaskRequest {
  string task_id = 1;
  string actor_id = 2;
  optional string comment = 3;
}

message RejectTaskResponse {
  string task_id = 1;
  string status = 2;
  optional string next_task_id = 3;
  string instance_status = 4;
}
```

---

## Kafka メッセージング設計

### ワークフロー状態変化通知

ワークフローインスタンスまたはタスクの状態変化時に以下のメッセージを Kafka トピック `k1s0.system.workflow.state_changed.v1` に送信する。

**メッセージフォーマット（インスタンス状態変化）**

```json
{
  "event_type": "INSTANCE_STATE_CHANGED",
  "instance_id": "inst_01JABCDEF1234567890",
  "workflow_id": "wf_01JABCDEF1234567890",
  "workflow_name": "purchase-approval",
  "previous_status": "running",
  "current_status": "completed",
  "current_step_id": null,
  "timestamp": "2026-02-20T15:00:00.000+00:00",
  "actor_id": "user-002"
}
```

**メッセージフォーマット（タスク状態変化）**

```json
{
  "event_type": "TASK_STATE_CHANGED",
  "task_id": "task_01JABCDEF1234567890",
  "instance_id": "inst_01JABCDEF1234567890",
  "step_id": "step-1",
  "step_name": "部門長承認",
  "previous_status": "assigned",
  "current_status": "approved",
  "assignee_id": "user-002",
  "timestamp": "2026-02-20T14:00:00.000+00:00",
  "actor_id": "user-002"
}
```

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.workflow.state_changed.v1` |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | インスタンス ID（例: `inst_01JABCDEF1234567890`） |

### scheduler-server 連携

タスク期日超過の監視は scheduler-server に以下のジョブを登録して行う。

```json
{
  "name": "workflow-task-due-checker",
  "description": "ワークフロータスク期日超過チェック（15分ごと）",
  "cron_expression": "*/15 * * * *",
  "timezone": "UTC",
  "target_type": "http",
  "target": "http://workflow.k1s0-system.svc.cluster.local:8080/internal/tasks/check-overdue",
  "payload": {}
}
```

期日超過タスクが検出された場合は notification-server の Kafka トピック `k1s0.system.notification.requested.v1` へメッセージを送信する。

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `WorkflowDefinition`, `WorkflowStep`, `WorkflowInstance`, `WorkflowTask` | エンティティ定義 |
| domain/repository | `WorkflowDefinitionRepository`, `WorkflowInstanceRepository`, `WorkflowTaskRepository` | リポジトリトレイト |
| domain/service | `WorkflowDomainService` | ステップ遷移計算・タスク生成・期日計算・状態機械ロジック |
| usecase | `CreateWorkflowUsecase`, `UpdateWorkflowUsecase`, `DeleteWorkflowUsecase`, `GetWorkflowUsecase`, `ListWorkflowsUsecase`, `StartInstanceUsecase`, `GetInstanceUsecase`, `ListInstancesUsecase`, `CancelInstanceUsecase`, `ListTasksUsecase`, `ApproveTaskUsecase`, `RejectTaskUsecase`, `ReassignTaskUsecase`, `CheckOverdueTasksUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `WorkflowDefinitionPostgresRepository`, `WorkflowInstancePostgresRepository`, `WorkflowTaskPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/messaging | `WorkflowStateChangedKafkaProducer`, `NotificationRequestKafkaProducer` | Kafka プロデューサー |

### ドメインモデル

#### WorkflowDefinition

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ワークフロー定義の一意識別子 |
| `name` | String | ワークフロー定義名（例: `purchase-approval`） |
| `description` | String | ワークフローの説明 |
| `version` | u32 | 定義バージョン（更新ごとにインクリメント） |
| `enabled` | bool | 有効/無効フラグ |
| `steps` | Vec\<WorkflowStep\> | ステップ定義の配列 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### WorkflowStep

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `step_id` | String | ステップ識別子（定義内で一意） |
| `name` | String | ステップ表示名 |
| `step_type` | String | ステップ種別（`human_task` / `automated`） |
| `assignee_role` | Option\<String\> | 担当者ロール（`human_task` 時に使用） |
| `timeout_hours` | Option\<u32\> | タイムアウト時間（時間） |
| `on_approve` | Option\<String\> | 承認時の遷移先 step_id（`end` で完了） |
| `on_reject` | Option\<String\> | 却下時の遷移先 step_id（`end` で失敗終了） |

#### WorkflowInstance

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | インスタンスの一意識別子 |
| `workflow_id` | String | 元のワークフロー定義 ID |
| `workflow_name` | String | ワークフロー定義名（スナップショット） |
| `title` | String | インスタンスのタイトル |
| `initiator_id` | String | 起動者のユーザー ID |
| `current_step_id` | Option\<String\> | 現在実行中のステップ ID |
| `status` | String | インスタンス状態（`pending` / `running` / `completed` / `cancelled` / `failed`） |
| `context` | serde_json::Value | ワークフロー実行コンテキスト（任意の JSON） |
| `started_at` | DateTime\<Utc\> | 起動日時 |
| `completed_at` | Option\<DateTime\<Utc\>\> | 完了日時 |

#### WorkflowTask

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | タスクの一意識別子 |
| `instance_id` | String | 所属インスタンス ID |
| `step_id` | String | 対応するステップ ID |
| `step_name` | String | ステップ表示名（スナップショット） |
| `assignee_id` | Option\<String\> | 担当者のユーザー ID |
| `status` | String | タスク状態（`pending` / `assigned` / `approved` / `rejected`） |
| `due_at` | Option\<DateTime\<Utc\>\> | 期日 |
| `comment` | Option\<String\> | 承認/却下コメント |
| `actor_id` | Option\<String\> | 操作実行者のユーザー ID |
| `decided_at` | Option\<DateTime\<Utc\>\> | 決裁日時 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (workflow_handler.rs)       │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_workflows / create_workflow        │   │
                    │  │  get_workflow / update_workflow          │   │
                    │  │  delete_workflow                         │   │
                    │  │  start_instance / list_instances         │   │
                    │  │  get_instance / cancel_instance          │   │
                    │  │  list_tasks / approve_task               │   │
                    │  │  reject_task / reassign_task             │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (workflow_grpc.rs)          │   │
                    │  │  StartInstance / GetInstance             │   │
                    │  │  ApproveTask / RejectTask                │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateWorkflowUsecase / UpdateWorkflowUsecase /│
                    │  DeleteWorkflowUsecase / GetWorkflowUsecase /   │
                    │  ListWorkflowsUsecase / StartInstanceUsecase /  │
                    │  GetInstanceUsecase / ListInstancesUsecase /    │
                    │  CancelInstanceUsecase / ListTasksUsecase /     │
                    │  ApproveTaskUsecase / RejectTaskUsecase /       │
                    │  ReassignTaskUsecase / CheckOverdueTasksUsecase │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  WorkflowDef,   │              │ WorkflowDefinitionRepo     │   │
    │  WorkflowStep,  │              │ WorkflowInstanceRepo       │   │
    │  WorkflowInst,  │              │ WorkflowTaskRepo           │   │
    │  WorkflowTask   │              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ WorkflowDomain │            │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ WorkflowDefinition     │  │
                    │  │ Producer     │  │ PostgresRepository     │  │
                    │  │ (state /     │  ├────────────────────────┤  │
                    │  │  notif req)  │  │ WorkflowInstance       │  │
                    │  └──────────────┘  │ PostgresRepository     │  │
                    │  ┌──────────────┐  ├────────────────────────┤  │
                    │  │ Config       │  │ WorkflowTask           │  │
                    │  │ Loader       │  │ PostgresRepository     │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## DB スキーマ

PostgreSQL の `workflow` スキーマに以下のテーブルを配置する。

```sql
CREATE SCHEMA IF NOT EXISTS workflow;

CREATE TABLE workflow.workflow_definitions (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    version     INTEGER NOT NULL DEFAULT 1,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    steps_json  JSONB NOT NULL DEFAULT '[]',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE workflow.workflow_instances (
    id              TEXT PRIMARY KEY,
    workflow_id     TEXT NOT NULL REFERENCES workflow.workflow_definitions(id),
    workflow_name   TEXT NOT NULL,
    title           TEXT NOT NULL,
    initiator_id    TEXT NOT NULL,
    current_step_id TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    context_json    JSONB NOT NULL DEFAULT '{}',
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_instances_workflow_id ON workflow.workflow_instances(workflow_id);
CREATE INDEX idx_workflow_instances_initiator_id ON workflow.workflow_instances(initiator_id);
CREATE INDEX idx_workflow_instances_status ON workflow.workflow_instances(status);

CREATE TABLE workflow.workflow_tasks (
    id           TEXT PRIMARY KEY,
    instance_id  TEXT NOT NULL REFERENCES workflow.workflow_instances(id) ON DELETE CASCADE,
    step_id      TEXT NOT NULL,
    step_name    TEXT NOT NULL,
    assignee_id  TEXT,
    status       TEXT NOT NULL DEFAULT 'pending',
    due_at       TIMESTAMPTZ,
    comment      TEXT,
    actor_id     TEXT,
    decided_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workflow_tasks_instance_id ON workflow.workflow_tasks(instance_id);
CREATE INDEX idx_workflow_tasks_assignee_id ON workflow.workflow_tasks(assignee_id);
CREATE INDEX idx_workflow_tasks_status ON workflow.workflow_tasks(status);
CREATE INDEX idx_workflow_tasks_due_at ON workflow.workflow_tasks(due_at)
    WHERE status IN ('pending', 'assigned');
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "workflow"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  state_topic: "k1s0.system.workflow.state_changed.v1"
  notification_topic: "k1s0.system.notification.requested.v1"

scheduler:
  internal_endpoint: "http://scheduler.k1s0-system.svc.cluster.local:8080"

overdue_check:
  cron_expression: "*/15 * * * *"
  timezone: "UTC"
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。workflow 固有の values は以下の通り。

```yaml
# values-workflow.yaml（infra/helm/services/system/workflow/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/workflow
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 9090

service:
  type: ClusterIP
  port: 80
  grpcPort: 9090

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/workflow/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/workflow/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-workflow-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-workflow-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-server.md](../auth/server.md) -- system tier サーバー一覧
- [system-notification-server.md](../notification/server.md) -- 通知連携先
- [system-scheduler-server.md](../scheduler/server.md) -- 期日監視ジョブ連携先
- [system-saga-server.md](../saga/server.md) -- システム間分散トランザクション（saga-server との役割分担）
