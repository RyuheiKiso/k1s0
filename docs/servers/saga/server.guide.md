# system-saga-server 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### POST /api/v1/sagas

**リクエスト例**

```json
{
  "workflow_name": "order-fulfillment",
  "payload": {
    "order_id": "ord-12345",
    "customer_id": "cust-67890",
    "items": [
      {"product_id": "prod-001", "quantity": 2}
    ],
    "total_amount": 5000
  },
  "correlation_id": "req-abc-123",
  "initiated_by": "order-service"
}
```

**レスポンス（201 Created）**

```json
{
  "saga_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "STARTED"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SAGA_VALIDATION_ERROR",
    "message": "workflow_name is required",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/sagas

**レスポンス（200 OK）**

```json
{
  "sagas": [
    {
      "saga_id": "550e8400-e29b-41d4-a716-446655440000",
      "workflow_name": "order-fulfillment",
      "current_step": 3,
      "status": "COMPLETED",
      "payload": {"order_id": "ord-12345"},
      "correlation_id": "req-abc-123",
      "initiated_by": "order-service",
      "error_message": null,
      "created_at": "2026-02-20T10:30:00.000Z",
      "updated_at": "2026-02-20T10:30:05.123Z"
    }
  ],
  "pagination": {
    "total_count": 150,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

### GET /api/v1/sagas/:saga_id

**レスポンス（200 OK）**

```json
{
  "saga": {
    "saga_id": "550e8400-e29b-41d4-a716-446655440000",
    "workflow_name": "order-fulfillment",
    "current_step": 3,
    "status": "COMPLETED",
    "payload": {"order_id": "ord-12345"},
    "correlation_id": "req-abc-123",
    "initiated_by": "order-service",
    "error_message": null,
    "created_at": "2026-02-20T10:30:00.000Z",
    "updated_at": "2026-02-20T10:30:05.123Z"
  },
  "step_logs": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "step_index": 0,
      "step_name": "reserve-inventory",
      "action": "EXECUTE",
      "status": "SUCCESS",
      "request_payload": {"order_id": "ord-12345"},
      "response_payload": {"reservation_id": "res-001"},
      "error_message": null,
      "started_at": "2026-02-20T10:30:00.100Z",
      "completed_at": "2026-02-20T10:30:01.200Z"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440002",
      "step_index": 1,
      "step_name": "process-payment",
      "action": "EXECUTE",
      "status": "SUCCESS",
      "request_payload": {"order_id": "ord-12345"},
      "response_payload": {"transaction_id": "txn-001"},
      "error_message": null,
      "started_at": "2026-02-20T10:30:01.300Z",
      "completed_at": "2026-02-20T10:30:03.500Z"
    }
  ]
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SAGA_NOT_FOUND",
    "message": "saga not found: invalid-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/sagas/:saga_id/cancel

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "saga 550e8400-e29b-41d4-a716-446655440000 cancelled"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SAGA_CONFLICT",
    "message": "saga is already in terminal state",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/workflows

**リクエスト例**

```json
{
  "workflow_yaml": "name: order-fulfillment\nsteps:\n  - name: reserve-inventory\n    service: inventory-service\n    method: InventoryService.Reserve\n    compensate: InventoryService.Release\n    timeout_secs: 30\n    retry:\n      max_attempts: 3\n      backoff: exponential\n      initial_interval_ms: 1000\n"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "order-fulfillment",
  "step_count": 3
}
```

### GET /api/v1/workflows

**レスポンス（200 OK）**

```json
{
  "workflows": [
    {
      "name": "order-fulfillment",
      "step_count": 3,
      "step_names": ["reserve-inventory", "process-payment", "arrange-shipping"]
    }
  ]
}
```

---

## 実行フロー

### 正常系（全ステップ成功）

```
1. StartSaga API 呼び出し
2. SagaState 作成 (status=STARTED)
3. tokio::spawn で非同期実行開始
4. status → RUNNING
5. Step 0: gRPC 呼び出し → 成功 → step_log 記録
6. Step 1: gRPC 呼び出し → 成功 → step_log 記録
7. Step N: gRPC 呼び出し → 成功 → step_log 記録
8. status → COMPLETED
9. Kafka イベント発行: SAGA_COMPLETED
```

### 異常系（ステップ失敗 → 補償）

```
1. StartSaga API 呼び出し
2. SagaState 作成 (status=STARTED)
3. tokio::spawn で非同期実行開始
4. status → RUNNING
5. Step 0: gRPC 呼び出し → 成功
6. Step 1: gRPC 呼び出し → 成功
7. Step 2: gRPC 呼び出し → 失敗（リトライ上限到達）
8. status → COMPENSATING
9. Kafka イベント発行: SAGA_COMPENSATING
10. Compensate Step 1: compensate メソッド呼び出し → 成功/失敗（best-effort）
11. Compensate Step 0: compensate メソッド呼び出し → 成功/失敗（best-effort）
12. status → FAILED
13. Kafka イベント発行: SAGA_FAILED
```

---

## ワークフロー YAML 定義例

```yaml
name: order-fulfillment
steps:
  - name: reserve-inventory
    service: inventory-service
    method: InventoryService.Reserve
    compensate: InventoryService.Release
    timeout_secs: 30
    retry:
      max_attempts: 3
      backoff: exponential
      initial_interval_ms: 1000

  - name: process-payment
    service: payment-service
    method: PaymentService.Charge
    compensate: PaymentService.Refund
    timeout_secs: 60
    retry:
      max_attempts: 2
      backoff: exponential
      initial_interval_ms: 2000

  - name: arrange-shipping
    service: shipping-service
    method: ShippingService.CreateShipment
    compensate: ShippingService.CancelShipment
    timeout_secs: 30
```

---

## トランザクション設計

`update_with_step_log` メソッドでは、Saga 状態の更新とステップログの挿入を単一のデータベーストランザクションで実行する。

```
BEGIN;
  UPDATE saga.saga_states SET current_step=$2, status=$3, ... WHERE id=$1;
  INSERT INTO saga.saga_step_logs (id, saga_id, step_index, ...) VALUES (...);
COMMIT;
```

---

## Cargo.toml

```toml
[package]
name = "k1s0-saga-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# 共通
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
tracing = "0.1"

# 内部ライブラリ
k1s0-auth = { path = "../../library/rust/auth" }
k1s0-telemetry = { path = "../../library/rust/telemetry" }

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
```

---

## サービスレジストリと gRPC 呼び出し

`config.yaml` の `services` セクションでサービスのエンドポイントを静的に定義し、`ServiceRegistry` が名前解決を行う。

```yaml
services:
  inventory-service:
    host: "inventory.k1s0-business.svc.cluster.local"
    port: 50051
  payment-service:
    host: "payment.k1s0-business.svc.cluster.local"
    port: 50051
```

`TonicGrpcCaller` は `ServiceRegistry` から取得したエンドポイントに対して tonic の gRPC チャネルを作成し、ワークフローステップの `method` フィールド（`ServiceName.MethodName` 形式）を gRPC パスに変換して動的に呼び出す。チャネルは `RwLock<HashMap<String, Channel>>` で接続プールとして管理する。

---

## Bootstrap（main.rs）

起動シーケンスは auth-server パターンに従う:

```
1. k1s0-telemetry 初期化
2. config.yaml ロード
3. PostgreSQL 接続プール作成（オプショナル）
4. SagaRepository 構築（Postgres or InMemory）
5. InMemoryWorkflowRepository 構築 + workflows/ ディレクトリからロード
6. ServiceRegistry + TonicGrpcCaller 構築
7. KafkaProducer 構築（オプショナル）
8. ユースケース群を構築（Arc でラップ）
9. RecoverSagasUseCase 実行（起動時リカバリ）
10. AppState 構築
11. REST サーバー（axum）+ gRPC サーバー（tonic）を tokio::select! で並行起動
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "saga-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

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
  consumer_group: "saga-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.system.saga.events.v1"
    subscribe: []

services:
  inventory-service:
    host: "inventory.k1s0-business.svc.cluster.local"
    port: 50051
  payment-service:
    host: "payment.k1s0-business.svc.cluster.local"
    port: 50051
  shipping-service:
    host: "shipping.k1s0-business.svc.cluster.local"
    port: 50051

saga:
  max_concurrent: 100
  workflow_dir: "workflows"
```

---

## Helm values 例

```yaml
# values-saga.yaml（infra/helm/services/system/saga/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/saga
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

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
    - path: "secret/data/k1s0/system/saga/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```
