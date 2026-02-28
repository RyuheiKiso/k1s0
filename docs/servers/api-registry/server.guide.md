# system-api-registry-server 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| スキーマ種別 | `openapi`（OpenAPI 3.x YAML/JSON）と `protobuf`（.proto ファイル）の 2 種類をサポート |
| バリデーション | 登録時に subprocess 経由で openapi-spec-validator（OpenAPI）または buf lint（Protobuf）を実行し検証エラーを返す |
| 破壊的変更検出 | 新バージョン登録時に前バージョンとの比較を行い、フィールド削除・型変更・必須化・パス削除等の変更を検出する |
| 差分表示 | バージョン間の差分を `added` / `modified` / `removed` に分類した構造化 JSON で提供する |
| kafka-schemaregistry との対比 | kafka-schemaregistry ライブラリは Kafka Avro スキーマ向け。当サーバーは REST/gRPC スキーマのレジストリとして機能する |
| DB | PostgreSQL の `apiregistry` スキーマ（api_schemas, api_schema_versions テーブル） |
| Kafka | プロデューサー（`k1s0.system.apiregistry.schema_updated.v1`） |
| 認証 | JWTによる認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |
| ポート | 8101（REST）/ 9090（gRPC） |

---

## API リクエスト・レスポンス例

### GET /api/v1/schemas

```json
{
  "schemas": [
    {
      "name": "k1s0-tenant-api",
      "description": "テナント管理 API スキーマ",
      "schema_type": "openapi",
      "latest_version": 3,
      "version_count": 3,
      "created_at": "2026-02-10T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "name": "k1s0-notification-proto",
      "description": "通知サービス Protobuf スキーマ",
      "schema_type": "protobuf",
      "latest_version": 1,
      "version_count": 1,
      "created_at": "2026-02-15T10:00:00.000+00:00",
      "updated_at": "2026-02-15T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 12,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### POST /api/v1/schemas

**リクエスト（OpenAPI）**

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 1.0.0\npaths:\n  /api/v1/tenants:\n    get:\n      summary: テナント一覧\n      responses:\n        '200':\n          description: OK\n"
}
```

**リクエスト（Protobuf）**

```json
{
  "name": "k1s0-notification-proto",
  "description": "通知サービス Protobuf スキーマ",
  "schema_type": "protobuf",
  "content": "syntax = \"proto3\";\npackage k1s0.system.notification.v1;\n\nservice NotificationService {\n  rpc SendNotification(SendNotificationRequest) returns (SendNotificationResponse);\n}\n\nmessage SendNotificationRequest {\n  string channel_id = 1;\n  string recipient = 2;\n}\n\nmessage SendNotificationResponse {\n  string notification_id = 1;\n  string status = 2;\n}\n"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "version": 1,
  "content_hash": "sha256:a1b2c3d4e5f6...",
  "created_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（422 Unprocessable Entity）**

```json
{
  "error": {
    "code": "SYS_APIREG_SCHEMA_INVALID",
    "message": "schema validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "content", "message": "[line 5] info.version is required"},
      {"field": "content", "message": "[line 12] response schema must have a type"}
    ]
  }
}
```

### GET /api/v1/schemas/:name

```json
{
  "name": "k1s0-tenant-api",
  "description": "テナント管理 API スキーマ",
  "schema_type": "openapi",
  "latest_version": 3,
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 3.0.0\n...",
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "created_at": "2026-02-10T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

### GET /api/v1/schemas/:name/versions

```json
{
  "name": "k1s0-tenant-api",
  "versions": [
    {
      "version": 3,
      "content_hash": "sha256:f6e5d4c3b2a1...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-20T12:30:00.000+00:00"
    },
    {
      "version": 2,
      "content_hash": "sha256:e5d4c3b2a1f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-15T10:00:00.000+00:00"
    },
    {
      "version": 1,
      "content_hash": "sha256:a1b2c3d4e5f6...",
      "breaking_changes": false,
      "registered_by": "user-001",
      "created_at": "2026-02-10T10:00:00.000+00:00"
    }
  ],
  "pagination": {
    "total_count": 3,
    "page": 1,
    "page_size": 20,
    "has_next": false
  }
}
```

### GET /api/v1/schemas/:name/versions/:version

```json
{
  "name": "k1s0-tenant-api",
  "version": 2,
  "schema_type": "openapi",
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 2.0.0\n...",
  "content_hash": "sha256:e5d4c3b2a1f6...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "created_at": "2026-02-15T10:00:00.000+00:00"
}
```

### POST /api/v1/schemas/:name/versions

**リクエスト**

```json
{
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 3.0.0\npaths:\n  /api/v1/tenants:\n    get:\n      summary: テナント一覧\n      ...\n",
  "registered_by": "user-001"
}
```

**レスポンス（201 Created）**

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "breaking_change_details": [],
  "registered_by": "user-001",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（201 Created -- 破壊的変更あり）**

```json
{
  "name": "k1s0-tenant-api",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": true,
  "breaking_change_details": [
    {
      "change_type": "field_removed",
      "path": "/api/v1/tenants GET response.properties.legacy_id",
      "description": "フィールド 'legacy_id' が削除されました"
    },
    {
      "change_type": "type_changed",
      "path": "/api/v1/tenants/{id} GET response.properties.created_at",
      "description": "'created_at' の型が string から integer に変更されました"
    }
  ],
  "registered_by": "user-001",
  "created_at": "2026-02-20T12:30:00.000+00:00"
}
```

### DELETE /api/v1/schemas/:name/versions/:version

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_APIREG_CANNOT_DELETE_LATEST",
    "message": "cannot delete the only remaining version of schema: k1s0-tenant-api",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/schemas/:name/compatibility

**リクエスト**

```json
{
  "content": "openapi: 3.0.3\ninfo:\n  title: Tenant API\n  version: 4.0.0\n...",
  "base_version": 3
}
```

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "base_version": 3,
  "compatible": false,
  "breaking_changes": [
    {
      "change_type": "field_removed",
      "path": "/api/v1/tenants GET response.properties.name",
      "description": "フィールド 'name' が削除されました"
    }
  ],
  "non_breaking_changes": [
    {
      "change_type": "field_added",
      "path": "/api/v1/tenants GET response.properties.display_name",
      "description": "フィールド 'display_name' が追加されました"
    }
  ]
}
```

### GET /api/v1/schemas/:name/diff

**レスポンス（200 OK）**

```json
{
  "name": "k1s0-tenant-api",
  "from_version": 2,
  "to_version": 3,
  "breaking_changes": false,
  "diff": {
    "added": [
      {
        "path": "/api/v1/tenants GET response.properties.display_name",
        "type": "object",
        "description": "新フィールド: display_name（表示名）"
      }
    ],
    "modified": [
      {
        "path": "/api/v1/tenants GET summary",
        "before": "テナント一覧",
        "after": "テナント一覧取得"
      }
    ],
    "removed": []
  }
}
```

---

## Kafka メッセージフォーマット

### 新バージョン登録

```json
{
  "event_type": "SCHEMA_VERSION_REGISTERED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 3,
  "content_hash": "sha256:f6e5d4c3b2a1...",
  "breaking_changes": false,
  "registered_by": "user-001",
  "timestamp": "2026-02-20T12:30:00.000+00:00"
}
```

### バージョン削除

```json
{
  "event_type": "SCHEMA_VERSION_DELETED",
  "schema_name": "k1s0-tenant-api",
  "schema_type": "openapi",
  "version": 1,
  "deleted_by": "user-001",
  "timestamp": "2026-02-20T15:00:00.000+00:00"
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (apiregistry_handler.rs)    │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_schemas / register_schema          │   │
                    │  │  get_schema / list_versions              │   │
                    │  │  get_schema_version                      │   │
                    │  │  register_version / delete_version       │   │
                    │  │  check_compatibility / get_diff          │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (apiregistry_grpc.rs)       │   │
                    │  │  GetSchema / GetSchemaVersion            │   │
                    │  │  CheckCompatibility                      │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  ListSchemasUsecase / RegisterSchemaUsecase /   │
                    │  GetSchemaUsecase / ListVersionsUsecase /       │
                    │  GetSchemaVersionUsecase /                      │
                    │  RegisterVersionUsecase / DeleteVersionUsecase /│
                    │  CheckCompatibilityUsecase / GetDiffUsecase     │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  ApiSchema,     │              │ ApiSchemaRepository        │   │
    │  ApiSchemaVer,  │              │ ApiSchemaVersionRepository │   │
    │  Compatibility  │              │ (trait)                    │   │
    │  Result,        │              └──────────┬─────────────────┘   │
    │  SchemaDiff     │                         │                     │
    └────────────────┘                         │                     │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ ApiRegistry    │            │                     │
                 │ DomainService  │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ ApiSchemaPostgres       │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ ApiSchemaVersion       │  │
                    │  │ OpenApi      │  │ PostgresRepository     │  │
                    │  │ Validator    │  └────────────────────────┘  │
                    │  ├──────────────┤  ┌────────────────────────┐  │
                    │  │ Protobuf     │  │ Database               │  │
                    │  │ Validator    │  │ Config                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐                              │
                    │  │ Config       │                              │
                    │  │ Loader       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "api-registry"
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
  topic: "k1s0.system.apiregistry.schema_updated.v1"

validator:
  openapi_spec_validator_path: "/usr/local/bin/openapi-spec-validator"
  buf_path: "/usr/local/bin/buf"
  timeout_seconds: 30
```

---

## デプロイ

### Helm values

[helm設計.md](../../infrastructure/kubernetes/helm設計.md) のサーバー用 Helm Chart を使用する。api-registry 固有の values は以下の通り。

```yaml
# values-api-registry.yaml（infra/helm/services/system/api-registry/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/api-registry
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
  maxReplicas: 4
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/api-registry/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/api-registry/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |
