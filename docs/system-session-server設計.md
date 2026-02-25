# system-session-server 設計

system tier のセッション管理サーバー設計を定義する。Redis によるセッションデータ管理とマルチデバイス対応、セッション失効を提供する。JWT 認証と補完し、ステートフルセッションが必要な要件に対応する。Rust での実装を定義する。

## 概要

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

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| セッションストア | Redis 7（redis-rs クレート） |
| DB アクセス | sqlx v0.8（セッションメタデータ） |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/session/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| セッションストア | Redis 7（TTL による自動失効）。セッションID は UUID v4 |
| セッションメタデータ | PostgreSQL の `session` スキーマ（user_sessions テーブル）にデバイス情報・作成日時を記録 |
| TTL | デフォルト 3600 秒（1 時間）。更新時にスライディング延長 |
| マルチデバイス | 1ユーザーにつき最大10デバイスのセッションを許可。超過時は最古のセッションを自動失効 |
| Kafka | コンシューマー（`k1s0.system.session.revoke_all.v1`）+ プロデューサー（`k1s0.system.session.created.v1`, `k1s0.system.session.revoked.v1`） |
| 認証 | JWT による認可。管理系エンドポイントは `sys_operator` / `sys_admin` ロールが必要 |
| ポート | HTTP 8080（docker-compose で 8102 にマップ） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_SESSION_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/sessions` | セッション作成 | JWTトークン必須（ユーザー本人） |
| GET | `/api/v1/sessions/:id` | セッション取得 | JWTトークン必須 |
| POST | `/api/v1/sessions/:id/refresh` | セッション更新（TTL延長） | JWTトークン必須 |
| DELETE | `/api/v1/sessions/:id` | セッション失効 | JWTトークン必須 |
| GET | `/api/v1/users/:user_id/sessions` | ユーザーのセッション一覧 | `sys_auditor` 以上 |
| DELETE | `/api/v1/users/:user_id/sessions` | ユーザーの全セッション失効 | `sys_operator` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/sessions

ユーザーID・デバイス情報を受け取り新しいセッションを作成する。Redis に TTL 付きで保存し、PostgreSQL にメタデータを記録する。1ユーザーの最大デバイス数（10）を超過した場合は最古のセッションを自動失効させる。

**リクエスト**

```json
{
  "user_id": "usr_01JABCDEF1234567890",
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "user_agent": "Mozilla/5.0 ...",
  "ip_address": "192.168.1.1"
}
```

**レスポンス（201 Created）**

```json
{
  "session_id": "sess_01JABCDEF1234567890",
  "user_id": "usr_01JABCDEF1234567890",
  "device_id": "device_abc123",
  "expires_at": "2026-02-23T11:00:00.000+00:00",
  "created_at": "2026-02-23T10:00:00.000+00:00"
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
  "device_id": "device_abc123",
  "device_name": "MacBook Pro",
  "device_type": "desktop",
  "ip_address": "192.168.1.1",
  "expires_at": "2026-02-23T11:00:00.000+00:00",
  "created_at": "2026-02-23T10:00:00.000+00:00",
  "last_accessed_at": "2026-02-23T10:30:00.000+00:00"
}
```

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

#### PUT /api/v1/sessions/:session_id/refresh

指定セッションの TTL をデフォルト値（3600 秒）だけ延長する（スライディング有効期限）。

**レスポンス（200 OK）**

```json
{
  "session_id": "sess_01JABCDEF1234567890",
  "expires_at": "2026-02-23T12:00:00.000+00:00"
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
      "device_id": "device_abc123",
      "device_name": "MacBook Pro",
      "device_type": "desktop",
      "ip_address": "192.168.1.1",
      "expires_at": "2026-02-23T11:00:00.000+00:00",
      "created_at": "2026-02-23T10:00:00.000+00:00",
      "last_accessed_at": "2026-02-23T10:30:00.000+00:00"
    }
  ],
  "total_count": 1
}
```

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
| `SYS_SESSION_MAX_DEVICES_EXCEEDED` | 422 | 最大デバイス数を超過 |
| `SYS_SESSION_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_SESSION_UNAUTHORIZED` | 401 | 認証エラー |
| `SYS_SESSION_FORBIDDEN` | 403 | 権限エラー（他ユーザーのセッション操作） |
| `SYS_SESSION_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.session.v1;

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
}

message CreateSessionResponse {
  string session_id = 1;
  string user_id = 2;
  string device_id = 3;
  string expires_at = 4;
  string created_at = 5;
}

message GetSessionRequest {
  string session_id = 1;
}

message GetSessionResponse {
  Session session = 1;
}

message RefreshSessionRequest {
  string session_id = 1;
}

message RefreshSessionResponse {
  string session_id = 1;
  string expires_at = 2;
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
  optional string ip_address = 6;
  string expires_at = 7;
  string created_at = 8;
  string last_accessed_at = 9;
}
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー・Kafka コンシューマー）
  ^
infrastructure（Redis接続・DB接続・Kafka Producer/Consumer・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Session`, `UserSession` | エンティティ定義 |
| domain/repository | `SessionRepository`, `UserSessionRepository` | リポジトリトレイト |
| domain/service | `SessionDomainService` | TTL計算・デバイス数制限ロジック |
| usecase | `CreateSessionUsecase`, `GetSessionUsecase`, `RefreshSessionUsecase`, `RevokeSessionUsecase`, `RevokeAllSessionsUsecase`, `ListUserSessionsUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic）, Kafka コンシューマー | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `SessionRedisRepository`, `UserSessionPostgresRepository` | Redis・PostgreSQL リポジトリ実装 |
| infrastructure/messaging | `SessionRevokeAllKafkaConsumer`, `SessionKafkaProducer` | Kafka コンシューマー・プロデューサー |

### ドメインモデル

#### Session

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | セッションの一意識別子 |
| `user_id` | String | セッション所有ユーザーID |
| `token` | String | セッショントークン |
| `expires_at` | DateTime\<Utc\> | セッション有効期限 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `revoked` | bool | 失効フラグ |
| `metadata` | HashMap\<String, String\> | セッションメタデータ（デバイス情報等を格納可能） |

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
    │  Session,       │              │ SessionRepository          │   │
    │  UserSession    │              │ UserSessionRepository      │   │
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
                    │  │ .v1 /        │  │ UserSessionPostgres     │  │
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

## 詳細設計ドキュメント

- [system-session-server-実装設計.md](system-session-server-実装設計.md) -- 実装設計の詳細
- [system-session-server-デプロイ設計.md](system-session-server-デプロイ設計.md) -- デプロイ設計の詳細

## 関連ドキュメント

- [認証設計.md](認証設計.md) -- 認証設計
- [認証認可設計.md](認証認可設計.md) -- RBAC 認可モデル
- [JWT設計.md](JWT設計.md) -- JWT 設計
- [メッセージング設計.md](メッセージング設計.md) -- Kafka メッセージング設計
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](コーディング規約.md) -- コーディング規約
- [system-library-概要.md](system-library-概要.md) -- ライブラリ一覧
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
