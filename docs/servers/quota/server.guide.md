# system-quota-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/quotas

```json
{
  "quotas": [
    {
      "id": "quota_01JABCDEF1234567890",
      "name": "スタンダードプランAPIクォータ",
      "subject_type": "tenant",
      "subject_id": "tenant-abc",
      "limit": 10000,
      "period": "daily",
      "enabled": true,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 15,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### POST /api/v1/quotas

**リクエスト**

```json
{
  "name": "スタンダードプランAPIクォータ",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "limit": 10000,
  "period": "daily",
  "enabled": true,
  "alert_threshold_percent": 80
}
```

**レスポンス（201 Created）**

```json
{
  "id": "quota_01JABCDEF1234567890",
  "name": "スタンダードプランAPIクォータ",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "limit": 10000,
  "period": "daily",
  "enabled": true,
  "alert_threshold_percent": 80,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_QUOTA_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "subject_type", "message": "must be one of: tenant, user, api_key"},
      {"field": "limit", "message": "limit must be greater than 0"}
    ]
  }
}
```

### GET /api/v1/quotas/:id/usage

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 7523,
  "remaining": 2477,
  "usage_percent": 75.23,
  "exceeded": false,
  "period_start": "2026-02-23T00:00:00.000+00:00",
  "period_end": "2026-02-23T23:59:59.999+00:00",
  "reset_at": "2026-02-24T00:00:00.000+00:00",
  "retrieved_at": "2026-02-23T14:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_QUOTA_NOT_FOUND",
    "message": "quota policy not found: quota_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/quotas/:id/usage/increment

**リクエスト**

```json
{
  "amount": 1,
  "request_id": "req_abc123def456"
}
```

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "used": 7524,
  "remaining": 2476,
  "usage_percent": 75.24,
  "exceeded": false,
  "allowed": true
}
```

**レスポンス（429 Too Many Requests）**

```json
{
  "error": {
    "code": "SYS_QUOTA_EXCEEDED",
    "message": "quota exceeded for tenant-abc: 10000/10000 (daily)",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "quota_id", "message": "quota_01JABCDEF1234567890"},
      {"field": "reset_at", "message": "2026-02-24T00:00:00.000+00:00"}
    ]
  }
}
```

### POST /api/v1/quotas/:id/usage/reset

**リクエスト**

```json
{
  "reason": "プラン変更に伴うリセット"
}
```

**レスポンス（200 OK）**

```json
{
  "quota_id": "quota_01JABCDEF1234567890",
  "used": 0,
  "reset_at": "2026-02-23T15:00:00.000+00:00",
  "reset_by": "admin@example.com"
}
```

---

## Kafka メッセージ例

### クォータ超過イベント

```json
{
  "event_type": "QUOTA_EXCEEDED",
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 10001,
  "exceeded_at": "2026-02-23T14:30:00.000+00:00",
  "reset_at": "2026-02-24T00:00:00.000+00:00"
}
```

### アラート閾値通知イベント

```json
{
  "event_type": "QUOTA_THRESHOLD_REACHED",
  "quota_id": "quota_01JABCDEF1234567890",
  "subject_type": "tenant",
  "subject_id": "tenant-abc",
  "period": "daily",
  "limit": 10000,
  "used": 8002,
  "usage_percent": 80.02,
  "alert_threshold_percent": 80,
  "reached_at": "2026-02-23T12:00:00.000+00:00"
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (quota_handler.rs)          │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_quotas / create_quota /            │   │
                    │  │  get_quota / update_quota / delete_quota │   │
                    │  │  get_usage / increment_usage / reset     │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (quota_grpc.rs)             │   │
                    │  │  CheckAndIncrement / GetUsage            │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateQuotaPolicyUsecase /                     │
                    │  UpdateQuotaPolicyUsecase /                     │
                    │  DeleteQuotaPolicyUsecase /                     │
                    │  GetQuotaPolicyUsecase /                        │
                    │  ListQuotaPoliciesUsecase /                     │
                    │  GetQuotaUsageUsecase /                         │
                    │  IncrementQuotaUsageUsecase /                   │
                    │  ResetQuotaUsageUsecase                         │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  QuotaPolicy,   │              │ QuotaPolicyRepository      │   │
    │  QuotaUsage     │              │ QuotaUsageRepository       │   │
    └────────────────┘              │ QuotaCounterRepository     │   │
              │                     │ (trait)                    │   │
              │  ┌────────────────┐  └──────────┬─────────────────┘   │
              └──▶ domain/service │             │                     │
                 │ QuotaDomain   │             │                     │
                 │ Service       │             │                     │
                 └────────────────┘             │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ QuotaPolicy/Usage       │  │
                    │  │ Producer     │  │ PostgresRepository      │  │
                    │  │ (exceeded)   │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ QuotaRedisCounter      │  │
                    │  │ Config       │  │ Repository (deadpool)   │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "quota"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "quota"
  max_connections: 10
  min_connections: 2
  connect_timeout_seconds: 5

redis:
  url: "redis://redis.k1s0-system.svc.cluster.local:6379"
  pool_size: 10
  key_prefix: "quota:"
  connect_timeout_seconds: 3

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_exceeded: "k1s0.system.quota.exceeded.v1"
  topic_threshold: "k1s0.system.quota.threshold.reached.v1"

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"

quota:
  reset_schedule:
    daily: "0 0 * * *"
    monthly: "0 0 1 * *"
```

### Helm values

```yaml
# values-quota.yaml（infra/helm/services/system/quota/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/quota
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
    - path: "secret/data/k1s0/system/quota/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
    - path: "secret/data/k1s0/system/quota/redis"
      key: "password"
      mountPath: "/vault/secrets/redis-password"
```
