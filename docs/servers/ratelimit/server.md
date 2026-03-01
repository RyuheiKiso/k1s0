# system-ratelimit-server 設計

Redis トークンバケットによるレート制限判定サーバー。Kong 連携・内部サービス間保護を提供。

## 概要

system tier のレートリミットサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| レート制限判定 | サービス・ユーザー・エンドポイントをキーとしたレート制限チェック |
| リミット設定管理 | リミットルールの作成・更新・削除・一覧取得 |
| リミットリセット | 緊急時の特定キーのリミットカウンターリセット |
| 使用量照会 | 現在の使用量・残余リクエスト数・リセット時刻の取得 |
| Lua スクリプト | Redis Lua スクリプトによるアトミックなトークンバケット実装 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Redis | redis crate v0.27（Lua スクリプト対応） |

### 配置パス

配置: `regions/system/server/rust/ratelimit/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) のレート制限方針に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| アルゴリズム | トークンバケット（Redis Lua スクリプトでアトミック実装） |
| キー設計 | `ratelimit:{scope}:{identifier}:{window}` 形式（scope: service/user/endpoint） |
| ウィンドウ | 固定ウィンドウ（60 秒）と設定可能ウィンドウをサポート |
| ルール永続化 | PostgreSQL の `ratelimit` スキーマ。Redis は判定状態のみ保持 |
| Redis 障害時 | フェイルオープン（障害時はリミットを通過させる）。設定で変更可能 |
| Kong 連携 | Kong プラグインから gRPC で `CheckRateLimit` を呼び出す |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_RATELIMIT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/ratelimit/check` | レート制限チェック | 不要（内部サービス用） |
| POST | `/api/v1/ratelimit/rules` | ルール作成 | `sys_operator` 以上 |
| GET | `/api/v1/ratelimit/rules/:id` | ルール取得 | `sys_auditor` 以上 |
| GET | `/api/v1/ratelimit/usage` | 使用量照会 | `sys_auditor` 以上 |
| GET | `/api/v1/ratelimit/rules` | ルール一覧取得 | `sys_auditor` 以上 |
| PUT | `/api/v1/ratelimit/rules/:id` | ルール更新 | `sys_operator` 以上 |
| DELETE | `/api/v1/ratelimit/rules/:id` | ルール削除 | `sys_admin` のみ |
| POST | `/api/v1/ratelimit/reset` | カウンターリセット | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/ratelimit/check

指定されたスコープ・識別子に対してレート制限チェックを行う。Redis のトークンバケットからトークンを消費し、許可/拒否を返す。内部サービス用のエンドポイントであり、認証は不要。

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
    "code": "SYS_RATELIMIT_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "scope", "message": "scope must be one of: service, user, endpoint"}
    ]
  }
}
```

#### GET /api/v1/ratelimit/usage

指定されたルール ID のルール情報および使用状況を照会する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `rule_id` | string | Yes | - | ルール ID（UUID） |

**レスポンス（200 OK）**

```json
{
  "rule_id": "550e8400-e29b-41d4-a716-446655440000",
  "rule_name": "user-global",
  "limit": 100,
  "window_seconds": 60,
  "algorithm": "token_bucket",
  "enabled": true,
  "used": 42,
  "remaining": 58,
  "reset_at": 1740052260
}
```

フィールド `used`, `remaining`, `reset_at` は Redis 状態が存在する場合のみ含まれる（省略可）。

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_RATELIMIT_VALIDATION_ERROR",
    "message": "rule_id is required",
    "request_id": "req_abc123def456"
  }
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RATELIMIT_RULE_NOT_FOUND",
    "message": "rule not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456"
  }
}
```

#### GET /api/v1/ratelimit/rules

レートリミットルール一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `scope` | string | No | - | スコープでフィルタ |
| `enabled_only` | bool | No | false | 有効なルールのみ取得 |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

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

#### POST /api/v1/ratelimit/rules

新しいレートリミットルールを作成する。

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
    "code": "SYS_RATELIMIT_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "limit", "message": "limit must be greater than 0"},
      {"field": "window_seconds", "message": "window_seconds must be greater than 0"}
    ]
  }
}
```

#### PUT /api/v1/ratelimit/rules/:id

既存のレートリミットルールを更新する。

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
    "code": "SYS_RATELIMIT_RULE_NOT_FOUND",
    "message": "rule not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456"
  }
}
```

#### DELETE /api/v1/ratelimit/rules/:id

レートリミットルールを削除する。

**レスポンス（204 No Content）**

ボディなし。削除成功時は 204 を返す。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_RATELIMIT_RULE_NOT_FOUND",
    "message": "rule not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456"
  }
}
```

#### POST /api/v1/ratelimit/reset

指定されたスコープ・識別子のレートリミットカウンターをリセットする。緊急時に使用する。

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
    "code": "SYS_RATELIMIT_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "scope", "message": "scope must be one of: service, user, endpoint"}
    ]
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_RATELIMIT_RULE_NOT_FOUND` | 404 | 指定されたルールが見つからない |
| `SYS_RATELIMIT_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_RATELIMIT_RULE_EXISTS` | 409 | ルールがすでに存在する |
| `SYS_RATELIMIT_ERROR` | 400 | レート制限チェックの汎用エラー |
| `SYS_RATELIMIT_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.ratelimit.v1;

import "google/protobuf/timestamp.proto";

service RateLimitService {
  rpc CheckRateLimit(CheckRateLimitRequest) returns (CheckRateLimitResponse);
  rpc CreateRule(CreateRuleRequest) returns (CreateRuleResponse);
  rpc GetRule(GetRuleRequest) returns (GetRuleResponse);
  rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);
  rpc ResetLimit(ResetLimitRequest) returns (ResetLimitResponse);
}

message CheckRateLimitRequest {
  string rule_id = 1;
  string subject = 2;
}

message CheckRateLimitResponse {
  bool allowed = 1;
  int64 remaining = 2;
  int64 reset_at = 3;
  string reason = 4;
}

message CreateRuleRequest {
  string name = 1;
  string key = 2;
  int64 limit = 3;
  int64 window_secs = 4;
  string algorithm = 5;
}

message CreateRuleResponse {
  RateLimitRule rule = 1;
}

message GetRuleRequest {
  string rule_id = 1;
}

message GetRuleResponse {
  RateLimitRule rule = 1;
}

message RateLimitRule {
  string id = 1;
  string name = 2;
  string key = 3;
  int64 limit = 4;
  int64 window_secs = 5;
  string algorithm = 6;
  bool enabled = 7;
  google.protobuf.Timestamp created_at = 8;
}

message GetUsageRequest {
  string rule_id = 1;
}

message GetUsageResponse {
  string rule_id = 1;
  string rule_name = 2;
  int64 limit = 3;
  int64 window_secs = 4;
  string algorithm = 5;
  bool enabled = 6;
}

message ResetLimitRequest {
  string scope = 1;
  string identifier = 2;
}

message ResetLimitResponse {
  bool success = 1;
}
```

---

## トークンバケット実装

### キー設計

| スコープ | キーフォーマット | 例 |
| --- | --- | --- |
| service | `ratelimit:service:{service_name}:{window}` | `ratelimit:service:order-service:60` |
| user | `ratelimit:user:{user_id}:{window}` | `ratelimit:user:user-001:60` |
| endpoint | `ratelimit:endpoint:{path}:{window}` | `ratelimit:endpoint:/api/v1/orders:60` |

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

### フェイルオープン動作

Redis が利用不能な場合のフォールバック動作。

| 設定値 | 動作 |
| --- | --- |
| `fail_open: true`（デフォルト） | Redis 障害時はリクエストを許可（`allowed: true`, `reason: "redis unavailable, fail-open"` ） |
| `fail_open: false` | Redis 障害時はリクエストを拒否（`allowed: false`, `reason: "redis unavailable, fail-closed"`） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `RateLimitRule`, `RateLimitStatus`, `RateLimitCheck` | エンティティ定義 |
| domain/repository | `RateLimitRuleRepository`（PostgreSQL）, `RateLimitStateRepository`（Redis） | リポジトリトレイト |
| domain/service | `RateLimitDomainService` | トークンバケット判定ロジック |
| usecase | `CheckRateLimitUsecase`, `GetUsageUsecase`, `ResetLimitUsecase`, `CreateRuleUsecase`, `UpdateRuleUsecase`, `DeleteRuleUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `RateLimitRulePostgresRepository` | PostgreSQL リポジトリ実装（ルール永続化） |
| infrastructure/cache | `RateLimitRedisRepository` + Lua スクリプト | Redis リポジトリ実装（状態管理） |

### ドメインモデル

#### RateLimitRule

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | ルールの一意識別子 |
| `scope` | String | スコープ（service / user / endpoint） |
| `identifier_pattern` | String | 識別子パターン（`*` でワイルドカード、特定値で個別指定） |
| `limit` | u32 | ウィンドウあたりの最大リクエスト数 |
| `window_seconds` | u32 | ウィンドウサイズ（秒） |
| `enabled` | bool | ルールの有効/無効 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |

#### RateLimitStatus

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `key` | String | Redis キー（`ratelimit:{scope}:{identifier}:{window}`） |
| `used` | u32 | 現在のウィンドウでの使用数 |
| `limit` | u32 | ウィンドウあたりの最大リクエスト数 |
| `remaining` | u32 | 残余リクエスト数 |
| `reset_at` | DateTime\<Utc\> | ウィンドウリセット時刻 |

#### RateLimitCheck

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `scope` | String | チェック対象のスコープ |
| `identifier` | String | チェック対象の識別子 |
| `allowed` | bool | 許可/拒否の判定結果 |
| `remaining` | u32 | 残余リクエスト数 |
| `reset_at` | DateTime\<Utc\> | ウィンドウリセット時刻 |
| `rule_id` | UUID | 適用されたルールの ID |

### ルールマッチング

リクエストに対するルール検索の優先順位: (1) scope + identifier 完全一致 → (2) scope + ワイルドカード(`*`) → (3) デフォルトルール。

---

## Kong 連携

### レスポンスヘッダー

Kong プラグインは ratelimit-server のレスポンスに基づいて以下のヘッダーを付与する。

| ヘッダー | 値 | 説明 |
| --- | --- | --- |
| `X-RateLimit-Limit` | `100` | ウィンドウあたりの最大リクエスト数 |
| `X-RateLimit-Remaining` | `95` | 残余リクエスト数 |
| `X-RateLimit-Reset` | `1740052260` | リセット時刻（Unix timestamp） |
| `Retry-After` | `45` | 429 レスポンス時のみ、リトライまでの秒数 |

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
                    │  │  CheckRateLimit / CreateRule             │   │
                    │  │  GetRule / GetUsage / ResetLimit         │   │
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
  grpc_port: 50051

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
  grpcPort: 50051

service:
  type: ClusterIP
  port: 80
  grpcPort: 50051

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

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/ratelimit/database` |
| Redis パスワード | `secret/data/k1s0/system/ratelimit/redis` |

---

## 詳細設計ドキュメント

- [system-ratelimit-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-ratelimit-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) -- Kong API ゲートウェイ設計
