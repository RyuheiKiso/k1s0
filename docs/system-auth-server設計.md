# system-auth-server 設計

system tier の認証・認可・監査基盤サーバー設計を定義する。全サービスに対して REST と gRPC でトークン検証・ユーザー管理・権限チェック・監査ログ機能を提供する。
Rust で実装する。

## 概要

system tier の認証サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| JWT トークン検証 | Keycloak 発行の JWT を JWKS で検証し、Claims を返却する |
| トークンイントロスペクション | RFC 7662 準拠のトークン情報取得 |
| ユーザー管理 API | Keycloak 連携によるユーザー情報・ロール取得 |
| RBAC 権限チェック | ロールベースのアクセス制御判定 |
| 監査ログ記録・検索 | 認証イベントの記録と検索 |
| ナビゲーション設定 | クライアント向けナビゲーション構成の提供 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC フレームワーク | tonic v0.12 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |
| JWT 検証 | jsonwebtoken + JWKS |
| API ドキュメント | utoipa + utoipa-swagger-ui |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/auth/` |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_AUTH_` とする。

#### 公開エンドポイント（認証不要）

| Method | Path | Description |
| --- | --- | --- |
| GET | `/healthz` | ヘルスチェック |
| GET | `/readyz` | レディネスチェック（DB・Keycloak 接続確認） |
| GET | `/metrics` | Prometheus メトリクス |
| POST | `/api/v1/auth/token/validate` | JWT トークン検証 |
| POST | `/api/v1/auth/token/introspect` | トークンイントロスペクション（RFC 7662） |
| GET | `/api/v1/navigation` | ナビゲーション設定取得 |

#### 保護エンドポイント（Bearer トークン + RBAC 必要）

| Method | Path | Description | RBAC リソース | RBAC 権限 |
| --- | --- | --- | --- | --- |
| GET | `/api/v1/users` | ユーザー一覧取得 | `users` | `read` |
| GET | `/api/v1/users/:id` | ユーザー情報取得 | `users` | `read` |
| GET | `/api/v1/users/:id/roles` | ユーザーロール取得 | `users` | `read` |
| POST | `/api/v1/auth/permissions/check` | 権限チェック | `auth_config` | `read` |
| GET | `/api/v1/audit/logs` | 監査ログ検索 | `audit_logs` | `read` |
| POST | `/api/v1/audit/logs` | 監査ログ記録 | `audit_logs` | `write` |

#### POST /api/v1/auth/token/validate

JWT トークンを検証し、有効であれば Claims を返却する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIs..."
}
```

**レスポンス（200 OK）**

```json
{
  "valid": true,
  "claims": {
    "sub": "user-uuid-1234",
    "iss": "https://auth.k1s0.internal.example.com/realms/k1s0",
    "aud": "k1s0-api",
    "exp": 1740000000,
    "iat": 1739996400,
    "jti": "token-uuid-5678",
    "preferred_username": "taro.yamada",
    "email": "taro.yamada@example.com",
    "scope": "openid profile email",
    "realm_access": {
      "roles": ["sys_auditor"]
    }
  }
}
```

**レスポンス（401 Unauthorized）**

```json
{
  "error": {
    "code": "SYS_AUTH_TOKEN_INVALID",
    "message": "Token validation failed",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/auth/token/introspect

RFC 7662 準拠のトークンイントロスペクション。トークンが無効でも 200 を返し、`active: false` で応答する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIs...",
  "token_type_hint": "access_token"
}
```

**レスポンス（200 OK - アクティブ）**

```json
{
  "active": true,
  "sub": "user-uuid-1234",
  "client_id": "react-spa",
  "username": "taro.yamada",
  "token_type": "Bearer",
  "exp": 1740000000,
  "iat": 1739996400,
  "scope": "openid profile email",
  "realm_access": {
    "roles": ["sys_auditor"]
  }
}
```

**レスポンス（200 OK - 非アクティブ）**

```json
{
  "active": false
}
```

#### GET /api/v1/users

ユーザー一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "users": [
    {
      "id": "user-uuid-1234",
      "username": "taro.yamada",
      "email": "taro.yamada@example.com",
      "display_name": "Taro Yamada",
      "status": "active",
      "email_verified": true,
      "created_at": "2026-01-15T09:00:00Z",
      "attributes": {}
    }
  ],
  "pagination": {
    "total_count": 42,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/users/:id

指定された ID のユーザー情報を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "user-uuid-1234",
  "username": "taro.yamada",
  "email": "taro.yamada@example.com",
  "display_name": "Taro Yamada",
  "status": "active",
  "email_verified": true,
  "created_at": "2026-01-15T09:00:00Z",
  "attributes": {}
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_AUTH_USER_NOT_FOUND",
    "message": "The specified user was not found",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/users/:id/roles

指定されたユーザーの Realm ロールおよびクライアントロールを取得する。

**レスポンス（200 OK）**

```json
{
  "user_id": "user-uuid-1234",
  "realm_roles": [
    {
      "id": "role-1",
      "name": "user",
      "description": "General user"
    },
    {
      "id": "role-2",
      "name": "sys_admin",
      "description": "System admin"
    }
  ],
  "client_roles": {
    "order-service": [
      {
        "id": "role-3",
        "name": "read",
        "description": "Read access"
      }
    ]
  }
}
```

#### POST /api/v1/auth/permissions/check

指定されたロール群が、特定リソースに対する権限を持つか判定する。

**リクエスト**

```json
{
  "roles": ["sys_admin"],
  "permission": "admin",
  "resource": "users"
}
```

**レスポンス（200 OK）**

```json
{
  "allowed": true,
  "reason": ""
}
```

#### GET /api/v1/audit/logs

監査ログをフィルター・ページネーション付きで検索する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 50 | 1ページあたりの件数 |
| `user_id` | string | No | - | ユーザー ID でフィルター |
| `event_type` | string | No | - | イベント種別でフィルター |
| `from` | string | No | - | 検索開始日時（RFC 3339） |
| `to` | string | No | - | 検索終了日時（RFC 3339） |
| `result` | string | No | - | 結果でフィルター（SUCCESS / FAILURE） |

**レスポンス（200 OK）**

```json
{
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "event_type": "LOGIN_SUCCESS",
      "user_id": "user-uuid-1234",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0",
      "resource": "/api/v1/auth/token",
      "resource_id": null,
      "action": "POST",
      "result": "SUCCESS",
      "detail": {"client_id": "react-spa"},
      "trace_id": "trace-001",
      "created_at": "2026-02-20T10:30:00Z"
    }
  ],
  "pagination": {
    "total_count": 1,
    "page": 1,
    "page_size": 50,
    "has_next": false
  }
}
```

#### POST /api/v1/audit/logs

監査ログエントリを記録する。

**リクエスト**

```json
{
  "event_type": "LOGIN_SUCCESS",
  "user_id": "user-uuid-1234",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0",
  "resource": "/api/v1/auth/token",
  "action": "POST",
  "result": "SUCCESS",
  "detail": {"client_id": "react-spa"}
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `event_type` | string | Yes | イベント種別（空文字不可） |
| `user_id` | string | Yes | 操作ユーザー ID |
| `ip_address` | string | Yes | リクエスト元 IP |
| `user_agent` | string | No | User-Agent ヘッダー |
| `resource` | string | Yes | 対象リソースパス |
| `resource_id` | string | No | 対象リソース ID |
| `action` | string | Yes | HTTP メソッドまたは操作種別 |
| `result` | string | Yes | 結果（`SUCCESS` / `FAILURE`） |
| `detail` | object | No | 付加情報（JSON） |
| `trace_id` | string | No | 分散トレーシング ID |

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-02-20T10:30:00Z"
}
```

#### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

#### GET /readyz

PostgreSQL と Keycloak への接続を確認する。

**レスポンス（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "keycloak": "ok"
  }
}
```

**レスポンス（503 Service Unavailable）**

```json
{
  "status": "not ready",
  "checks": {
    "database": "ok",
    "keycloak": "error"
  }
}
```

### gRPC サービス定義

proto ファイルは [API設計.md](API設計.md) D-009 の命名規則に従い、以下に配置する。

```
api/proto/k1s0/system/auth/v1/auth.proto
```

```protobuf
// k1s0/system/auth/v1/auth.proto
syntax = "proto3";
package k1s0.system.auth.v1;

import "google/protobuf/timestamp.proto";
import "v1/common.proto";

// AuthService は認証・認可サービス。
service AuthService {
  // JWT トークンを検証する。
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  // ユーザー情報を取得する。
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  // ユーザー一覧を取得する。
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  // ユーザーのロール一覧を取得する。
  rpc GetUserRoles(GetUserRolesRequest) returns (GetUserRolesResponse);
  // ロールベースのパーミッション確認を行う。
  rpc CheckPermission(CheckPermissionRequest) returns (CheckPermissionResponse);
}

// AuditService は監査ログサービス。
service AuditService {
  // 監査ログエントリを記録する。
  rpc RecordAuditLog(RecordAuditLogRequest) returns (RecordAuditLogResponse);
  // 監査ログを検索する。
  rpc SearchAuditLogs(SearchAuditLogsRequest) returns (SearchAuditLogsResponse);
}
```

#### AuthService RPC 詳細

| RPC | リクエスト | レスポンス | 説明 |
| --- | --- | --- | --- |
| ValidateToken | `ValidateTokenRequest { token }` | `ValidateTokenResponse { valid, claims, error_message }` | JWT 検証。無効時は `valid=false` + `error_message` |
| GetUser | `GetUserRequest { user_id }` | `GetUserResponse { user }` | ユーザー情報取得。未発見時は `NOT_FOUND` |
| ListUsers | `ListUsersRequest { pagination, search, enabled }` | `ListUsersResponse { users, pagination }` | ユーザー一覧取得（ページネーション対応） |
| GetUserRoles | `GetUserRolesRequest { user_id }` | `GetUserRolesResponse { user_id, realm_roles, client_roles }` | ユーザーロール取得。未発見時は `NOT_FOUND` |
| CheckPermission | `CheckPermissionRequest { user_id, permission, resource, roles }` | `CheckPermissionResponse { allowed, reason }` | 権限判定。拒否時は `reason` に理由 |

#### AuditService RPC 詳細

| RPC | リクエスト | レスポンス | 説明 |
| --- | --- | --- | --- |
| RecordAuditLog | `RecordAuditLogRequest { event_type, user_id, ip_address, user_agent, resource, action, result, detail, resource_id, trace_id }` | `RecordAuditLogResponse { id, created_at }` | 監査ログ記録。`event_type` 空は `INVALID_ARGUMENT` |
| SearchAuditLogs | `SearchAuditLogsRequest { pagination, user_id, event_type, from, to, result }` | `SearchAuditLogsResponse { logs, pagination }` | 監査ログ検索（フィルター・ページネーション対応） |

#### 主要メッセージ型

```protobuf
message TokenClaims {
  string sub = 1;
  string iss = 2;
  string aud = 3;
  int64 exp = 4;
  int64 iat = 5;
  string jti = 6;
  string preferred_username = 7;
  string email = 8;
  RealmAccess realm_access = 9;
  map<string, ClientRoles> resource_access = 10;
  repeated string tier_access = 11;
}

message User {
  string id = 1;
  string username = 2;
  string email = 3;
  string display_name = 4;
  string status = 5;          // "active", "suspended", "deleted"
  bool email_verified = 6;
  google.protobuf.Timestamp created_at = 7;
  map<string, StringList> attributes = 8;
}

message AuditLog {
  string id = 1;
  string event_type = 2;
  string user_id = 3;
  string ip_address = 4;
  string user_agent = 5;
  string resource = 6;
  string action = 7;
  string result = 8;
  google.protobuf.Struct detail = 9;
  google.protobuf.Timestamp created_at = 10;
  string resource_id = 11;
  string trace_id = 12;
}
```

---

## Keycloak JWKS 連携フロー

auth-server は Keycloak の JWKS エンドポイントから公開鍵を取得し、JWT の署名を検証する。

### トークン検証シーケンス

```
Client                  auth-server               Keycloak
  |                         |                         |
  |  POST /token/validate   |                         |
  |  { token: "eyJ..." }   |                         |
  |------------------------>|                         |
  |                         |                         |
  |                         |  GET /realms/k1s0/      |
  |                         |  protocol/openid-connect|
  |                         |  /certs                 |
  |                         |------------------------>|
  |                         |                         |
  |                         |  JWKS { keys: [...] }   |
  |                         |<------------------------|
  |                         |                         |
  |                         | 1. kid から公開鍵を特定   |
  |                         | 2. RS256 署名を検証       |
  |                         | 3. iss/aud/exp を検証     |
  |                         |                         |
  |  { valid: true,         |                         |
  |    claims: {...} }      |                         |
  |<------------------------|                         |
```

### JWKS キャッシュ戦略

- JWKS レスポンスはインメモリにキャッシュ
- kid が未知の場合のみ JWKS を再取得（rotate 対応）
- Keycloak の JWKS エンドポイント: `{keycloak.base_url}/realms/{keycloak.realm}/protocol/openid-connect/certs`

### トークン検証の詳細手順

1. JWT ヘッダーから `kid`（Key ID）を抽出
2. キャッシュ済み JWKS から対応する公開鍵を検索
3. 未発見の場合、Keycloak から JWKS を再取得してキャッシュを更新
4. 公開鍵で RS256 署名を検証
5. Claims を検証:
   - `iss` が `auth.jwt.issuer` と一致するか
   - `aud` が `auth.jwt.audience` と一致するか
   - `exp` が現在時刻より未来か
6. 検証成功時、Claims を返却

---

## RBAC ミドルウェア設計

### ミドルウェアスタック

保護エンドポイントには2段階のミドルウェアが適用される。

```
リクエスト
  |
  v
auth_middleware       -- Bearer トークン検証、Claims を Extensions に格納
  |
  v
rbac_middleware       -- Claims のロールが必要な権限を持つか判定
  |
  v
ハンドラー
```

### ロール階層

| ロール | read | write | admin |
| --- | --- | --- | --- |
| `sys_admin` | Yes | Yes | Yes |
| `sys_operator` | Yes | Yes | No |
| `sys_auditor` | Yes | No | No |

`sys_admin` は全権限を持つ。`sys_operator` は read/write、`sys_auditor` は read のみ。

### RBAC エンドポイント設定

| エンドポイントグループ | リソース | 必要権限 |
| --- | --- | --- |
| `/api/v1/users/**` | `users` | `read` |
| `/api/v1/auth/permissions/check` | `auth_config` | `read` |
| `GET /api/v1/audit/logs` | `audit_logs` | `read` |
| `POST /api/v1/audit/logs` | `audit_logs` | `write` |

### パーミッションキャッシュ

moka を使用したインメモリキャッシュで RBAC 判定結果をキャッシュする。

- TTL: `permission_cache.ttl_secs`（デフォルト 300 秒）
- キャッシュミス時の自動リフレッシュ: `permission_cache.refresh_on_miss`

---

## 設定ファイル仕様

`config/config.yaml` の全フィールド定義。

### app

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `name` | string | サービス名（`auth-server`） |
| `version` | string | サービスバージョン |
| `environment` | string | 実行環境（`development` / `staging` / `production`） |

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8080` | REST API ポート |
| `grpc_port` | int | `50051` | gRPC ポート |

### auth.jwks

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `url` | string | - | JWKS エンドポイント URL（例: `http://auth-server:8080/.well-known/jwks.json`）。Keycloak URL から自動導出せず、明示的に指定する |
| `cache_ttl_secs` | int | `3600` | JWKS キャッシュ TTL（秒） |

### auth.jwt

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `issuer` | string | JWT の期待される発行者（Keycloak realm URL） |
| `audience` | string | JWT の期待される audience |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | - | PostgreSQL ホスト |
| `port` | int | `5432` | PostgreSQL ポート |
| `name` | string | - | データベース名 |
| `user` | string | - | 接続ユーザー |
| `password` | string | - | 接続パスワード（Vault 経由で注入） |
| `ssl_mode` | string | `disable` | SSL モード |
| `max_open_conns` | int | `25` | 最大接続数 |
| `max_idle_conns` | int | `5` | 最大アイドル接続数 |
| `conn_max_lifetime` | string | `5m` | 接続の最大生存時間 |

### kafka

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `brokers` | string[] | Kafka ブローカーアドレス一覧 |
| `consumer_group` | string | コンシューマーグループ名 |
| `topics.publish` | string[] | 発行トピック一覧 |

発行トピック:

| トピック名 | 説明 |
| --- | --- |
| `k1s0.system.auth.login.v1` | ログインイベント |
| `k1s0.system.auth.token_validate.v1` | トークン検証イベント |
| `k1s0.system.auth.permission_denied.v1` | 権限拒否イベント |

### keycloak

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `base_url` | string | Keycloak ベース URL |
| `realm` | string | Keycloak realm 名 |
| `client_id` | string | サービスのクライアント ID |
| `client_secret` | string | クライアントシークレット（Vault 経由で注入） |

### keycloak_admin

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `token_cache_ttl_secs` | int | Admin API トークンのキャッシュ TTL（秒） |

### permission_cache

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `ttl_secs` | int | `300` | パーミッションキャッシュの TTL（秒） |
| `refresh_on_miss` | bool | `true` | キャッシュミス時の自動リフレッシュ |

### audit

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `kafka_enabled` | bool | `false` | Kafka への監査イベント配信を有効化 |
| `retention_days` | int | `365` | 監査ログの保持日数（pg_partman によりDB レベルでパーティション管理される） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の4レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース）
  ^
usecase（ビジネスロジック）
  ^
adapter（ハンドラー・プレゼンター・ゲートウェイ）
  ^
infrastructure（DB接続・Kafka プロデューサー・キャッシュ・設定ローダー）
```

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Claims`, `User`, `UserRoles`, `Role`, `AuditLog` | エンティティ定義 |
| domain/repository | `UserRepository`, `AuditLogRepository` | リポジトリインターフェース / トレイト |
| usecase | `ValidateTokenUseCase`, `GetUserUseCase`, `ListUsersUseCase`, `GetUserRolesUseCase`, `CheckPermissionUseCase`, `RecordAuditLogUseCase`, `SearchAuditLogsUseCase` | ユースケース |
| adapter/handler | REST ハンドラー（`auth_handler`, `audit_handler`, `navigation_handler`） | HTTP プロトコル変換 |
| adapter/grpc | gRPC ハンドラー（`AuthGrpcService`, `AuditGrpcService`） | gRPC プロトコル変換 |
| adapter/middleware | `auth_middleware`, `rbac_middleware` | 認証・認可ミドルウェア |
| infrastructure/persistence | PostgreSQL リポジトリ実装 | ユーザー・監査ログの永続化 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/gateway | Keycloak クライアント | JWKS 取得・Admin API 連携 |

### 依存関係図

```
                    ┌──────────────────────────────────────────────────────────────┐
                    │                       adapter 層                             │
                    │  ┌──────────────┐  ┌──────────────┐  ┌───────────────────┐  │
                    │  │ REST Handler │  │ gRPC Handler │  │ auth/rbac         │  │
                    │  │ (auth,audit, │  │ (AuthGrpc,   │  │ middleware        │  │
                    │  │  navigation) │  │  AuditGrpc)  │  │                   │  │
                    │  └──────┬───────┘  └──────┬───────┘  └─────────┬─────────┘  │
                    │         │                  │                    │            │
                    └─────────┼──────────────────┼────────────────────┼────────────┘
                              │                  │                    │
                    ┌─────────▼──────────────────▼────────────────────▼────────────┐
                    │                      usecase 層                              │
                    │  ValidateToken / GetUser / ListUsers / GetUserRoles /        │
                    │  CheckPermission / RecordAuditLog / SearchAuditLogs          │
                    └─────────┬────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/entity  │  │ domain/service │  │ domain/repository   │
    │  Claims, User,  │  │ (RBAC check   │  │ UserRepository      │
    │  AuditLog,      │  │  logic)        │  │ AuditLogRepository  │
    │  Role           │  │                │  │ (interface/trait)    │
    └────────────────┘  └────────────────┘  │                     │
                                            └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │             infrastructure 層         │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ Permission   │  │ PostgreSQL Repository  │  │
                    │  │ Cache (moka) │  │ (impl)                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Keycloak     │  │ Kafka Producer         │  │
                    │  │ Gateway      │  │ (audit events)         │  │
                    │  │ (JWKS/Admin) │  └────────────────────────┘  │
                    │  └──────────────┘  ┌────────────────────────┐  │
                    │  ┌──────────────┐  │ Config Loader          │  │
                    │  │ Telemetry    │  │ (config.yaml)          │  │
                    │  │ (Metrics)    │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 依存サービス

### auth-db (PostgreSQL)

ユーザー情報・ロール・権限・監査ログの永続化を担当する。

| テーブル | 説明 |
| --- | --- |
| `users` | ユーザー基本情報 |
| `roles` | ロール定義 |
| `permissions` | パーミッション定義 |
| `user_roles` | ユーザーとロールの関連 |
| `role_permissions` | ロールとパーミッションの関連 |
| `audit_logs` | 監査ログ |
| `api_keys` | API キー管理 |

接続先: `postgres.k1s0-system.svc.cluster.local:5432/k1s0_system`

### Keycloak

OAuth 2.0 / OpenID Connect の ID プロバイダーとして以下の機能を提供する。

- JWT トークン発行
- JWKS 公開鍵エンドポイント（RS256 署名検証用）
- Admin API（ユーザー情報取得・ロール管理）

接続先: `https://auth.k1s0.internal.example.com`
Realm: `k1s0`

### Kafka

認証関連イベントの非同期配信に使用する。トークン検証・権限チェックの結果は Kafka に自動パブリッシュされる（`audit.kafka_enabled` が `true` の場合）。

| トピック | 用途 |
| --- | --- |
| `k1s0.system.auth.login.v1` | ログイン成功・失敗イベント |
| `k1s0.system.auth.token_validate.v1` | トークン検証イベント（自動パブリッシュ） |
| `k1s0.system.auth.permission_denied.v1` | 権限拒否イベント（自動パブリッシュ） |

接続先: `kafka-0.messaging.svc.cluster.local:9092`
コンシューマーグループ: `auth-server.default`

---

## デプロイ

デプロイに関する詳細（Dockerfile・Helm values・Kubernetes マニフェスト・ヘルスチェック設定等）は以下を参照。

- [system-server-デプロイ設計.md](system-server-デプロイ設計.md)

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST (HTTP) | 8080 | REST API + Swagger UI |
| gRPC | 50051 | gRPC サービス |

---

## 関連ドキュメント

- [認証認可設計.md](認証認可設計.md) -- Keycloak 設定・OAuth 2.0 フロー・RBAC 設計・Vault 戦略
- [API設計.md](API設計.md) -- REST / gRPC / GraphQL 設計・エラーレスポンス・バージョニング
- [config設計.md](config設計.md) -- config.yaml スキーマと環境別管理
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [メッセージング設計.md](メッセージング設計.md) -- Kafka トピック設計・監査イベント配信
- [system-config-server設計.md](system-config-server設計.md) -- 設定管理サーバー設計（同 tier の参考実装）
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](コーディング規約.md) -- Linter・Formatter・命名規則
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャの詳細
