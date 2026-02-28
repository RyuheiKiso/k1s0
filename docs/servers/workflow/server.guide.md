# system-workflow-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/workflows -- レスポンス例

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

### POST /api/v1/workflows -- リクエスト・レスポンス例

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

### GET /api/v1/workflows/:id -- レスポンス例

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

### POST /api/v1/workflows/:id/instances -- リクエスト・レスポンス例

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

### GET /api/v1/instances -- レスポンス例

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

### POST /api/v1/instances/:id/cancel -- リクエスト・レスポンス例

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

### GET /api/v1/tasks -- レスポンス例

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

### POST /api/v1/tasks/:id/approve -- リクエスト・レスポンス例

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

### POST /api/v1/tasks/:id/reject -- リクエスト・レスポンス例

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

### POST /api/v1/tasks/:id/reassign -- リクエスト・レスポンス例

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

---

## Kafka メッセージ例

### インスタンス状態変化メッセージ

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

### タスク状態変化メッセージ

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

### scheduler-server 連携ジョブ定義

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

---

## 依存関係図

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

## 設定ファイル例

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

### Helm values

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
