# system-session-server 設計

> **認可モデル注記（2026-03-03更新）**: 実装では `resource/action`（例: `sessions/read`, `sessions/write`, `sessions/admin`）で判定し、ロール `sys_admin` / `sys_operator` / `sys_auditor` は middleware でそれぞれ `admin` / `write` / `read` にマッピングされます。


system tier のセッション管理サーバー設計を定義する。Redis によるセッションデータ管理とマルチデバイス対応、セッション失効を提供する。JWT 認証と補完し、ステートフルセッションが必要な要件に対応する。Rust での実装を定義する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| sys_auditor 以上 | sessions/read |
| sys_operator 以上 | sessions/write |
| sys_admin のみ | sessions/admin |


system tier のセッション管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| セッション作成 | ユーザーID・デバイス情報を受け取りセッションを作成。RedisにTTL付きで保存 |
| セッション取得 | セッションIDによるセッション情報取得・有効性確認 |
| セッション更新 | セッションのTTLを延長（スライディング有効期限） |
| セッション失効 | 指定セッションを即時無効化（ログアウト） |
| デバイス別セッション一覧 | ユーザーIDに紐づく全アクティブセッション一覧取得 |
| 全デバイスセッション失効 | ユーザーの全セッションを一括失効（強制ログアウト） |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| セッションストア | Redis 7（redis-rs クレート） |

### 配置パス

配置: `regions/system/server/rust/session/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| セッションストア | Redis 7（TTL による自動失効）。セッションID は UUID v4 |
| セッションメタデータ | PostgreSQL の `session` スキーマ（user_sessions テーブル）にデバイス情報・作成日時を記録 |
| TTL | デフォルト 3600 秒（1 時間）。更新時にスライディング延長 |
| マルチデバイス | 1ユーザーにつき最大10デバイスのセッションを許可。超過時は最古のセッションを自動失効 |
| Kafka | コンシューマー（`k1s0.system.session.revoke_all.v1`）+ プロデューサー（`k1s0.system.session.created.v1`, `k1s0.system.session.revoked.v1`） |
| 認証 | JWT による認可。管理系エンドポイントは `sessions/read`, `sessions/write`, `sessions/admin` を使用 |
| ポート | HTTP 8102（docker-compose で 8102 にマップ） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SESSION_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/sessions` | セッション作成 | JWTトークン必須（ユーザー本人） |
| GET | `/api/v1/sessions/:session_id` | セッション取得 | JWTトークン必須 |
| POST | `/api/v1/sessions/:session_id/refresh` | セッション更新（TTL延長） | JWTトークン必須 |
| DELETE | `/api/v1/sessions/:session_id` | セッション失効 | JWTトークン必須 |
| GET | `/api/v1/users/:user_id/sessions` | ユーザーのセッション一覧 | `sessions/read` |
| DELETE | `/api/v1/users/:user_id/sessions` | ユーザーの全セッション失効 | `sessions/write` |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック（Redis / PostgreSQL / Kafka の疎通確認） | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/sessions

ユーザーID・デバイス情報を受け取り新しいセッションを作成する。Redis に TTL 付きで保存し、PostgreSQL にメタデータを記録する。1ユーザーの最大デバイス数（10）を超過した場合は最古のセッションを自動失効させる。

**リクエスト**

```json
{
  "user_id": "usr_01JABCDEF1234567890",
  "ttl_seconds": 3600,
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "user_agent": "Mozilla/5.0 ...",
  "ip_address": "192.168.1.1",
  "max_devices": 10,
  "metadata": {
    "tenant_id": "tenant-abc",
    "app_version": "1.2.3"
  }
}
```

> `ttl_seconds` は省略可能（デフォルト: 3600秒）。`max_devices` と `metadata` も省略可能。デバイス情報はフラットフィールド（`device_id`, `device_name`, `device_type`, `user_agent`, `ip_address`）で渡す。

**レスポンス（201 Created）**

```json
{
  "session_id": "sess_01JABCDEF1234567890",
  "user_id": "usr_01JABCDEF1234567890",
  "token": "tok_01JABCDEF1234567890",
  "expires_at": "2026-02-23T11:00:00.000+00:00",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "status": "active",
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "user_agent": "Mozilla/5.0 ...",
  "ip_address": "192.168.1.1",
  "metadata": {
    "tenant_id": "tenant-abc",
    "app_version": "1.2.3"
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_SESSION_VALIDATION_ERROR",
    "message": "validation failed",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "user_id", "message": "user_id is required"},
      {"field": "device_id", "message": "device_id is required"}
    ]
  }
}
```

#### GET /api/v1/sessions/:session_id

セッションIDによりセッション情報を取得する。セッションが存在しないか期限切れの場合はエラーを返す。

**レスポンス（200 OK）**

```json
{
  "session_id": "sess_01JABCDEF1234567890",
  "user_id": "usr_01JABCDEF1234567890",
  "token": "tok_01JABCDEF1234567890",
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "user_agent": "Mozilla/5.0 ...",
  "ip_address": "192.168.1.1",
  "metadata": {
    "tenant_id": "tenant-abc",
    "app_version": "1.2.3"
  },
  "status": "active",
  "expires_at": "2026-02-23T11:00:00.000+00:00",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "last_accessed_at": "2026-02-23T10:30:00.000+00:00"
}
```

> `last_accessed_at` は optional フィールド。未アクセスのセッションでは省略される場合がある。
> `token` は高機密情報。ログ出力や外部監視への転送を避け、必要最小限の権限コンテキストでのみ扱うこと。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_SESSION_NOT_FOUND",
    "message": "session not found: sess_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```
#### POST /api/v1/sessions/:session_id/refresh

指定セッションの TTL を延長する（スライディング有効期限）。

**リクエスト（オプション）**

リクエストボディは省略可能。省略時はデフォルト TTL（3600 秒）を使用する。

```json
{
  "ttl_seconds": 7200
}
```

**レスポンス（200 OK）**

```json
{
  "session_id": "sess_01JABCDEF1234567890",
  "user_id": "usr_01JABCDEF1234567890",
  "token": "tok_01JABCDEF1234567890",
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "user_agent": "Mozilla/5.0 ...",
  "ip_address": "192.168.1.1",
  "metadata": {
    "tenant_id": "tenant-abc",
    "app_version": "1.2.3"
  },
  "expires_at": "2026-02-23T12:00:00.000+00:00",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "last_accessed_at": "2026-02-23T10:45:00.000+00:00",
  "status": "active"
}
```

**レスポンス（410 Gone）**

```json
{
  "error": {
    "code": "SYS_SESSION_EXPIRED",
    "message": "session has expired: sess_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```
#### DELETE /api/v1/sessions/:session_id

指定セッションを即時無効化する。Redis から削除し Kafka へ `k1s0.system.session.revoked.v1` を発行する。

**レスポンス（204 No Content）**

レスポンスボディなし。

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_SESSION_ALREADY_REVOKED",
    "message": "session is already revoked: sess_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/users/:user_id/sessions

ユーザーIDに紐づく全アクティブセッションの一覧を取得する。

**レスポンス（200 OK）**

```json
{
  "sessions": [
    {
      "session_id": "sess_01JABCDEF1234567890",
      "user_id": "usr_01JABCDEF1234567890",
      "token": "tok_01JABCDEF1234567890",
      "device_id": "device_abc123",
      "device_name": "MacBook Pro",
      "device_type": "desktop",
      "user_agent": "Mozilla/5.0 ...",
      "ip_address": "192.168.1.1",
      "metadata": {
        "tenant_id": "tenant-abc",
        "app_version": "1.2.3"
      },
      "status": "active",
      "expires_at": "2026-02-23T11:00:00.000+00:00",
      "created_at": "2026-02-23T10:00:00.000+00:00",
      "last_accessed_at": "2026-02-23T10:30:00.000+00:00"
    }
  ],
  "total_count": 1
}
```

> `total_count` は `u32` 相当で返却される。
#### DELETE /api/v1/users/:user_id/sessions

ユーザーの全セッションを一括失効させる。Redis から全セッションを削除し Kafka へ `k1s0.system.session.revoked.v1` を各セッション分発行する。

**レスポンス（200 OK）**

```json
{
  "revoked_count": 3
}
```

**レスポンス（403 Forbidden）**

```json
{
  "error": {
    "code": "SYS_SESSION_FORBIDDEN",
    "message": "operation not permitted for user: usr_01JABCDEF1234567890",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_SESSION_NOT_FOUND` | 404 | 指定されたセッションが見つからない |
| `SYS_SESSION_EXPIRED` | 410 | セッションが有効期限切れ |
| `SYS_SESSION_ALREADY_REVOKED` | 409 | セッションは既に失効済み |
| `SYS_SESSION_MAX_DEVICES_EXCEEDED` | 429 | 最大デバイス数を超過 |
| `SYS_SESSION_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_SESSION_UNAUTHORIZED` | 401 | 認証エラー |
| `SYS_SESSION_FORBIDDEN` | 403 | 権限エラー（他ユーザーのセッション操作） |
| `SYS_SESSION_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
// k1s0 セッション管理サービス gRPC 定義。
// ユーザーセッションのライフサイクル管理を提供する。
syntax = "proto3";

package k1s0.system.session.v1;

option go_package = "github.com/k1s0-platform/system-proto-go/session/v1;sessionv1";

import "k1s0/system/common/v1/types.proto";

service SessionService {
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);
  rpc GetSession(GetSessionRequest) returns (GetSessionResponse);
  rpc RefreshSession(RefreshSessionRequest) returns (RefreshSessionResponse);
  rpc RevokeSession(RevokeSessionRequest) returns (RevokeSessionResponse);
  rpc RevokeAllSessions(RevokeAllSessionsRequest) returns (RevokeAllSessionsResponse);
  rpc ListUserSessions(ListUserSessionsRequest) returns (ListUserSessionsResponse);
}

message CreateSessionRequest {
  string user_id = 1;
  string device_id = 2;
  optional string device_name = 3;
  optional string device_type = 4;
  optional string user_agent = 5;
  optional string ip_address = 6;
  optional uint32 ttl_seconds = 7;
  optional int32 max_devices = 8;
  map<string, string> metadata = 9;
}

message CreateSessionResponse {
  string session_id = 1;
  string user_id = 2;
  string device_id = 3;
  k1s0.system.common.v1.Timestamp expires_at = 4;
  k1s0.system.common.v1.Timestamp created_at = 5;
  string token = 6;
  map<string, string> metadata = 7;
  optional string device_name = 8;
  optional string device_type = 9;
  optional string user_agent = 10;
  optional string ip_address = 11;
  string status = 12;
}

message GetSessionRequest {
  string session_id = 1;
}

message GetSessionResponse {
  Session session = 1;
}

message RefreshSessionRequest {
  string session_id = 1;
  optional uint32 ttl_seconds = 2;
}

message RefreshSessionResponse {
  string session_id = 1;
  k1s0.system.common.v1.Timestamp expires_at = 2;
  string user_id = 3;
  string token = 4;
  string device_id = 5;
  optional string device_name = 6;
  optional string device_type = 7;
  optional string user_agent = 8;
  optional string ip_address = 9;
  map<string, string> metadata = 10;
  k1s0.system.common.v1.Timestamp created_at = 11;
  optional k1s0.system.common.v1.Timestamp last_accessed_at = 12;
  // valid values: "active", "revoked"
  string status = 13;
}

message RevokeSessionRequest {
  string session_id = 1;
}

message RevokeSessionResponse {
  bool success = 1;
}

message RevokeAllSessionsRequest {
  string user_id = 1;
}

message RevokeAllSessionsResponse {
  uint32 revoked_count = 1;
}

message ListUserSessionsRequest {
  string user_id = 1;
}

message ListUserSessionsResponse {
  repeated Session sessions = 1;
  uint32 total_count = 2;
}

message Session {
  string session_id = 1;
  string user_id = 2;
  string device_id = 3;
  optional string device_name = 4;
  optional string device_type = 5;
  optional string user_agent = 6;
  optional string ip_address = 7;
  // valid values: "active", "revoked"
  string status = 8;
  k1s0.system.common.v1.Timestamp expires_at = 9;
  k1s0.system.common.v1.Timestamp created_at = 10;
  optional k1s0.system.common.v1.Timestamp last_accessed_at = 11;
  string token = 12;
}
```

#### Session.status 値仕様

- `active`: セッションが有効（`revoked=false`）
- `revoked`: セッションが失効済み（`revoked=true`）

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Session` | エンティティ定義 |
| domain/repository | `SessionRepository` | リポジトリトレイト |
| domain/service | `SessionDomainService` | TTL計算・デバイス数制限ロジック |
| usecase | `CreateSessionUsecase`, `GetSessionUsecase`, `RefreshSessionUsecase`, `RevokeSessionUsecase`, `RevokeAllSessionsUsecase`, `ListUserSessionsUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic）, Kafka コンシューマー | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `SessionRedisRepository`, `SessionMetadataPostgresRepository` | Redis・PostgreSQL リポジトリ実装 |
| infrastructure/messaging | `SessionRevokeAllKafkaConsumer`, `SessionKafkaProducer` | Kafka コンシューマー・プロデューサー |

### ドメインモデル

#### Session

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `session_id` | String | セッションの一意識別子 |
| `user_id` | String | セッション所有ユーザーID |
| `token` | String | セッショントークン |
| `expires_at` | DateTime\<Utc\> | セッション有効期限 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `revoked` | bool | 失効フラグ |
| `device_id` | String | デバイス ID |
| `device_name` | Option\<String\> | デバイス名 |
| `device_type` | Option\<String\> | デバイスタイプ |
| `user_agent` | Option\<String\> | User-Agent |
| `ip_address` | Option\<String\> | IP アドレス |
| `metadata` | HashMap\<String, String\> | 任意メタデータ |

**メソッド:**
- `is_valid()` -- セッションが有効か判定（未失効かつ未期限切れ）
- `is_expired()` -- 有効期限切れか判定
- `revoke()` -- セッションを失効状態にする
- `refresh(new_expires_at)` -- 有効期限を延長する

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (session_handler.rs)        │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  create_session / get_session /          │   │
                    │  │  refresh_session / revoke_session        │   │
                    │  │  list_user_sessions /                    │   │
                    │  │  revoke_all_sessions                     │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (session_grpc.rs)           │   │
                    │  │  CreateSession / GetSession /            │   │
                    │  │  RefreshSession / RevokeSession /        │   │
                    │  │  RevokeAllSessions / ListUserSessions    │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ Kafka Consumer (session_consumer.rs)     │   │
                    │  │  k1s0.system.session.revoke_all.v1       │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  CreateSessionUsecase /                         │
                    │  GetSessionUsecase /                            │
                    │  RefreshSessionUsecase /                        │
                    │  RevokeSessionUsecase /                         │
                    │  RevokeAllSessionsUsecase /                     │
                    │  ListUserSessionsUsecase                        │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  Session        │              │ SessionRepository          │   │
    └────────────────┘              │ (trait)                    │   │
              │                     └──────────┬─────────────────┘   │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ Session        │            │                     │
                 │ DomainService  │            │                     │
                 │ (TTL計算/      │            │                     │
                 │  デバイス制限) │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │             infrastructure 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ SessionRedis            │  │
                    │  │ Consumer /   │  │ Repository              │  │
                    │  │ Producer     │  │ (Redis 7, TTL)          │  │
                    │  │ revoke_all   │  ├────────────────────────┤  │
                    │  │ .v1 /        │  │ SessionMetadataPostgres │  │
                    │  │ created.v1 / │  │ Repository              │  │
                    │  │ revoked.v1   │  │ (メタデータ)            │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐                              │
                    │  │ Config       │                              │
                    │  │ Loader       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### session 設定フィールド

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `session.default_ttl_seconds` | int | `3600` | 作成時のデフォルト TTL（秒） |
| `session.max_ttl_seconds` | int | `86400` | 許可する最大 TTL（秒） |

### config.yaml（本番）

```yaml
server:
  host: "0.0.0.0"
  port: 8102
  grpc_port: 50051

session:
  default_ttl_seconds: 3600
  max_ttl_seconds: 86400

redis:
  url: "redis://redis.k1s0-system.svc.cluster.local:6379"
  pool_size: 10
  connect_timeout_seconds: 3

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "session-server.default"
  security_protocol: "PLAINTEXT"
  topic_revoke_all: "k1s0.system.session.revoke_all.v1"
  topic_created: "k1s0.system.session.created.v1"
  topic_revoked: "k1s0.system.session.revoked.v1"
```

---

## 詳細設計ドキュメント

- [system-session-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-session-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [認証設計.md](../../architecture/auth/認証設計.md) -- 認証設計
- [JWT設計.md](../../architecture/auth/JWT設計.md) -- JWT 設計

## Doc Sync (2026-03-03)

### Message/Field Corrections
- `CreateSessionRequest.ttl_seconds` is optional.
- `CreateSessionResponse.token` is present.
- `CreateSessionResponse.metadata` is present.
- Session-related timestamps use `k1s0.system.common.v1.Timestamp`.
- `Session.status` valid values are `active` and `revoked`.


### 2026-03-03 追補
- CreateSessionRequest は optional int32 max_devices と map<string,string> metadata を持つ。
- CreateSessionResponse は status を含む。
- Session は token を含む。
- `GET /healthz` は `{"status":"ok","service":"session"}` を返す。
- `GET /readyz` は Redis / PostgreSQL / Kafka の個別チェック結果と `service` を返し、`status` は `ready` / `not_ready` をとる。
- `max_devices` は作成リクエスト専用フィールドであり、Session ドメインモデルの保持対象ではない。
---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
