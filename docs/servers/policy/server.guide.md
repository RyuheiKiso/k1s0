# system-policy-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### GET /api/v1/policies

```json
{
  "policies": [
    {
      "id": "policy-001",
      "name": "k1s0-tenant-access",
      "description": "テナントへのアクセス制御ポリシー",
      "package_path": "k1s0.system.tenant",
      "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
      "bundle_id": "bundle-001",
      "enabled": true,
      "version": 3,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
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

### GET /api/v1/policies/:id

**レスポンス（200 OK）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 3,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T12:30:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found: policy-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/policies

**リクエスト**

```json
{
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true
}
```

**レスポンス（201 Created）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 1,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_POLICY_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "rego_content", "message": "invalid Rego syntax: unexpected token at line 3"},
      {"field": "package_path", "message": "package_path is required and must be non-empty"}
    ]
  }
}
```

### PUT /api/v1/policies/:id

**リクエスト**

```json
{
  "description": "テナントへのアクセス制御ポリシー（v2 - operatorも許可）",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}\n\nallow {\n  input.role == \"sys_operator\"\n}",
  "enabled": true
}
```

**レスポンス（200 OK）**

```json
{
  "id": "policy-001",
  "name": "k1s0-tenant-access",
  "description": "テナントへのアクセス制御ポリシー（v2 - operatorも許可）",
  "package_path": "k1s0.system.tenant",
  "rego_content": "package k1s0.system.tenant\n\ndefault allow = false\n\nallow {\n  input.role == \"sys_admin\"\n}\n\nallow {\n  input.role == \"sys_operator\"\n}",
  "bundle_id": "bundle-001",
  "enabled": true,
  "version": 4,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T15:00:00.000+00:00"
}
```

### DELETE /api/v1/policies/:id

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "policy policy-001 deleted"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found: policy-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/policies/:id/evaluate

**リクエスト**

```json
{
  "package_path": "k1s0.system.tenant",
  "input": {
    "role": "sys_operator",
    "action": "read",
    "resource": "tenant",
    "tenant_id": "tenant-abc"
  }
}
```

**レスポンス（200 OK -- 許可）**

```json
{
  "allowed": true,
  "package_path": "k1s0.system.tenant",
  "decision_id": "dec_xyz789abc123",
  "cached": false
}
```

**レスポンス（200 OK -- 拒否）**

```json
{
  "allowed": false,
  "package_path": "k1s0.system.tenant",
  "decision_id": "dec_xyz789abc124",
  "cached": true
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_POLICY_NOT_FOUND",
    "message": "policy not found for package: k1s0.system.tenant",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/bundles

```json
{
  "bundles": [
    {
      "id": "bundle-001",
      "name": "k1s0-system-policies",
      "description": "system tier の標準アクセス制御ポリシー群",
      "policy_count": 5,
      "enabled": true,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T12:30:00.000+00:00"
    }
  ],
  "total_count": 1
}
```

---

## Kafka メッセージ例

### ポリシー変更通知

```json
{
  "event_type": "POLICY_UPDATED",
  "policy_id": "policy-001",
  "package_path": "k1s0.system.tenant",
  "operation": "UPDATE",
  "version": 4,
  "timestamp": "2026-02-20T15:00:00.000+00:00",
  "actor_user_id": "admin-001"
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (policy_handler.rs)         │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_policies / get_policy              │   │
                    │  │  create_policy / update_policy           │   │
                    │  │  delete_policy / evaluate                │   │
                    │  │  list_bundles                            │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (policy_grpc.rs)            │   │
                    │  │  EvaluatePolicy / GetPolicy              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  GetPolicyUsecase / ListPoliciesUsecase /       │
                    │  CreatePolicyUsecase / UpdatePolicyUsecase /    │
                    │  DeletePolicyUsecase / EvaluatePolicyUsecase /  │
                    │  ListBundlesUsecase                             │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  Policy,        │              │ PolicyRepository           │   │
    │  PolicyBundle,  │              │ PolicyBundleRepository     │   │
    │  PolicyEvaluation              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ PolicyDomain   │            │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ PolicyPostgres         │  │
                    │  │ Producer     │  │ Repository             │  │
                    │  └──────────────┘  ├────────────────────────┤  │
                    │  ┌──────────────┐  │ PolicyBundlePostgres   │  │
                    │  │ moka Cache   │  │ Repository             │  │
                    │  │ Service      │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Database               │  │
                    │  │ OPA HTTP     │  │ Config                 │  │
                    │  │ Client       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "policy"
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

opa:
  url: "http://opa.k1s0-system.svc.cluster.local:8181"
  timeout_ms: 2000

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic: "k1s0.system.policy.updated.v1"

cache:
  max_entries: 50000
  ttl_seconds: 30
```

### Helm values

```yaml
# values-policy.yaml（infra/helm/services/system/policy/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/policy
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
    - path: "secret/data/k1s0/system/policy/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```
