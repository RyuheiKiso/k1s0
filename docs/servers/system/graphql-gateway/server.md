# system-graphql-gateway 設計

system tier の GraphQL BFF ゲートウェイ。複数 gRPC バックエンドを単一 GraphQL スキーマに集約する。Rust（async-graphql）実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| GraphQL スキーマ集約 | 認証・設定・テナント・セッション・Vault・スケジューラー・通知・ワークフロー等の system サービスを単一スキーマに統合 |
| DataLoader によるバッチ処理 | N+1 問題を DataLoader で解決し、バックエンドへの呼び出しを最小化 |
| サブスクリプション | WebSocket で公開。現実装は 5 秒ポーリングで状態同期し、将来はイベント駆動へ移行予定 |
| イントロスペクション | 開発環境のみ GraphQL スキーマイントロスペクションを有効化 |
| 認証ミドルウェア | JWT 検証により認証済みリクエストのみを許可 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| GraphQL | async-graphql v7 |
| HTTP Framework | axum 0.8 |

### 配置パス

配置: `regions/system/server/rust/graphql-gateway/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) および [GraphQL設計.md](../../architecture/api/GraphQL設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust（async-graphql v7 + axum） |
| 役割 | GraphQL はクライアント向け集約レイヤー（BFF）としてのみ使用。バックエンドは REST/gRPC を維持 |
| スキーマ管理 | async-graphql のマクロベースでプログラム的にスキーマを構築（コードファースト） |
| バックエンド通信 | tonic gRPC クライアントで各バックエンドサービスを呼び出す |
| N+1 対策 | async-graphql の DataLoader を使用してバッチ化 |
| イントロスペクション | `environment: development` 時のみ有効。本番・ステージングでは無効化 |
| サブスクリプション | axum の WebSocket サポートを使用。`/graphql/ws` エンドポイント |
| 認証 | JWT 検証ミドルウェアを axum レイヤーに組み込み。`Authorization: Bearer` ヘッダー必須 |
| ポート | ホスト側 8095（内部 8080） |

---

## API 定義

### REST / GraphQL エンドポイント

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/graphql` | GraphQL クエリ / ミューテーション | JWT 必須 |
| GET | `/graphql` | GraphQL Playground（development のみ） | 不要 |
| GET | `/graphql/ws` | WebSocket サブスクリプション（Upgrade） | JWT 必須 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /graphql

GraphQL クエリおよびミューテーションを受け付ける。リクエストボディは `application/json` 形式。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `query` | string | Yes | GraphQL クエリ文字列 |
| `variables` | object | No | クエリ変数 |
| `operationName` | string | No | 実行するオペレーション名 |

**エラーレスポンス**: GraphQL 仕様に従い HTTP 200 で `errors` フィールドを返却。JWT 欠落・無効時のみ HTTP 401。

**Mutation の RBAC チェック**:
- `createTenant` / `updateTenant` / `setFeatureFlag` は `has_write_role()` を通して `sys_admin` または `sys_operator` を必須とする
- 権限不足時は `extensions.code = FORBIDDEN` を返却する
- 新規バックエンド（auth / session / vault / scheduler / notification / workflow）のミューテーションも同様に `sys_admin` または `sys_operator` を必須とする

**リクエスト例**

```json
{
  "query": "query GetTenant($id: ID!) { tenant(id: $id) { id name status createdAt } }",
  "variables": {
    "id": "tenant-abc"
  }
}
```

**レスポンス例（200 OK）**

```json
{
  "data": {
    "tenant": {
      "id": "tenant-abc",
      "name": "株式会社サンプル",
      "status": "ACTIVE",
      "createdAt": "2026-02-20T10:00:00.000+00:00"
    }
  }
}
```

**レスポンス例（200 OK -- エラー）**

GraphQL 仕様に従い、エラー時も HTTP 200 を返し `errors` フィールドにエラー情報を含める。

```json
{
  "data": null,
  "errors": [
    {
      "message": "tenant not found: tenant-abc",
      "locations": [{"line": 1, "column": 9}],
      "path": ["tenant"],
      "extensions": {
        "code": "NOT_FOUND",
        "request_id": "req_abc123def456"
      }
    }
  ]
}
```

**レスポンス例（401 Unauthorized）**

JWT が無効または欠落している場合は HTTP 401 を返す（GraphQL レスポンスではなく HTTP エラー）。

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "missing or invalid JWT token",
    "request_id": "req_abc123def456"
  }
}
```

#### GET /graphql/ws（WebSocket サブスクリプション）

`graphql-ws` プロトコルを使用。接続時に `connection_init` メッセージで JWT を送信する。

**接続メッセージ（クライアント送信）**

```json
{
  "type": "connection_init",
  "payload": {
    "Authorization": "Bearer eyJhbGciOiJSUzI1NiJ9..."
  }
}
```

**サブスクリプション例**

```json
{
  "type": "subscribe",
  "id": "sub-001",
  "payload": {
    "query": "subscription OnTenantUpdated($tenantId: ID!) { tenantUpdated(tenantId: $tenantId) { id name status } }"
  }
}
```

#### GET /readyz

バックエンド gRPC サービス（tenant / featureflag / config / navigation / service-catalog / auth / session / vault / scheduler / notification / workflow）の疎通を確認する。全サービスが応答すれば `200 OK`、いずれかが失敗すれば `503 Service Unavailable` を返す。

**レスポンス例（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "tenant_grpc": "ok",
    "featureflag_grpc": "ok",
    "config_grpc": "ok",
    "navigation_grpc": "ok",
    "service_catalog_grpc": "ok",
    "auth_grpc": "ok",
    "session_grpc": "ok",
    "vault_grpc": "ok",
    "scheduler_grpc": "ok",
    "notification_grpc": "ok",
    "workflow_grpc": "ok"
  }
}
```

**レスポンス例（503 Service Unavailable）**

```json
{
  "status": "not_ready",
  "checks": {
    "tenant_grpc": "ok",
    "featureflag_grpc": "error",
    "config_grpc": "ok",
    "navigation_grpc": "ok",
    "service_catalog_grpc": "ok",
    "auth_grpc": "ok",
    "session_grpc": "error",
    "vault_grpc": "ok",
    "scheduler_grpc": "ok",
    "notification_grpc": "ok",
    "workflow_grpc": "ok"
  }
}
```

### GraphQL スキーマ（主要型）

```graphql
type Query {
  tenant(id: ID!): Tenant
  tenants(first: Int, after: String): TenantConnection!
  featureFlag(key: String!): FeatureFlag
  featureFlags(environment: String): [FeatureFlag!]!
  config(key: String!): ConfigEntry
  # Auth
  user(id: ID!): User
  users(pageSize: Int, page: Int, search: String, enabled: Boolean): [User!]!
  userRoles(userId: ID!): [Role!]!
  checkPermission(userId: ID, permission: String!, resource: String!, roles: [String!]!): PermissionCheck!
  searchAuditLogs(pageSize: Int, page: Int, userId: String, eventType: String, result: String): AuditLogConnection!
  # Session
  session(sessionId: ID!): Session
  userSessions(userId: ID!): [Session!]!
  # Vault
  secretMetadata(path: String!): SecretMetadata
  secrets(prefix: String): [String!]!
  vaultAuditLogs(offset: Int, limit: Int): [VaultAuditLogEntry!]!
  # Scheduler
  job(jobId: ID!): Job
  jobs(status: String, pageSize: Int, page: Int): [Job!]!
  jobExecution(executionId: ID!): JobExecution
  jobExecutions(jobId: ID!, pageSize: Int, page: Int, status: String): [JobExecution!]!
  # Notification
  notification(notificationId: ID!): NotificationLog
  notifications(channelId: String, status: String, page: Int, pageSize: Int): [NotificationLog!]!
  notificationChannel(id: ID!): NotificationChannel
  notificationChannels(channelType: String, enabledOnly: Boolean!, page: Int, pageSize: Int): [NotificationChannel!]!
  notificationTemplate(id: ID!): NotificationTemplate
  notificationTemplates(channelType: String, page: Int, pageSize: Int): [NotificationTemplate!]!
  # Workflow
  workflow(workflowId: ID!): WorkflowDefinition
  workflows(enabledOnly: Boolean!, pageSize: Int, page: Int): [WorkflowDefinition!]!
  workflowInstance(instanceId: ID!): WorkflowInstance
  workflowInstances(status: String, workflowId: String, initiatorId: String, pageSize: Int, page: Int): [WorkflowInstance!]!
  workflowTasks(assigneeId: String, status: String, instanceId: String, overdueOnly: Boolean!, pageSize: Int, page: Int): [WorkflowTask!]!
}

type Mutation {
  createTenant(input: CreateTenantInput!): CreateTenantPayload!
  updateTenant(id: ID!, input: UpdateTenantInput!): UpdateTenantPayload!
  setFeatureFlag(key: String!, input: SetFeatureFlagInput!): SetFeatureFlagPayload!
  # Auth
  recordAuditLog(input: RecordAuditLogInput!): RecordAuditLogPayload!
  # Session
  createSession(input: CreateSessionInput!): CreateSessionPayload!
  refreshSession(sessionId: ID!, ttlSeconds: Int): RefreshSessionPayload!
  revokeSession(sessionId: ID!): RevokeSessionPayload!
  revokeAllSessions(userId: ID!): RevokeAllSessionsPayload!
  # Vault
  setSecret(input: SetSecretInput!): SetSecretPayload!
  rotateSecret(path: String!, data: [SecretDataInput!]!): RotateSecretPayload!
  deleteSecret(path: String!, versions: [Int!]!): DeleteSecretPayload!
  # Scheduler
  createJob(input: CreateJobInput!): CreateJobPayload!
  updateJob(input: UpdateJobInput!): UpdateJobPayload!
  deleteJob(jobId: ID!): DeleteJobPayload!
  pauseJob(jobId: ID!): PauseJobPayload!
  resumeJob(jobId: ID!): ResumeJobPayload!
  triggerJob(jobId: ID!): TriggerJobPayload!
  # Notification
  sendNotification(input: SendNotificationInput!): SendNotificationPayload!
  retryNotification(notificationId: ID!): RetryNotificationPayload!
  createChannel(input: CreateChannelInput!): CreateChannelPayload!
  updateChannel(input: UpdateChannelInput!): UpdateChannelPayload!
  deleteChannel(id: ID!): DeleteChannelPayload!
  createTemplate(input: CreateTemplateInput!): CreateTemplatePayload!
  updateTemplate(input: UpdateTemplateInput!): UpdateTemplatePayload!
  deleteTemplate(id: ID!): DeleteTemplatePayload!
  # Workflow
  createWorkflow(input: CreateWorkflowInput!): CreateWorkflowPayload!
  updateWorkflow(input: UpdateWorkflowInput!): UpdateWorkflowPayload!
  deleteWorkflow(workflowId: ID!): DeleteWorkflowPayload!
  startWorkflowInstance(input: StartInstanceInput!): StartInstancePayload!
  cancelWorkflowInstance(instanceId: ID!, reason: String): CancelInstancePayload!
  reassignTask(input: ReassignTaskInput!): ReassignTaskPayload!
  approveTask(input: TaskDecisionInput!): ApproveTaskPayload!
  rejectTask(input: TaskDecisionInput!): RejectTaskPayload!
}

type Subscription {
  tenantUpdated(tenantId: ID!): Tenant!
  featureFlagChanged(key: String!): FeatureFlag!
  configChanged(namespaces: [String!] = []): ConfigEntry!
}

> `Subscription` は全サブスクリプション（`configChanged` / `tenantUpdated` / `featureFlagChanged`）で gRPC Server-Side Streaming を使用したイベント駆動方式で実装済み。

type Tenant {
  id: ID!
  name: String!
  status: TenantStatus!
  createdAt: String!
  updatedAt: String!
}

enum TenantStatus {
  ACTIVE
  SUSPENDED
  DELETED
}

type FeatureFlag {
  key: String!
  name: String!
  enabled: Boolean!
  rolloutPercentage: Int!
  targetEnvironments: [String!]!
}

type ConfigEntry {
  key: String!
  value: String!
  updatedAt: String!
}

type TenantConnection {
  edges: [TenantEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type TenantEdge {
  node: Tenant!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

type CreateTenantPayload {
  tenant: Tenant
  errors: [UserError!]!
}

type UpdateTenantPayload {
  tenant: Tenant
  errors: [UserError!]!
}

type SetFeatureFlagPayload {
  featureFlag: FeatureFlag
  errors: [UserError!]!
}

type UserError {
  field: [String!]
  message: String!
}

input CreateTenantInput {
  name: String!
}

# createTenant の owner_user_id は入力では受け取らず、JWT claims.sub から暗黙的に設定する

input UpdateTenantInput {
  name: String
  status: TenantStatus
}

input SetFeatureFlagInput {
  enabled: Boolean!
  rolloutPercentage: Int
  targetEnvironments: [String!]
}

# ── Auth ──
type User {
  id: ID!
  username: String!
  email: String!
  firstName: String!
  lastName: String!
  enabled: Boolean!
  emailVerified: Boolean!
  createdAt: String!
}

type Role {
  id: String!
  name: String!
  description: String!
}

type PermissionCheck {
  allowed: Boolean!
  reason: String!
}

type AuditLog {
  id: String!
  eventType: String!
  userId: String!
  ipAddress: String!
  userAgent: String!
  resource: String!
  action: String!
  result: String!
  resourceId: String!
  traceId: String!
  createdAt: String!
}

type AuditLogConnection {
  logs: [AuditLog!]!
  totalCount: Int!
  hasNext: Boolean!
}

# ── Session ──
type Session {
  sessionId: String!
  userId: String!
  deviceId: String!
  deviceName: String
  deviceType: String
  userAgent: String
  ipAddress: String
  status: SessionStatus!
  expiresAt: String!
  createdAt: String!
  lastAccessedAt: String
}

enum SessionStatus {
  ACTIVE
  REVOKED
}

# ── Vault ──
type SecretMetadata {
  path: String!
  currentVersion: Int!
  versionCount: Int!
  createdAt: String!
  updatedAt: String!
}

type VaultAuditLogEntry {
  id: String!
  keyPath: String!
  action: String!
  actorId: String!
  ipAddress: String!
  success: Boolean!
  errorMsg: String
  createdAt: String!
}

# ── Scheduler ──
type Job {
  id: String!
  name: String!
  description: String!
  cronExpression: String!
  timezone: String!
  targetType: String!
  target: String!
  status: String!
  nextRunAt: String
  lastRunAt: String
  createdAt: String!
  updatedAt: String!
}

type JobExecution {
  id: String!
  jobId: String!
  status: String!
  triggeredBy: String!
  startedAt: String!
  finishedAt: String
  durationMs: Int
  errorMessage: String
}

# ── Notification ──
type NotificationLog {
  id: String!
  channelId: String!
  channelType: String!
  templateId: String
  recipient: String!
  subject: String
  body: String!
  status: String!
  retryCount: Int!
  errorMessage: String
  sentAt: String
  createdAt: String!
}

type NotificationChannel {
  id: String!
  name: String!
  channelType: String!
  configJson: String!
  enabled: Boolean!
  createdAt: String!
  updatedAt: String!
}

type NotificationTemplate {
  id: String!
  name: String!
  channelType: String!
  subjectTemplate: String
  bodyTemplate: String!
  createdAt: String!
  updatedAt: String!
}

# ── Workflow ──
type WorkflowDefinition {
  id: String!
  name: String!
  description: String!
  version: Int!
  enabled: Boolean!
  steps: [WorkflowStep!]!
  createdAt: String!
  updatedAt: String!
}

type WorkflowStep {
  stepId: String!
  name: String!
  stepType: String!
  assigneeRole: String
  timeoutHours: Int
  onApprove: String
  onReject: String
}

type WorkflowInstance {
  id: String!
  workflowId: String!
  workflowName: String!
  title: String!
  initiatorId: String!
  currentStepId: String
  status: String!
  contextJson: String
  startedAt: String!
  completedAt: String
  createdAt: String
}

type WorkflowTask {
  id: String!
  instanceId: String!
  stepId: String!
  stepName: String!
  assigneeId: String
  status: String!
  dueAt: String
  comment: String
  actorId: String
  decidedAt: String
  createdAt: String!
  updatedAt: String!
}
```

### エラーコード（GraphQL extensions.code）

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `NOT_FOUND` | 200（GraphQL エラー） | 要求したリソースが見つからない |
| `UNAUTHORIZED` | 401 | JWT 認証エラー |
| `FORBIDDEN` | 403 | 権限不足 |
| `VALIDATION_ERROR` | 200（GraphQL エラー） | 入力バリデーションエラー |
| `BACKEND_ERROR` | 200（GraphQL エラー） | バックエンド gRPC 呼び出しエラー |
| `TIMEOUT` | 200（GraphQL エラー） | クエリ実行タイムアウト |
| `INTERNAL_ERROR` | 500 | 内部エラー |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/model | GraphQL 出力型（`Tenant`, `FeatureFlag`, `ConfigEntry`, `User`, `Session`, `SecretMetadata`, `Job`, `NotificationLog`, `WorkflowDefinition` 等） | GraphQL スキーマ型定義 |
| domain/port | ポートトレイト（`TenantPort`, `FeatureFlagPort`, `ConfigPort`） | 依存性逆転（DIP）によるインフラ層抽象化 |
| domain/loader | DataLoader 実装（`TenantLoader`, `FeatureFlagLoader`, `ConfigLoader`） | バッチ取得（ポートトレイト経由） |
| usecase | `TenantQueryResolver`, `FeatureFlagQueryResolver`, `ConfigQueryResolver`, `TenantMutationResolver`, `AuthQueryResolver`, `SessionQueryResolver`, `VaultQueryResolver`, `SchedulerQueryResolver`, `NotificationQueryResolver`, `WorkflowQueryResolver`, `SubscriptionResolver` | クエリ・ミューテーション・サブスクリプション解決 |
| adapter/graphql | async-graphql の Query / Mutation / Subscription 実装 | GraphQL レイヤー |
| adapter/middleware | JWT 検証ミドルウェア（axum layer） | 認証処理 |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/grpc | `TenantGrpcClient`, `FeatureFlagGrpcClient`, `ConfigGrpcClient`, `AuthGrpcClient`, `SessionGrpcClient`, `VaultGrpcClient`, `SchedulerGrpcClient`, `NotificationGrpcClient`, `WorkflowGrpcClient`（各 Port トレイトを実装） | tonic gRPC クライアント |
| infra/auth | JWT 検証実装 | JWKS 取得・署名検証 |

> **監査 C-05 対応（2026-03-22）**: `graphql_context.rs` が `use crate::infrastructure::grpc::{...}` で infrastructure 層に直接依存していた問題を修正。`domain/port.rs` にポートトレイトを定義し、各 gRPC クライアントがこれを実装する形で依存性を逆転させた（クリーンアーキテクチャ DIP 準拠）。

### ドメインモデル

#### GraphQL Context

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `user_id` | String | JWT から取得したユーザー ID |
| `roles` | Vec\<String\> | JWT から取得したロールリスト |
| `request_id` | String | リクエスト追跡 ID |
| `tenant_loader` | DataLoader | テナントバッチローダー |
| `flag_loader` | DataLoader | フィーチャーフラグバッチローダー |
| `config_loader` | DataLoader | 設定バッチローダー |

`graphql_handler` は認証ミドルウェアで検証済みの Claims を受け取り、`GraphqlContext` と Claims の両方を async-graphql の request data に注入する。

#### DataLoader 設計

| DataLoader | バッチキー | 呼び出し先 gRPC |
| --- | --- | --- |
| `TenantLoader` | テナント ID リスト | TenantService.BatchGetTenants |
| `FeatureFlagLoader` | フラグキーリスト | FeatureFlagService.ListFlags |
| `ConfigLoader` | 設定キーリスト | ConfigService.GetConfig（逐次取得） |

> **注記**: ConfigLoader は現在 `GetConfig` を逐次呼び出しで実装している。将来 `BatchGetConfigs` RPC が ConfigService に実装された時点でバッチ化予定。

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ GraphQL Handler (graphql_handler.rs)     │   │
                    │  │  POST /graphql (Query / Mutation)        │   │
                    │  │  GET /graphql (Playground)               │   │
                    │  │  GET /graphql/ws (Subscription)          │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ JWT Middleware (auth_middleware.rs)       │   │
                    │  │  Authorization ヘッダー検証              │   │
                    │  │  JWKS 署名検証                           │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  TenantQueryResolver / FeatureFlagQueryResolver │
                    │  ConfigQueryResolver / TenantMutationResolver   │
                    │  SubscriptionResolver                           │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/model   │              │ domain/loader              │   │
    │  Tenant,        │              │ TenantLoader               │   │
    │  FeatureFlag,   │              │ FeatureFlagLoader          │   │
    │  ConfigEntry,   │              │ ConfigLoader               │   │
    │  GraphqlContext │              │ (DataLoader trait)         │   │
    └────────────────┘              └──────────┬─────────────────┘   │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ JWT 検証     │  │ TenantGrpcClient       │  │
                    │  │ (JWKS)       │  ├────────────────────────┤  │
                    │  └──────────────┘  │ FeatureFlagGrpcClient  │  │
                    │  ┌──────────────┐  ├────────────────────────┤  │
                    │  │ Config       │  │ ConfigGrpcClient       │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## セキュリティ・耐障害性

### サーキットブレーカー

`infrastructure/circuit_breaker.rs` に実装された `CircuitBreakerRegistry` により、各バックエンド gRPC サービスへの呼び出しをサーキットブレーカーで保護する。外部サービスの障害が他のサービスへ連鎖的に伝播するのを防止する。

| パラメータ | デフォルト値 | 説明 |
| --- | --- | --- |
| `failure_threshold` | 5 | オープン状態に遷移するまでの連続失敗回数 |
| `success_threshold` | 3 | ハーフオープン状態からクローズドに復帰するまでの連続成功回数 |
| `timeout_secs` | 30 | オープン状態からハーフオープンに遷移するまでの待機時間（秒） |

#### 動作フロー

1. **クローズド状態**: 通常どおりリクエストをバックエンドに転送する
2. **オープン状態**: `failure_threshold` 回の連続失敗後に遷移。即座に `CircuitBreakerError::Open` を返し、バックエンドへのリクエストを遮断する
3. **ハーフオープン状態**: `timeout_secs` 経過後に遷移。試行リクエストを許可し、`success_threshold` 回の連続成功でクローズドに復帰する

レジストリはサービス名をキーとして各バックエンドに個別のサーキットブレーカーを保持する。未登録のサービスへの初回アクセス時にデフォルト設定で自動作成される（ダブルチェックロッキングパターン）。`k1s0_circuit_breaker` ライブラリを使用する。

### リクエストボディサイズ制限

`tower_http::limit::RequestBodyLimitLayer` を使用して、リクエストボディサイズを **2 MB** に制限する。過大なペイロードによるメモリ枯渇攻撃を防止する。

```rust
// startup.rs での設定
.layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
```

併せて `ConcurrencyLimitLayer` で同時リクエスト数を **100 並列** に制限し、サーバーリソースの過剰消費を防止する。

### JWT issuer/audience 検証

`config.yaml` の `auth.issuer` / `auth.audience` を設定することで、JWT の `iss` / `aud` クレームを厳密に検証する。未設定時は検証をスキップする（後方互換性のため）。

本番環境では両フィールドを必ず設定し、不正な JWT の受け入れを防止すること。

### subscription RBAC

GraphQL Subscription（`configChanged`・`tenantUpdated`・`featureFlagChanged`）は、Query と同等の read 権限チェック（`sys_admin` / `sys_operator` / `sys_auditor` ロール）を要求する。未認証またはロール不足の場合は `FORBIDDEN` エラーを返す。

### レート制限キー改善

リクエストのレート制限キーは以下の優先順位で決定する:

1. **認証済みユーザー**: JWT Claims の `sub`（ユーザーID）をキーとして使用（IP スプーフィング耐性あり）
2. **X-Forwarded-For ヘッダー**: プロキシ経由のクライアント IP
3. **ConnectInfo**: 直接接続のソケット IP（`into_make_service_with_connect_info` が必要）
4. **anonymous**: 識別子が取得できない場合（`warn!` ログを出力する）

---

## 設定フィールド

### graphql

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `introspection` | bool | `false` | スキーマイントロスペクション有効化（development のみ推奨） |
| `playground` | bool | `false` | GraphQL Playground 有効化（development のみ推奨） |
| `max_depth` | int | `10` | クエリネスト深度の上限 |
| `max_complexity` | int | `1000` | クエリ複雑度の上限 |
| `query_timeout_seconds` | int | `30` | クエリ実行タイムアウト（秒） |

### auth

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `jwks_url` | string | JWKS エンドポイント URL |
| `issuer` | string（省略可） | JWT issuer 検証。設定時は JWT の `iss` クレームと一致しない場合は 401 を返す |
| `audience` | string（省略可） | JWT audience 検証。設定時は JWT の `aud` クレームと一致しない場合は 401 を返す |

### backends

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `tenant.address` | string | テナントサービス gRPC エンドポイント |
| `tenant.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `featureflag.address` | string | フィーチャーフラグサービス gRPC エンドポイント |
| `featureflag.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `config.address` | string | 設定サービス gRPC エンドポイント |
| `config.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `navigation.address` | string | ナビゲーションサービス gRPC エンドポイント |
| `navigation.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `service_catalog.address` | string | サービスカタログサービス gRPC エンドポイント |
| `service_catalog.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `auth.address` | string | 認証サービス gRPC エンドポイント |
| `auth.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `session.address` | string | セッションサービス gRPC エンドポイント |
| `session.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `vault.address` | string | Vault サービス gRPC エンドポイント |
| `vault.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `scheduler.address` | string | スケジューラーサービス gRPC エンドポイント |
| `scheduler.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `notification.address` | string | 通知サービス gRPC エンドポイント |
| `notification.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `workflow.address` | string | ワークフローサービス gRPC エンドポイント |
| `workflow.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |

### observability

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `log.level` | string | ログレベル（`debug` / `info` / `warn` / `error`） |
| `log.format` | string | ログフォーマット（`text` / `json`） |
| `trace.enabled` | bool | トレース出力の有効/無効 |
| `trace.endpoint` | string | OTLP エンドポイント |
| `trace.sample_rate` | float | サンプリングレート |
| `metrics.enabled` | bool | メトリクス出力の有効/無効 |
| `metrics.path` | string | メトリクスエンドポイントパス |

---

## 設定ファイル例

### config.yaml（本番）
> ※ dev環境では省略可能なセクションがあります。


```yaml
app:
  name: "graphql-gateway"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

graphql:
  introspection: false
  playground: false
  max_depth: 10
  max_complexity: 1000
  query_timeout_seconds: 30

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local/jwks"

backends:
  tenant:
    address: "http://tenant-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  featureflag:
    address: "http://featureflag-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  config:
    address: "http://config-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  navigation:
    address: "http://navigation-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  service_catalog:
    address: "http://service-catalog-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  auth:
    address: "http://auth-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  session:
    address: "http://session-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  vault:
    address: "http://vault-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  scheduler:
    address: "http://scheduler-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  notification:
    address: "http://notification-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  workflow:
    address: "http://workflow-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "http://otel-collector.observability.svc.cluster.local:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
```

### Helm values

```yaml
# values-graphql-gateway.yaml（infra/helm/services/system/graphql-gateway/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/graphql-gateway
  tag: ""

replicaCount: 2

container:
  port: 8080

service:
  type: ClusterIP
  port: 80

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

vault:
  enabled: false
```

---

## デプロイ

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST / GraphQL (HTTP) | 8080 | GraphQL API + Playground |

---

## 詳細設計ドキュメント

- [system-graphql-gateway-implementation.md](implementation.md) -- 実装設計の詳細
- [system-graphql-gateway-deploy.md](deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [GraphQL設計.md](../../architecture/api/GraphQL設計.md) -- GraphQL 設計ガイドライン
- [テンプレート仕様-BFF.md](../../templates/client/BFF.md) -- BFF テンプレート仕様
- [system-server.md](../auth/server.md) -- system tier サーバー一覧

## Doc Sync (2026-03-03)

### Message/Field Corrections
- GraphqlContext は tenant_loader, flag_loader に加えて config_loader を保持する。
---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
