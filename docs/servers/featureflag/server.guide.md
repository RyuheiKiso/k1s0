# system-featureflag-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/flags

```json
{
  "flags": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "flag_key": "enable-new-checkout",
      "description": "新しいチェックアウトフローを有効化する",
      "enabled": true,
      "variants": [
        { "name": "on", "value": "true", "weight": 80 },
        { "name": "off", "value": "false", "weight": 20 }
      ],
      "created_at": "2026-02-20T10:00:00+00:00",
      "updated_at": "2026-02-20T12:30:00+00:00"
    }
  ]
}
```

### GET /api/v1/flags/:key

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": true,
  "variants": [
    { "name": "on", "value": "true", "weight": 80 },
    { "name": "off", "value": "false", "weight": 20 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T12:30:00+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "feature flag not found: enable-new-checkout",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/flags

**リクエスト**

```json
{
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ]
}
```

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する",
  "enabled": false,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T10:00:00+00:00"
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_FF_ALREADY_EXISTS",
    "message": "flag already exists: enable-new-checkout"
  }
}
```

### PUT /api/v1/flags/:key

**リクエスト**

```json
{
  "enabled": true,
  "description": "新しいチェックアウトフローを有効化する（v2）"
}
```

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "flag_key": "enable-new-checkout",
  "description": "新しいチェックアウトフローを有効化する（v2）",
  "enabled": true,
  "variants": [
    { "name": "on", "value": "true", "weight": 100 }
  ],
  "created_at": "2026-02-20T10:00:00+00:00",
  "updated_at": "2026-02-20T12:30:00+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

### DELETE /api/v1/flags/:key

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "flag enable-new-checkout deleted"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

### POST /api/v1/flags/:key/evaluate

**リクエスト**

```json
{
  "user_id": "user-001",
  "tenant_id": "tenant-abc",
  "attributes": {
    "environment": "production",
    "region": "ap-northeast-1"
  }
}
```

**レスポンス（200 OK -- フラグ有効）**

```json
{
  "flag_key": "enable-new-checkout",
  "enabled": true,
  "variant": "on",
  "reason": "flag is enabled"
}
```

**レスポンス（200 OK -- フラグ無効）**

```json
{
  "flag_key": "enable-new-checkout",
  "enabled": false,
  "variant": null,
  "reason": "flag is disabled"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_FF_NOT_FOUND",
    "message": "flag not found: enable-new-checkout"
  }
}
```

---

## フラグ評価ロジック

### 評価フロー

フラグ評価は以下の順序で判定される。

```
1. フラグが存在するか確認（未存在: FlagNotFound エラー）
2. フラグが enabled=false の場合
   → EvaluationResult { enabled: false, variant: None, reason: "flag is disabled" }
3. フラグが enabled=true の場合
   → variants の先頭バリアントを選択
   → EvaluationResult { enabled: true, variant: Some(variants[0].name), reason: "flag is enabled" }
```

現在の実装では、ルール（FlagRule）による属性マッチング評価は未実装。ドメインモデルに `rules: Vec<FlagRule>` フィールドは定義済みであり、将来的にルールベースの条件分岐評価を追加予定。

---

## Kafka メッセージ例

### フラグ変更通知

```json
{
  "event_type": "FLAG_UPDATED",
  "flag_key": "enable-new-checkout",
  "timestamp": "2026-02-20T12:30:00.000+00:00",
  "actor_user_id": "admin-001",
  "before": {
    "enabled": false,
    "variants": []
  },
  "after": {
    "enabled": true,
    "variants": [
      { "name": "on", "value": "true", "weight": 100 }
    ]
  }
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (flag_handler.rs)           │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_flags / get_flag / create_flag     │   │
                    │  │  update_flag / delete_flag               │   │
                    │  │  evaluate_flag                           │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (flag_grpc.rs)              │   │
                    │  │  EvaluateFlag / GetFlag / CreateFlag     │   │
                    │  │  UpdateFlag                              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  EvaluateFlagUsecase / GetFlagUsecase /         │
                    │  ListFlagsUsecase / CreateFlagUsecase /         │
                    │  UpdateFlagUsecase / DeleteFlagUsecase          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  FeatureFlag,   │              │ FeatureFlagRepository      │   │
    │  FlagEvaluation,│              │ FlagAuditLogRepository     │   │
    │  FlagAuditLog   │              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ FeatureFlagDomain           │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ FeatureFlagPostgres    │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ FlagAuditLogPostgres   │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ Service      │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Database               │  │
                    │  │ Config       │  │ Config                 │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "featureflag"
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
  topic: "k1s0.system.featureflag.changed.v1"

cache:
  max_entries: 10000
  ttl_seconds: 60
```

### Helm values

```yaml
# values-featureflag.yaml（infra/helm/services/system/featureflag/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/featureflag
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
    - path: "secret/data/k1s0/system/featureflag/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```
