# system-ratelimit-server ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## API リクエスト・レスポンス例

### POST /api/v1/ratelimit/check

**リクエスト**

```json
{
  "scope": "user",
  "identifier": "user-001",
  "window": "60s"
}
```

**レスポンス（200 OK -- 許可）**

```json
{
  "allowed": true,
  "remaining": 95,
  "reset_at": 1740052260,
  "limit": 100,
  "reason": ""
}
```

**レスポンス（200 OK -- 拒否）**

```json
{
  "allowed": false,
  "remaining": 0,
  "reset_at": 1740052260,
  "limit": 100,
  "reason": "rate limit exceeded for user:user-001"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_RATE_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "scope", "message": "scope must be one of: service, user, endpoint"}
    ]
  }
}
```

### GET /api/v1/ratelimit/usage

**レスポンス（200 OK）**

```json
{
  "used": 42,
  "limit": 100,
  "remaining": 58,
  "reset_at": 1740052260
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RATE_NOT_FOUND",
    "message": "no rate limit state found for user:user-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### GET /api/v1/ratelimit/rules

**レスポンス（200 OK）**

```json
{
  "rules": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "scope": "user",
      "identifier_pattern": "*",
      "limit": 100,
      "window_seconds": 60,
      "enabled": true,
      "created_at": "2026-02-20T10:00:00.000+00:00",
      "updated_at": "2026-02-20T10:00:00.000+00:00"
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "scope": "service",
      "identifier_pattern": "order-service",
      "limit": 1000,
      "window_seconds": 60,
      "enabled": true,
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

### POST /api/v1/ratelimit/rules

**リクエスト**

```json
{
  "scope": "user",
  "identifier_pattern": "*",
  "limit": 100,
  "window_seconds": 60,
  "enabled": true
}
```

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "scope": "user",
  "identifier_pattern": "*",
  "limit": 100,
  "window_seconds": 60,
  "enabled": true,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T10:00:00.000+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_RATE_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "limit", "message": "limit must be greater than 0"},
      {"field": "window_seconds", "message": "window_seconds must be greater than 0"}
    ]
  }
}
```

### PUT /api/v1/ratelimit/rules/:id

**リクエスト**

```json
{
  "scope": "user",
  "identifier_pattern": "*",
  "limit": 200,
  "window_seconds": 60,
  "enabled": true
}
```

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "scope": "user",
  "identifier_pattern": "*",
  "limit": 200,
  "window_seconds": 60,
  "enabled": true,
  "created_at": "2026-02-20T10:00:00.000+00:00",
  "updated_at": "2026-02-20T14:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RATE_NOT_FOUND",
    "message": "rate limit rule not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### DELETE /api/v1/ratelimit/rules/:id

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "rate limit rule 550e8400-e29b-41d4-a716-446655440000 deleted"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RATE_NOT_FOUND",
    "message": "rate limit rule not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### POST /api/v1/ratelimit/reset

**リクエスト**

```json
{
  "scope": "user",
  "identifier": "user-001"
}
```

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "rate limit counter reset for user:user-001"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_RATE_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "scope", "message": "scope must be one of: service, user, endpoint"}
    ]
  }
}
```

---

## トークンバケット実装

### Redis Lua スクリプト

レート制限判定はアトミック性を保証するため、Redis Lua スクリプトで実装する。

```lua
-- token_bucket.lua
-- KEYS[1]: ratelimit:{scope}:{identifier}:{window}
-- ARGV[1]: limit (最大トークン数)
-- ARGV[2]: window_seconds (ウィンドウサイズ秒)
-- ARGV[3]: now (現在の Unix timestamp)

local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1])
local last_refill = tonumber(bucket[2])

if tokens == nil then
  -- 初回アクセス: バケットを初期化
  tokens = limit
  last_refill = now
end

-- トークンの補充
local elapsed = now - last_refill
local refill_rate = limit / window
local new_tokens = math.min(limit, tokens + elapsed * refill_rate)

if new_tokens >= 1 then
  -- トークンを消費
  new_tokens = new_tokens - 1
  redis.call('HMSET', key, 'tokens', new_tokens, 'last_refill', now)
  redis.call('EXPIRE', key, window)
  return {1, math.floor(new_tokens), now + window, limit}
else
  -- トークン不足: 拒否
  redis.call('HMSET', key, 'tokens', new_tokens, 'last_refill', now)
  redis.call('EXPIRE', key, window)
  return {0, 0, now + window, limit}
end
```

### ルールマッチングフロー

リクエストに対するルール検索は以下の優先順位で行う。

```
1. scope + identifier の完全一致ルール
2. scope + ワイルドカード（*）ルール
3. ルールなし → デフォルトルール（設定ファイルで定義）
```

---

## Kong 連携

### Kong プラグイン連携フロー

API ゲートウェイ（Kong）のカスタムプラグインから gRPC で `CheckRateLimit` を呼び出すことで、リクエストのレート制限を実現する。

```
1. クライアントが Kong にリクエスト送信
2. Kong のレート制限プラグインが ratelimit-server に gRPC 呼び出し
3. ratelimit-server が Redis でトークンバケットをチェック
4. CheckRateLimitResponse を Kong に返却
5. allowed=true: Kong がリクエストを上流サービスに転送
   allowed=false: Kong が 429 Too Many Requests を返却
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (ratelimit_handler.rs)      │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  check_ratelimit / get_usage             │   │
                    │  │  list_rules / create_rule                │   │
                    │  │  update_rule / delete_rule               │   │
                    │  │  reset_limit                             │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (ratelimit_grpc.rs)         │   │
                    │  │  CheckRateLimit / GetUsage               │   │
                    │  │  ResetLimit                              │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CheckRateLimitUsecase / GetUsageUsecase /      │
                    │  ResetLimitUsecase / CreateRuleUsecase /        │
                    │  UpdateRuleUsecase / DeleteRuleUsecase          │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  RateLimitRule, │              │ RateLimitRuleRepository    │   │
    │  RateLimitStatus│              │ (PostgreSQL trait)         │   │
    │  RateLimitCheck │              │ RateLimitStateRepository   │   │
    │                 │              │ (Redis trait)              │   │
    └────────────────┘              └──────────┬─────────────────┘   │
              │                                │                     │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ RateLimitDomain│            │                     │
                 │ Service        │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Redis +      │  │ RateLimitRulePostgres  │  │
                    │  │ Lua Script   │  │ Repository             │  │
                    │  │ (State)      │  │ (Rules)                │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Config       │  │ Database               │  │
                    │  │ Loader       │  │ Config                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "ratelimit"
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

redis:
  url: "redis://redis.k1s0-system.svc.cluster.local:6379"
  pool_size: 20
  timeout_ms: 100

ratelimit:
  fail_open: true
  default_limit: 100
  default_window_seconds: 60
```

### Helm values

```yaml
# values-ratelimit.yaml（infra/helm/services/system/ratelimit/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/ratelimit
  tag: ""

replicaCount: 3

container:
  port: 8080
  grpcPort: 9090

service:
  type: ClusterIP
  port: 80
  grpcPort: 9090

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 60

redis:
  enabled: true
  url: ""

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/ratelimit/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
    - path: "secret/data/k1s0/system/ratelimit/redis"
      key: "password"
      mountPath: "/vault/secrets/redis-password"
```
