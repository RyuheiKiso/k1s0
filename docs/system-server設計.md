# system-server 設計

system tier の認証サーバー設計を定義する。Keycloak 26.0 と連携し、JWT トークン検証・ユーザー情報管理・監査ログ記録を提供する。
Go と Rust の両実装を対等に定義する。

## 概要

system tier の認証サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ユーザー情報の取得・管理 API | Keycloak Admin API と連携してユーザー情報を取得・管理する |
| JWT トークンの検証・イントロスペクション | JWKS エンドポイントから公開鍵を取得し、JWT の署名検証・有効期限検証を行う |
| RBAC による認可制御 | Keycloak の `realm_access.roles` と `resource_access` に基づいてパーミッションを判定する |
| 監査ログの記録 | 認証・認可イベントを PostgreSQL に記録し、Kafka に非同期配信する |

### 技術スタック

| コンポーネント | Go | Rust |
| --- | --- | --- |
| HTTP フレームワーク | gin v1.10 | axum + tokio |
| gRPC フレームワーク | google.golang.org/grpc v1.68 | tonic v0.12 |
| JWT ライブラリ | github.com/lestrrat-go/jwx/v2 | jsonwebtoken v9 |
| DB アクセス | github.com/jmoiron/sqlx | sqlx v0.8 |
| Kafka | github.com/segmentio/kafka-go | rdkafka (rust-rdkafka) |
| OTel | go.opentelemetry.io/otel v1.31 | opentelemetry v0.27 |
| 設定管理 | gopkg.in/yaml.v3 | serde_yaml |
| バリデーション | go-playground/validator/v10 | validator v0.18 |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Go | `regions/system/server/go/auth/` |
| Rust | `regions/system/server/rust/auth/` |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_AUTH_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/auth/token/validate` | トークン検証 | 不要（公開） |
| POST | `/api/v1/auth/token/introspect` | トークンイントロスペクション | `sys_operator` 以上 |
| GET | `/api/v1/users/:id` | ユーザー情報取得 | `sys_auditor` 以上 |
| GET | `/api/v1/users` | ユーザー一覧 | `sys_auditor` 以上 |
| GET | `/api/v1/users/:id/roles` | ユーザーロール取得 | `sys_auditor` 以上 |
| POST | `/api/v1/audit/logs` | 監査ログ記録 | `sys_operator` 以上 |
| GET | `/api/v1/audit/logs` | 監査ログ検索 | `sys_auditor` 以上 |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/auth/token/validate

JWT トークンの署名・有効期限・issuer・audience を検証し、Claims を返却する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ii..."
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
    "exp": 1710000900,
    "iat": 1710000000,
    "jti": "token-uuid-5678",
    "typ": "Bearer",
    "azp": "react-spa",
    "scope": "openid profile email",
    "preferred_username": "taro.yamada",
    "email": "taro.yamada@example.com",
    "realm_access": {
      "roles": ["user", "order_manager"]
    },
    "resource_access": {
      "order-service": {
        "roles": ["read", "write"]
      }
    },
    "tier_access": ["system", "business", "service"]
  }
}
```

**レスポンス（401 Unauthorized）**

```json
{
  "error": {
    "code": "SYS_AUTH_TOKEN_INVALID",
    "message": "トークンの署名検証に失敗しました",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/auth/token/introspect

[RFC 7662](https://tools.ietf.org/html/rfc7662) に準拠したトークンイントロスペクション。Keycloak のイントロスペクションエンドポイントに委譲する。

**リクエスト**

```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ii...",
  "token_type_hint": "access_token"
}
```

**レスポンス（200 OK）**

```json
{
  "active": true,
  "sub": "user-uuid-1234",
  "client_id": "react-spa",
  "username": "taro.yamada",
  "token_type": "Bearer",
  "exp": 1710000900,
  "iat": 1710000000,
  "scope": "openid profile email",
  "realm_access": {
    "roles": ["user", "order_manager"]
  }
}
```

**レスポンス（200 OK - 無効トークン）**

```json
{
  "active": false
}
```

#### GET /api/v1/users/:id

Keycloak Admin API からユーザー情報を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "user-uuid-1234",
  "username": "taro.yamada",
  "email": "taro.yamada@example.com",
  "first_name": "太郎",
  "last_name": "山田",
  "enabled": true,
  "email_verified": true,
  "created_at": "2024-01-15T09:30:00Z",
  "attributes": {
    "department": ["engineering"],
    "employee_id": ["EMP001"]
  }
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_AUTH_USER_NOT_FOUND",
    "message": "指定されたユーザーが見つかりません",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/users

ユーザー一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1ページあたりの件数（最大 100） |
| `search` | string | No | - | ユーザー名・メールアドレスで部分一致検索 |
| `enabled` | bool | No | - | 有効/無効フィルタ |

**レスポンス（200 OK）**

```json
{
  "users": [
    {
      "id": "user-uuid-1234",
      "username": "taro.yamada",
      "email": "taro.yamada@example.com",
      "first_name": "太郎",
      "last_name": "山田",
      "enabled": true
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

#### GET /api/v1/users/:id/roles

ユーザーに割り当てられたロール一覧を取得する。

**レスポンス（200 OK）**

```json
{
  "user_id": "user-uuid-1234",
  "realm_roles": [
    {
      "id": "role-uuid-1",
      "name": "user",
      "description": "一般ユーザー"
    },
    {
      "id": "role-uuid-2",
      "name": "sys_auditor",
      "description": "監査担当"
    }
  ],
  "client_roles": {
    "order-service": [
      {
        "id": "role-uuid-3",
        "name": "read",
        "description": "読み取り権限"
      }
    ]
  }
}
```

#### POST /api/v1/audit/logs

監査ログエントリを記録する。内部サービスからの呼び出しを想定する。

**リクエスト**

```json
{
  "event_type": "LOGIN_SUCCESS",
  "user_id": "user-uuid-1234",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0 ...",
  "resource": "/api/v1/auth/token",
  "action": "POST",
  "result": "SUCCESS",
  "metadata": {
    "client_id": "react-spa",
    "grant_type": "authorization_code"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "audit-uuid-5678",
  "recorded_at": "2026-02-17T10:30:00.123Z"
}
```

#### GET /api/v1/audit/logs

監査ログを検索する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 50 | 1ページあたりの件数（最大 200） |
| `user_id` | string | No | - | ユーザー ID でフィルタ |
| `event_type` | string | No | - | イベント種別でフィルタ |
| `from` | string | No | - | 開始日時（ISO 8601） |
| `to` | string | No | - | 終了日時（ISO 8601） |
| `result` | string | No | - | `SUCCESS` / `FAILURE` |

**レスポンス（200 OK）**

```json
{
  "logs": [
    {
      "id": "audit-uuid-5678",
      "event_type": "LOGIN_SUCCESS",
      "user_id": "user-uuid-1234",
      "ip_address": "192.168.1.100",
      "resource": "/api/v1/auth/token",
      "action": "POST",
      "result": "SUCCESS",
      "recorded_at": "2026-02-17T10:30:00.123Z",
      "metadata": {
        "client_id": "react-spa"
      }
    }
  ],
  "pagination": {
    "total_count": 5000,
    "page": 1,
    "page_size": 50,
    "has_next": true
  }
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
    "keycloak": "error: connection timeout"
  }
}
```

### gRPC サービス定義

proto ファイルは [API設計.md](API設計.md) D-009 の命名規則に従い、サービス内の `api/proto/` に配置する。

```
{auth-server}/api/proto/
└── k1s0/
    └── system/
        └── auth/
            └── v1/
                └── auth.proto
```

```protobuf
// k1s0/system/auth/v1/auth.proto
syntax = "proto3";
package k1s0.system.auth.v1;

import "k1s0/system/common/v1/types.proto";

service AuthService {
  // JWT トークン検証
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);

  // ユーザー情報取得
  rpc GetUser(GetUserRequest) returns (GetUserResponse);

  // ユーザー一覧取得
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);

  // ユーザーロール取得
  rpc GetUserRoles(GetUserRolesRequest) returns (GetUserRolesResponse);

  // パーミッション確認
  rpc CheckPermission(CheckPermissionRequest) returns (CheckPermissionResponse);
}

// --- Token Validation ---

message ValidateTokenRequest {
  string token = 1;
}

message ValidateTokenResponse {
  bool valid = 1;
  TokenClaims claims = 2;
  string error_message = 3;  // valid == false の場合のエラー理由
}

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

message RealmAccess {
  repeated string roles = 1;
}

message ClientRoles {
  repeated string roles = 1;
}

// --- User ---

message GetUserRequest {
  string user_id = 1;
}

message GetUserResponse {
  User user = 1;
}

message ListUsersRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
  string search = 2;
  optional bool enabled = 3;
}

message ListUsersResponse {
  repeated User users = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}

message User {
  string id = 1;
  string username = 2;
  string email = 3;
  string first_name = 4;
  string last_name = 5;
  bool enabled = 6;
  bool email_verified = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
  map<string, StringList> attributes = 9;
}

message StringList {
  repeated string values = 1;
}

// --- Roles ---

message GetUserRolesRequest {
  string user_id = 1;
}

message GetUserRolesResponse {
  string user_id = 1;
  repeated Role realm_roles = 2;
  map<string, RoleList> client_roles = 3;
}

message Role {
  string id = 1;
  string name = 2;
  string description = 3;
}

message RoleList {
  repeated Role roles = 1;
}

// --- Permission Check ---

message CheckPermissionRequest {
  string user_id = 1;
  string permission = 2;   // read, write, delete, admin
  string resource = 3;     // users, auth_config, audit_logs, etc.
  repeated string roles = 4;  // JWT Claims から取得したロール一覧
}

message CheckPermissionResponse {
  bool allowed = 1;
  string reason = 2;       // 拒否理由（allowed == false の場合）
}
```

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
infra（DB接続・Keycloak クライアント・Kafka・設定ローダー）
```

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/model | `User`, `Role`, `Permission`, `AuditLog` | エンティティ定義 |
| domain/repository | `UserRepository`, `AuditLogRepository` | リポジトリインターフェース / トレイト |
| domain/service | `AuthDomainService` | ドメインサービス（パーミッション解決ロジック） |
| usecase | `ValidateTokenUsecase`, `GetUserUsecase`, `ListUsersUsecase`, `GetUserRolesUsecase`, `CheckPermissionUsecase`, `RecordAuditLogUsecase`, `SearchAuditLogUsecase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換 |
| adapter/presenter | レスポンスフォーマット | ドメインモデル → API レスポンス変換 |
| adapter/gateway | Keycloak クライアント | 外部サービスとの通信 |
| infra/persistence | PostgreSQL リポジトリ実装 | 監査ログの永続化 |
| infra/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infra/messaging | Kafka プロデューサー | 監査イベントの非同期配信 |
| infra/auth | JWKS 検証 | JWT 署名検証（JWKS キャッシュ管理） |

### ドメインモデル

#### User

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string (UUID) | ユーザーの一意識別子 |
| `username` | string | ログイン ID |
| `email` | string | メールアドレス |
| `first_name` | string | 名 |
| `last_name` | string | 姓 |
| `enabled` | bool | アカウント有効/無効 |
| `email_verified` | bool | メール認証済み |
| `created_at` | timestamp | 作成日時 |
| `attributes` | map<string, string[]> | カスタム属性（部署、社員番号等） |

#### Role

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string (UUID) | ロールの一意識別子 |
| `name` | string | ロール名（例: `sys_admin`） |
| `description` | string | ロールの説明 |

#### Permission

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `role` | string | 対象ロール名 |
| `permission` | string | 操作種別（`read`, `write`, `delete`, `admin`） |
| `resource` | string | 対象リソース名 |

#### AuditLog

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string (UUID) | 監査ログの一意識別子 |
| `event_type` | string | イベント種別（`LOGIN_SUCCESS`, `LOGIN_FAILURE`, `TOKEN_VALIDATE`, `PERMISSION_DENIED` 等） |
| `user_id` | string (UUID) | 操作者のユーザー ID |
| `ip_address` | string | クライアントの IP アドレス |
| `user_agent` | string | User-Agent ヘッダー |
| `resource` | string | アクセス対象リソース |
| `action` | string | HTTP メソッドまたは gRPC メソッド名 |
| `result` | string | `SUCCESS` / `FAILURE` |
| `metadata` | map<string, string> | 追加メタデータ（client_id、grant_type 等） |
| `recorded_at` | timestamp | 記録日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────────┐
                    │                    adapter 層                       │
                    │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
                    │  │ REST Handler │  │ gRPC Handler │  │ Presenter│ │
                    │  └──────┬───────┘  └──────┬───────┘  └─────┬────┘ │
                    │         │                  │                │      │
                    └─────────┼──────────────────┼────────────────┼──────┘
                              │                  │                │
                    ┌─────────▼──────────────────▼────────────────▼──────┐
                    │                   usecase 層                       │
                    │  ValidateToken / GetUser / ListUsers /             │
                    │  GetUserRoles / CheckPermission /                  │
                    │  RecordAuditLog / SearchAuditLog                   │
                    └─────────┬──────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/model   │  │ domain/service │  │ domain/repository   │
    │  User, Role,    │  │ AuthDomain     │  │ UserRepository      │
    │  Permission,    │  │ Service        │  │ AuditLogRepository  │
    │  AuditLog       │  │                │  │ (interface/trait)    │
    └────────────────┘  └────────────────┘  └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │                  infra 層         │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ Keycloak     │  │ PostgreSQL Repository  │  │
                    │  │ Gateway      │  │ (impl)                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ JWKS         │  │ Kafka Producer         │  │
                    │  │ Verifier     │  │ (audit events)         │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## Go 実装 (regions/system/server/go/auth/)

### ディレクトリ構成

```
regions/system/server/go/auth/
├── cmd/
│   └── main.go                          # エントリポイント
├── internal/
│   ├── domain/
│   │   ├── model/
│   │   │   ├── user.go                  # User エンティティ
│   │   │   ├── role.go                  # Role エンティティ
│   │   │   ├── permission.go            # Permission エンティティ
│   │   │   └── audit_log.go             # AuditLog エンティティ
│   │   ├── repository/
│   │   │   ├── user_repository.go       # UserRepository インターフェース
│   │   │   ├── audit_log_repository.go  # AuditLogRepository インターフェース
│   │   │   └── mock_*.go               # gomock 生成モック
│   │   └── service/
│   │       └── auth_domain_service.go   # パーミッション解決ロジック
│   ├── usecase/
│   │   ├── validate_token.go            # トークン検証
│   │   ├── get_user.go                  # ユーザー情報取得
│   │   ├── list_users.go               # ユーザー一覧
│   │   ├── get_user_roles.go           # ユーザーロール取得
│   │   ├── check_permission.go          # パーミッション確認
│   │   ├── record_audit_log.go          # 監査ログ記録
│   │   ├── search_audit_log.go          # 監査ログ検索
│   │   └── *_test.go                    # 各ユースケースのテスト
│   ├── adapter/
│   │   ├── handler/
│   │   │   ├── rest_handler.go          # REST ハンドラー
│   │   │   ├── grpc_handler.go          # gRPC ハンドラー
│   │   │   ├── error.go                 # エラーレスポンス
│   │   │   └── *_test.go               # ハンドラーテスト
│   │   ├── presenter/
│   │   │   └── response.go              # レスポンスフォーマット
│   │   ├── gateway/
│   │   │   ├── keycloak_client.go       # Keycloak Admin API クライアント
│   │   │   └── keycloak_client_test.go
│   │   └── middleware/
│   │       ├── auth.go                  # JWT 認証ミドルウェア
│   │       └── rbac.go                  # RBAC ミドルウェア
│   └── infra/
│       ├── config/
│       │   ├── config.go                # Config 構造体・ローダー
│       │   └── logger.go               # 構造化ログ初期化
│       ├── persistence/
│       │   ├── db.go                    # DB 接続
│       │   ├── audit_log_repository.go  # AuditLog リポジトリ実装
│       │   ├── migrations/
│       │   │   └── 001_create_audit_logs.sql
│       │   └── *_test.go               # リポジトリテスト
│       ├── auth/
│       │   ├── jwks.go                  # JWKS 検証
│       │   └── jwks_test.go
│       └── messaging/
│           ├── producer.go              # Kafka プロデューサー
│           └── producer_test.go
├── api/
│   ├── openapi/
│   │   ├── openapi.yaml                 # OpenAPI 定義
│   │   └── gen.yaml                     # oapi-codegen 設定
│   └── proto/
│       └── k1s0/system/auth/v1/
│           └── auth.proto               # gRPC サービス定義
├── config/
│   ├── config.yaml                      # デフォルト設定
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── go.mod
├── go.sum
├── Dockerfile
└── README.md
```

### go.mod

```go
module github.com/k1s0/regions/system/server/go/auth

go 1.23

require (
    github.com/gin-gonic/gin v1.10.0
    github.com/go-playground/validator/v10 v10.22.1
    github.com/jmoiron/sqlx v1.4.0
    github.com/lestrrat-go/jwx/v2 v2.1.3
    github.com/lib/pq v1.10.9
    github.com/oapi-codegen/oapi-codegen/v2 v2.4.1
    github.com/oapi-codegen/runtime v1.1.1
    github.com/segmentio/kafka-go v0.4.47
    github.com/stretchr/testify v1.9.0
    go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin v0.56.0
    go.opentelemetry.io/otel v1.31.0
    go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.31.0
    go.opentelemetry.io/otel/sdk v1.31.0
    go.uber.org/mock v0.5.0
    google.golang.org/grpc v1.68.0
    google.golang.org/protobuf v1.35.2
    gopkg.in/yaml.v3 v3.0.1
)
```

### cmd/main.go

```go
package main

import (
    "context"
    "fmt"
    "log/slog"
    "net"
    "net/http"
    "os"
    "os/signal"
    "syscall"
    "time"

    "github.com/gin-gonic/gin"
    "github.com/prometheus/client_golang/prometheus/promhttp"
    "go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"
    "google.golang.org/grpc"

    "github.com/k1s0/regions/system/server/go/auth/internal/adapter/gateway"
    "github.com/k1s0/regions/system/server/go/auth/internal/adapter/handler"
    "github.com/k1s0/regions/system/server/go/auth/internal/adapter/middleware"
    "github.com/k1s0/regions/system/server/go/auth/internal/domain/service"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/auth"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/config"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/messaging"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/persistence"
    "github.com/k1s0/regions/system/server/go/auth/internal/usecase"
)

func main() {
    // --- Config ---
    cfg, err := config.Load("config/config.yaml")
    if err != nil {
        slog.Error("failed to load config", "error", err)
        os.Exit(1)
    }
    if err := cfg.Validate(); err != nil {
        slog.Error("config validation failed", "error", err)
        os.Exit(1)
    }

    // --- Logger ---
    logger := config.NewLogger(
        cfg.App.Environment, cfg.App.Name, cfg.App.Version, cfg.App.Tier,
    )
    slog.SetDefault(logger)

    // --- OpenTelemetry ---
    tp, err := config.InitTracer(context.Background(), cfg.App.Name)
    if err != nil {
        slog.Error("failed to init tracer", "error", err)
        os.Exit(1)
    }
    defer tp.Shutdown(context.Background())

    // --- Database ---
    db, err := persistence.NewDB(cfg.Database)
    if err != nil {
        slog.Error("failed to connect database", "error", err)
        os.Exit(1)
    }
    defer db.Close()

    // --- Kafka ---
    producer := messaging.NewProducer(cfg.Kafka)
    defer producer.Close()

    // --- DI ---
    jwksVerifier := auth.NewJWKSVerifier(cfg.Auth.OIDC.JWKSURI, cfg.Auth.OIDC.JWKSCacheTTL)
    keycloakClient := gateway.NewKeycloakClient(cfg.Auth.OIDC)
    auditRepo := persistence.NewAuditLogRepository(db)
    authDomainSvc := service.NewAuthDomainService()

    // Usecases
    validateTokenUC := usecase.NewValidateTokenUseCase(jwksVerifier, cfg.Auth.JWT)
    getUserUC := usecase.NewGetUserUseCase(keycloakClient)
    listUsersUC := usecase.NewListUsersUseCase(keycloakClient)
    getUserRolesUC := usecase.NewGetUserRolesUseCase(keycloakClient)
    checkPermissionUC := usecase.NewCheckPermissionUseCase(authDomainSvc)
    recordAuditLogUC := usecase.NewRecordAuditLogUseCase(auditRepo, producer)
    searchAuditLogUC := usecase.NewSearchAuditLogUseCase(auditRepo)

    // --- REST Router ---
    r := gin.New()
    r.Use(gin.Recovery())
    r.Use(otelgin.Middleware(cfg.App.Name))
    r.Use(middleware.RequestID())

    // ヘルスチェック
    r.GET("/healthz", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{"status": "ok"})
    })
    r.GET("/readyz", handler.ReadyzHandler(db, keycloakClient))
    r.GET("/metrics", gin.WrapH(promhttp.Handler()))

    // API ルート
    restHandler := handler.NewRESTHandler(
        validateTokenUC, getUserUC, listUsersUC,
        getUserRolesUC, recordAuditLogUC, searchAuditLogUC,
    )
    restHandler.RegisterRoutes(r)

    // --- gRPC Server ---
    grpcServer := grpc.NewServer(
        grpc.ChainUnaryInterceptor(
            handler.UnaryLoggingInterceptor(logger),
            handler.UnaryTracingInterceptor(),
        ),
    )
    grpcHandler := handler.NewGRPCHandler(
        validateTokenUC, getUserUC, listUsersUC,
        getUserRolesUC, checkPermissionUC,
    )
    grpcHandler.Register(grpcServer)

    // --- Start Servers ---
    // REST
    srv := &http.Server{
        Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
        Handler:      r,
        ReadTimeout:  cfg.Server.ReadTimeout,
        WriteTimeout: cfg.Server.WriteTimeout,
    }
    go func() {
        slog.Info("REST server starting", "port", cfg.Server.Port)
        if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
            slog.Error("REST server failed", "error", err)
            os.Exit(1)
        }
    }()

    // gRPC
    go func() {
        lis, err := net.Listen("tcp", fmt.Sprintf(":%d", cfg.GRPC.Port))
        if err != nil {
            slog.Error("gRPC listen failed", "error", err)
            os.Exit(1)
        }
        slog.Info("gRPC server starting", "port", cfg.GRPC.Port)
        if err := grpcServer.Serve(lis); err != nil {
            slog.Error("gRPC server failed", "error", err)
            os.Exit(1)
        }
    }()

    // --- Graceful Shutdown ---
    quit := make(chan os.Signal, 1)
    signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
    <-quit
    slog.Info("shutting down servers...")

    ctx, cancel := context.WithTimeout(context.Background(), cfg.Server.ShutdownTimeout)
    defer cancel()

    grpcServer.GracefulStop()
    if err := srv.Shutdown(ctx); err != nil {
        slog.Error("REST server forced to shutdown", "error", err)
    }
    slog.Info("servers exited")
}
```

### ドメインモデル（Go）

```go
// internal/domain/model/user.go
package model

import "time"

// User は Keycloak ユーザーを表すドメインエンティティ。
type User struct {
    ID            string            `json:"id"`
    Username      string            `json:"username"`
    Email         string            `json:"email"`
    FirstName     string            `json:"first_name"`
    LastName      string            `json:"last_name"`
    Enabled       bool              `json:"enabled"`
    EmailVerified bool              `json:"email_verified"`
    CreatedAt     time.Time         `json:"created_at"`
    Attributes    map[string][]string `json:"attributes,omitempty"`
}
```

```go
// internal/domain/model/role.go
package model

// Role はロールを表す。
type Role struct {
    ID          string `json:"id"`
    Name        string `json:"name"`
    Description string `json:"description"`
}

// Permission はロールに紐づくパーミッションを表す。
type Permission struct {
    Role       string `json:"role"`
    Permission string `json:"permission"` // read, write, delete, admin
    Resource   string `json:"resource"`
}
```

```go
// internal/domain/model/audit_log.go
package model

import "time"

// AuditLog は監査ログエントリを表す。
type AuditLog struct {
    ID         string            `json:"id" db:"id"`
    EventType  string            `json:"event_type" db:"event_type"`
    UserID     string            `json:"user_id" db:"user_id"`
    IPAddress  string            `json:"ip_address" db:"ip_address"`
    UserAgent  string            `json:"user_agent" db:"user_agent"`
    Resource   string            `json:"resource" db:"resource"`
    Action     string            `json:"action" db:"action"`
    Result     string            `json:"result" db:"result"`
    Metadata   map[string]string `json:"metadata" db:"metadata"`
    RecordedAt time.Time         `json:"recorded_at" db:"recorded_at"`
}
```

### リポジトリインターフェース（Go）

```go
// internal/domain/repository/audit_log_repository.go
package repository

//go:generate mockgen -source=audit_log_repository.go -destination=mock_audit_log_repository.go -package=repository

import (
    "context"

    "github.com/k1s0/regions/system/server/go/auth/internal/domain/model"
)

// AuditLogRepository は監査ログの永続化インターフェース。
type AuditLogRepository interface {
    Create(ctx context.Context, log *model.AuditLog) error
    Search(ctx context.Context, params AuditLogSearchParams) ([]*model.AuditLog, int, error)
}

// AuditLogSearchParams は監査ログ検索パラメータ。
type AuditLogSearchParams struct {
    UserID    string
    EventType string
    Result    string
    From      *time.Time
    To        *time.Time
    Page      int
    PageSize  int
}
```

### ユースケース（Go）

```go
// internal/usecase/validate_token.go
package usecase

import (
    "context"

    "go.opentelemetry.io/otel"

    "github.com/k1s0/regions/system/server/go/auth/internal/infra/auth"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/config"
)

// ValidateTokenUseCase は JWT トークン検証ユースケース。
type ValidateTokenUseCase struct {
    verifier  *auth.JWKSVerifier
    jwtConfig config.JWTConfig
}

func NewValidateTokenUseCase(verifier *auth.JWKSVerifier, jwtConfig config.JWTConfig) *ValidateTokenUseCase {
    return &ValidateTokenUseCase{
        verifier:  verifier,
        jwtConfig: jwtConfig,
    }
}

// Execute はトークンを検証し、Claims を返却する。
func (uc *ValidateTokenUseCase) Execute(ctx context.Context, tokenString string) (*auth.TokenClaims, error) {
    ctx, span := otel.Tracer("auth-server").Start(ctx, "ValidateTokenUseCase.Execute")
    defer span.End()

    claims, err := uc.verifier.VerifyToken(ctx, tokenString)
    if err != nil {
        return nil, err
    }

    // issuer・audience の追加検証
    if claims.Issuer != uc.jwtConfig.Issuer {
        return nil, ErrInvalidIssuer
    }
    if claims.Audience != uc.jwtConfig.Audience {
        return nil, ErrInvalidAudience
    }

    return claims, nil
}
```

### REST ハンドラー（Go）

```go
// internal/adapter/handler/rest_handler.go
package handler

import (
    "net/http"

    "github.com/gin-gonic/gin"

    "github.com/k1s0/regions/system/server/go/auth/internal/adapter/middleware"
    "github.com/k1s0/regions/system/server/go/auth/internal/usecase"
)

type RESTHandler struct {
    validateTokenUC  *usecase.ValidateTokenUseCase
    getUserUC        *usecase.GetUserUseCase
    listUsersUC      *usecase.ListUsersUseCase
    getUserRolesUC   *usecase.GetUserRolesUseCase
    recordAuditLogUC *usecase.RecordAuditLogUseCase
    searchAuditLogUC *usecase.SearchAuditLogUseCase
}

func NewRESTHandler(
    validateTokenUC *usecase.ValidateTokenUseCase,
    getUserUC *usecase.GetUserUseCase,
    listUsersUC *usecase.ListUsersUseCase,
    getUserRolesUC *usecase.GetUserRolesUseCase,
    recordAuditLogUC *usecase.RecordAuditLogUseCase,
    searchAuditLogUC *usecase.SearchAuditLogUseCase,
) *RESTHandler {
    return &RESTHandler{
        validateTokenUC:  validateTokenUC,
        getUserUC:        getUserUC,
        listUsersUC:      listUsersUC,
        getUserRolesUC:   getUserRolesUC,
        recordAuditLogUC: recordAuditLogUC,
        searchAuditLogUC: searchAuditLogUC,
    }
}

func (h *RESTHandler) RegisterRoutes(r *gin.Engine) {
    v1 := r.Group("/api/v1")

    // 公開エンドポイント（認可不要）
    auth := v1.Group("/auth")
    {
        auth.POST("/token/validate", h.ValidateToken)
        auth.POST("/token/introspect",
            middleware.RequirePermission("read", "auth_config"),
            h.IntrospectToken,
        )
    }

    // ユーザー管理（sys_auditor 以上）
    users := v1.Group("/users")
    users.Use(middleware.RequirePermission("read", "users"))
    {
        users.GET("", h.ListUsers)
        users.GET("/:id", h.GetUser)
        users.GET("/:id/roles", h.GetUserRoles)
    }

    // 監査ログ
    audit := v1.Group("/audit")
    {
        audit.POST("/logs",
            middleware.RequirePermission("write", "audit_logs"),
            h.RecordAuditLog,
        )
        audit.GET("/logs",
            middleware.RequirePermission("read", "audit_logs"),
            h.SearchAuditLogs,
        )
    }
}

func (h *RESTHandler) ValidateToken(c *gin.Context) {
    var req struct {
        Token string `json:"token" binding:"required"`
    }
    if err := c.ShouldBindJSON(&req); err != nil {
        WriteError(c, http.StatusBadRequest, "SYS_AUTH_VALIDATION_FAILED",
            "リクエストのバリデーションに失敗しました")
        return
    }

    claims, err := h.validateTokenUC.Execute(c.Request.Context(), req.Token)
    if err != nil {
        WriteError(c, http.StatusUnauthorized, "SYS_AUTH_TOKEN_INVALID",
            "トークンの検証に失敗しました")
        return
    }

    c.JSON(http.StatusOK, gin.H{
        "valid":  true,
        "claims": claims,
    })
}

func (h *RESTHandler) GetUser(c *gin.Context) {
    userID := c.Param("id")

    user, err := h.getUserUC.Execute(c.Request.Context(), userID)
    if err != nil {
        WriteError(c, http.StatusNotFound, "SYS_AUTH_USER_NOT_FOUND",
            "指定されたユーザーが見つかりません")
        return
    }

    c.JSON(http.StatusOK, user)
}

// ... 他のハンドラーメソッドも同様のパターンで実装
```

### gRPC ハンドラー（Go）

```go
// internal/adapter/handler/grpc_handler.go
package handler

import (
    "context"

    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"

    pb "github.com/k1s0/regions/system/server/go/auth/api/proto/k1s0/system/auth/v1"
    "github.com/k1s0/regions/system/server/go/auth/internal/usecase"
)

type GRPCHandler struct {
    pb.UnimplementedAuthServiceServer
    validateTokenUC   *usecase.ValidateTokenUseCase
    getUserUC         *usecase.GetUserUseCase
    listUsersUC       *usecase.ListUsersUseCase
    getUserRolesUC    *usecase.GetUserRolesUseCase
    checkPermissionUC *usecase.CheckPermissionUseCase
}

func NewGRPCHandler(
    validateTokenUC *usecase.ValidateTokenUseCase,
    getUserUC *usecase.GetUserUseCase,
    listUsersUC *usecase.ListUsersUseCase,
    getUserRolesUC *usecase.GetUserRolesUseCase,
    checkPermissionUC *usecase.CheckPermissionUseCase,
) *GRPCHandler {
    return &GRPCHandler{
        validateTokenUC:   validateTokenUC,
        getUserUC:         getUserUC,
        listUsersUC:       listUsersUC,
        getUserRolesUC:    getUserRolesUC,
        checkPermissionUC: checkPermissionUC,
    }
}

func (h *GRPCHandler) Register(s *grpc.Server) {
    pb.RegisterAuthServiceServer(s, h)
}

func (h *GRPCHandler) ValidateToken(
    ctx context.Context, req *pb.ValidateTokenRequest,
) (*pb.ValidateTokenResponse, error) {
    claims, err := h.validateTokenUC.Execute(ctx, req.Token)
    if err != nil {
        return &pb.ValidateTokenResponse{
            Valid:        false,
            ErrorMessage: err.Error(),
        }, nil
    }

    return &pb.ValidateTokenResponse{
        Valid:  true,
        Claims: toProtoClaims(claims),
    }, nil
}

func (h *GRPCHandler) GetUser(
    ctx context.Context, req *pb.GetUserRequest,
) (*pb.GetUserResponse, error) {
    user, err := h.getUserUC.Execute(ctx, req.UserId)
    if err != nil {
        return nil, status.Error(codes.NotFound, "user not found")
    }

    return &pb.GetUserResponse{
        User: toProtoUser(user),
    }, nil
}

func (h *GRPCHandler) CheckPermission(
    ctx context.Context, req *pb.CheckPermissionRequest,
) (*pb.CheckPermissionResponse, error) {
    allowed, reason := h.checkPermissionUC.Execute(
        ctx, req.UserId, req.Permission, req.Resource, req.Roles,
    )
    return &pb.CheckPermissionResponse{
        Allowed: allowed,
        Reason:  reason,
    }, nil
}
```

### Keycloak クライアント（Go）

```go
// internal/adapter/gateway/keycloak_client.go
package gateway

import (
    "context"
    "encoding/json"
    "fmt"
    "net/http"
    "sync"
    "time"

    "go.opentelemetry.io/otel"

    "github.com/k1s0/regions/system/server/go/auth/internal/domain/model"
    "github.com/k1s0/regions/system/server/go/auth/internal/infra/config"
)

// KeycloakClient は Keycloak Admin API と通信するゲートウェイ。
type KeycloakClient struct {
    baseURL      string
    realm        string
    clientID     string
    clientSecret string
    httpClient   *http.Client

    mu          sync.RWMutex
    adminToken  string
    tokenExpiry time.Time
}

func NewKeycloakClient(cfg *config.OIDCConfig) *KeycloakClient {
    return &KeycloakClient{
        baseURL:      extractBaseURL(cfg.DiscoveryURL),
        realm:        "k1s0",
        clientID:     cfg.ClientID,
        clientSecret: cfg.ClientSecret,
        httpClient: &http.Client{
            Timeout: 10 * time.Second,
        },
    }
}

// GetUser は Keycloak Admin API からユーザー情報を取得する。
func (c *KeycloakClient) GetUser(ctx context.Context, userID string) (*model.User, error) {
    ctx, span := otel.Tracer("auth-server").Start(ctx, "KeycloakClient.GetUser")
    defer span.End()

    token, err := c.getAdminToken(ctx)
    if err != nil {
        return nil, fmt.Errorf("failed to get admin token: %w", err)
    }

    url := fmt.Sprintf("%s/admin/realms/%s/users/%s", c.baseURL, c.realm, userID)
    req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
    if err != nil {
        return nil, err
    }
    req.Header.Set("Authorization", "Bearer "+token)

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("keycloak request failed: %w", err)
    }
    defer resp.Body.Close()

    if resp.StatusCode == http.StatusNotFound {
        return nil, ErrUserNotFound
    }
    if resp.StatusCode != http.StatusOK {
        return nil, fmt.Errorf("keycloak returned status %d", resp.StatusCode)
    }

    var user model.User
    if err := json.NewDecoder(resp.Body).Decode(&user); err != nil {
        return nil, fmt.Errorf("failed to decode user: %w", err)
    }
    return &user, nil
}

// Healthy は Keycloak への接続を確認する。
func (c *KeycloakClient) Healthy(ctx context.Context) error {
    url := fmt.Sprintf("%s/realms/%s", c.baseURL, c.realm)
    req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
    if err != nil {
        return err
    }
    resp, err := c.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("keycloak health check failed: %w", err)
    }
    defer resp.Body.Close()
    if resp.StatusCode != http.StatusOK {
        return fmt.Errorf("keycloak returned status %d", resp.StatusCode)
    }
    return nil
}

// getAdminToken は Client Credentials で管理用トークンを取得する（キャッシュ付き）。
func (c *KeycloakClient) getAdminToken(ctx context.Context) (string, error) {
    c.mu.RLock()
    if c.adminToken != "" && time.Now().Before(c.tokenExpiry) {
        defer c.mu.RUnlock()
        return c.adminToken, nil
    }
    c.mu.RUnlock()

    c.mu.Lock()
    defer c.mu.Unlock()

    // Client Credentials Grant で取得
    // POST /realms/k1s0/protocol/openid-connect/token
    // grant_type=client_credentials&client_id=...&client_secret=...
    // ...（実装省略）

    return c.adminToken, nil
}
```

---

## Rust 実装 (regions/system/server/rust/auth/)

### ディレクトリ構成

```
regions/system/server/rust/auth/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs                  # User エンティティ
│   │   │   ├── role.rs                  # Role エンティティ
│   │   │   ├── permission.rs            # Permission エンティティ
│   │   │   └── audit_log.rs             # AuditLog エンティティ
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── user_repository.rs       # UserRepository トレイト
│   │   │   └── audit_log_repository.rs  # AuditLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── auth_domain_service.rs   # パーミッション解決ロジック
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── validate_token.rs
│   │   ├── get_user.rs
│   │   ├── list_users.rs
│   │   ├── get_user_roles.rs
│   │   ├── check_permission.rs
│   │   ├── record_audit_log.rs
│   │   └── search_audit_log.rs
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── rest_handler.rs          # axum REST ハンドラー
│   │   │   ├── grpc_handler.rs          # tonic gRPC ハンドラー
│   │   │   └── error.rs                 # エラーレスポンス
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── response.rs
│   │   ├── gateway/
│   │   │   ├── mod.rs
│   │   │   └── keycloak_client.rs       # Keycloak Admin API クライアント
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infra/
│       ├── mod.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── logger.rs
│       ├── persistence/
│       │   ├── mod.rs
│       │   ├── db.rs
│       │   └── audit_log_repository.rs
│       ├── auth/
│       │   ├── mod.rs
│       │   └── jwks.rs                  # JWKS 検証
│       └── messaging/
│           ├── mod.rs
│           └── producer.rs              # Kafka プロデューサー
├── api/
│   └── proto/
│       └── k1s0/system/auth/v1/
│           └── auth.proto
├── migrations/
│   └── 001_create_audit_logs.sql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
├── build.rs                             # tonic-build（proto コンパイル）
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
└── README.md
```

### Cargo.toml

```toml
[package]
name = "auth-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = { version = "1", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# JWT
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# OpenTelemetry
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"

# ユーティリティ
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
validator = { version = "0.18", features = ["derive"] }

# メトリクス
prometheus = "0.13"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
testcontainers = "0.23"

[build-dependencies]
tonic-build = "0.12"
```

### build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["api/proto/k1s0/system/auth/v1/auth.proto"],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

### src/main.rs

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tonic::transport::Server as TonicServer;
use tracing::info;

mod adapter;
mod domain;
mod infra;
mod usecase;

use adapter::gateway::KeycloakClient;
use adapter::handler::{grpc_handler, rest_handler};
use domain::service::AuthDomainService;
use infra::auth::JwksVerifier;
use infra::config::Config;
use infra::messaging::KafkaProducer;
use infra::persistence;
use usecase::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Config ---
    let cfg = Config::load("config/config.yaml")?;
    cfg.validate()?;

    // --- Logger ---
    infra::config::init_logger(&cfg.app.environment);

    // --- OpenTelemetry ---
    let _tracer = infra::config::init_tracer(&cfg.app.name)?;

    // --- Database ---
    let pool = persistence::connect(&cfg.database).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // --- Kafka ---
    let producer = KafkaProducer::new(&cfg.kafka)?;

    // --- DI ---
    let jwks_verifier = Arc::new(
        JwksVerifier::new(
            cfg.auth.oidc.jwks_uri.clone(),
            cfg.auth.oidc.jwks_cache_ttl(),
        ),
    );
    let keycloak_client = Arc::new(KeycloakClient::new(&cfg.auth.oidc));
    let audit_repo = Arc::new(persistence::AuditLogRepositoryImpl::new(pool.clone()));
    let auth_domain_svc = Arc::new(AuthDomainService::new());

    let validate_token_uc = ValidateTokenUseCase::new(
        jwks_verifier.clone(), cfg.auth.jwt.clone(),
    );
    let get_user_uc = GetUserUseCase::new(keycloak_client.clone());
    let list_users_uc = ListUsersUseCase::new(keycloak_client.clone());
    let get_user_roles_uc = GetUserRolesUseCase::new(keycloak_client.clone());
    let check_permission_uc = CheckPermissionUseCase::new(auth_domain_svc.clone());
    let record_audit_log_uc = RecordAuditLogUseCase::new(
        audit_repo.clone(), Arc::new(producer),
    );
    let search_audit_log_uc = SearchAuditLogUseCase::new(audit_repo.clone());

    let app_state = AppState {
        validate_token_uc: Arc::new(validate_token_uc),
        get_user_uc: Arc::new(get_user_uc),
        list_users_uc: Arc::new(list_users_uc),
        get_user_roles_uc: Arc::new(get_user_roles_uc),
        record_audit_log_uc: Arc::new(record_audit_log_uc),
        search_audit_log_uc: Arc::new(search_audit_log_uc),
        keycloak_client: keycloak_client.clone(),
        pool: pool.clone(),
    };

    // --- REST Server (axum) ---
    let rest_app = rest_handler::router(app_state);
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));

    let rest_handle = tokio::spawn(async move {
        info!("REST server starting on {}", rest_addr);
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        axum::serve(listener, rest_app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });

    // --- gRPC Server (tonic) ---
    let grpc_addr = SocketAddr::from(([0, 0, 0, 0], cfg.grpc.port));
    let grpc_service = grpc_handler::AuthServiceImpl::new(
        Arc::new(ValidateTokenUseCase::new(jwks_verifier, cfg.auth.jwt.clone())),
        Arc::new(GetUserUseCase::new(keycloak_client.clone())),
        Arc::new(ListUsersUseCase::new(keycloak_client.clone())),
        Arc::new(GetUserRolesUseCase::new(keycloak_client)),
        Arc::new(CheckPermissionUseCase::new(auth_domain_svc)),
    );

    let grpc_handle = tokio::spawn(async move {
        info!("gRPC server starting on {}", grpc_addr);
        TonicServer::builder()
            .add_service(grpc_handler::auth_service_server(grpc_service))
            .serve_with_shutdown(grpc_addr, shutdown_signal())
            .await
            .unwrap();
    });

    tokio::try_join!(rest_handle, grpc_handle)?;
    info!("servers exited");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
```

### ドメインモデル（Rust）

```rust
// src/domain/model/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub enabled: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub attributes: HashMap<String, Vec<String>>,
}
```

```rust
// src/domain/model/role.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub role: String,
    pub permission: String, // read, write, delete, admin
    pub resource: String,
}
```

```rust
// src/domain/model/audit_log.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub resource: String,
    pub action: String,
    pub result: String,
    #[sqlx(json)]
    pub metadata: HashMap<String, String>,
    pub recorded_at: DateTime<Utc>,
}
```

### リポジトリトレイト（Rust）

```rust
// src/domain/repository/audit_log_repository.rs
use async_trait::async_trait;

use crate::domain::model::AuditLog;

#[derive(Debug, Clone)]
pub struct AuditLogSearchParams {
    pub user_id: Option<String>,
    pub event_type: Option<String>,
    pub result: Option<String>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub page: i32,
    pub page_size: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn create(&self, log: &AuditLog) -> anyhow::Result<()>;
    async fn search(
        &self,
        params: &AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<AuditLog>, i64)>;
}
```

### ユースケース（Rust）

```rust
// src/usecase/validate_token.rs
use std::sync::Arc;

use tracing::instrument;

use crate::infra::auth::JwksVerifier;
use crate::infra::config::JwtConfig;

pub struct ValidateTokenUseCase {
    verifier: Arc<JwksVerifier>,
    jwt_config: JwtConfig,
}

impl ValidateTokenUseCase {
    pub fn new(verifier: Arc<JwksVerifier>, jwt_config: JwtConfig) -> Self {
        Self { verifier, jwt_config }
    }

    #[instrument(skip(self, token), fields(service = "auth-server"))]
    pub async fn execute(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let claims = self.verifier.verify_token(token).await?;

        // issuer・audience の追加検証
        if claims.iss != self.jwt_config.issuer {
            return Err(AuthError::InvalidIssuer);
        }
        if claims.aud != self.jwt_config.audience {
            return Err(AuthError::InvalidAudience);
        }

        Ok(claims)
    }
}
```

### REST ハンドラー（Rust）

```rust
// src/adapter/handler/rest_handler.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::adapter::handler::error::ErrorResponse;
use crate::adapter::middleware;

pub fn router(state: AppState) -> Router {
    Router::new()
        // ヘルスチェック
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        // 公開エンドポイント
        .route("/api/v1/auth/token/validate", post(validate_token))
        // 認可付きエンドポイント
        .route(
            "/api/v1/auth/token/introspect",
            post(introspect_token).layer(middleware::require_permission("read", "auth_config")),
        )
        .route(
            "/api/v1/users",
            get(list_users).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id",
            get(get_user).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/users/:id/roles",
            get(get_user_roles).layer(middleware::require_permission("read", "users")),
        )
        .route(
            "/api/v1/audit/logs",
            post(record_audit_log)
                .layer(middleware::require_permission("write", "audit_logs"))
                .get(search_audit_logs)
                .layer(middleware::require_permission("read", "audit_logs")),
        )
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .is_ok();

    let kc_ok = state.keycloak_client.healthy().await.is_ok();

    if db_ok && kc_ok {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "checks": {"database": "ok", "keycloak": "ok"}
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[derive(Deserialize)]
struct ValidateTokenRequest {
    token: String,
}

async fn validate_token(
    State(state): State<AppState>,
    Json(req): Json<ValidateTokenRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let claims = state
        .validate_token_uc
        .execute(&req.token)
        .await
        .map_err(|_| {
            ErrorResponse::unauthorized(
                "SYS_AUTH_TOKEN_INVALID",
                "トークンの検証に失敗しました",
            )
        })?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "claims": claims,
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let user = state.get_user_uc.execute(&id).await.map_err(|_| {
        ErrorResponse::not_found(
            "SYS_AUTH_USER_NOT_FOUND",
            "指定されたユーザーが見つかりません",
        )
    })?;

    Ok(Json(serde_json::to_value(user).unwrap()))
}

// ... 他のハンドラーも同様のパターンで実装
```

### gRPC ハンドラー（Rust）

```rust
// src/adapter/handler/grpc_handler.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("k1s0.system.auth.v1");
}

use proto::auth_service_server::{AuthService, AuthServiceServer};
use proto::*;

pub struct AuthServiceImpl {
    validate_token_uc: Arc<ValidateTokenUseCase>,
    get_user_uc: Arc<GetUserUseCase>,
    list_users_uc: Arc<ListUsersUseCase>,
    get_user_roles_uc: Arc<GetUserRolesUseCase>,
    check_permission_uc: Arc<CheckPermissionUseCase>,
}

impl AuthServiceImpl {
    pub fn new(
        validate_token_uc: Arc<ValidateTokenUseCase>,
        get_user_uc: Arc<GetUserUseCase>,
        list_users_uc: Arc<ListUsersUseCase>,
        get_user_roles_uc: Arc<GetUserRolesUseCase>,
        check_permission_uc: Arc<CheckPermissionUseCase>,
    ) -> Self {
        Self {
            validate_token_uc,
            get_user_uc,
            list_users_uc,
            get_user_roles_uc,
            check_permission_uc,
        }
    }
}

pub fn auth_service_server(svc: AuthServiceImpl) -> AuthServiceServer<AuthServiceImpl> {
    AuthServiceServer::new(svc)
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.validate_token_uc.execute(&req.token).await {
            Ok(claims) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                claims: Some(claims.into()),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                claims: None,
                error_message: e.to_string(),
            })),
        }
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .get_user_uc
            .execute(&req.user_id)
            .await
            .map_err(|_| Status::not_found("user not found"))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user.into()),
        }))
    }

    async fn check_permission(
        &self,
        request: Request<CheckPermissionRequest>,
    ) -> Result<Response<CheckPermissionResponse>, Status> {
        let req = request.into_inner();

        let (allowed, reason) = self
            .check_permission_uc
            .execute(&req.user_id, &req.permission, &req.resource, &req.roles)
            .await;

        Ok(Response::new(CheckPermissionResponse {
            allowed,
            reason,
        }))
    }

    // ListUsers, GetUserRoles も同様のパターンで実装
}
```

### Keycloak クライアント（Rust）

```rust
// src/adapter/gateway/keycloak_client.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::domain::model::User;
use crate::infra::config::OidcConfig;

pub struct KeycloakClient {
    base_url: String,
    realm: String,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
}

struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl KeycloakClient {
    pub fn new(cfg: &OidcConfig) -> Self {
        Self {
            base_url: extract_base_url(&cfg.discovery_url),
            realm: "k1s0".to_string(),
            client_id: cfg.client_id.clone(),
            client_secret: cfg.client_secret.clone(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap(),
            admin_token: Arc::new(RwLock::new(None)),
        }
    }

    #[instrument(skip(self), fields(service = "auth-server"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<User> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.base_url, self.realm, user_id
        );

        let resp = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("user not found");
        }

        let user: User = resp.error_for_status()?.json().await?;
        Ok(user)
    }

    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.base_url, self.realm);
        let resp = self.http_client.get(&url).send().await?;
        resp.error_for_status()?;
        Ok(())
    }

    async fn get_admin_token(&self) -> anyhow::Result<String> {
        // Client Credentials Grant でトークンを取得（キャッシュ付き）
        let cache = self.admin_token.read().await;
        if let Some(ref cached) = *cache {
            if chrono::Utc::now() < cached.expires_at {
                return Ok(cached.token.clone());
            }
        }
        drop(cache);

        let mut cache = self.admin_token.write().await;
        // POST /realms/k1s0/protocol/openid-connect/token
        // ...（実装省略）
        Ok(cache.as_ref().unwrap().token.clone())
    }
}
```

---

## config.yaml

[config設計.md](config設計.md) のスキーマを拡張した認証サーバー固有の設定。

```yaml
# config/config.yaml
app:
  name: "auth-server"
  version: "0.1.0"
  tier: "system"
  environment: "dev"

server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"

grpc:
  port: 50051
  max_recv_msg_size: 4194304  # 4MB

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""               # Vault パス: secret/data/k1s0/system/auth/database キー: password
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "auth-server.default"
  security_protocol: "PLAINTEXT"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: username
    password: ""             # Vault パス: secret/data/k1s0/system/kafka/sasl キー: password
  topics:
    publish:
      - "k1s0.system.auth.login.v1"
      - "k1s0.system.auth.token_validate.v1"
      - "k1s0.system.auth.permission_denied.v1"
    subscribe: []

observability:
  log:
    level: "debug"
    format: "json"
  trace:
    enabled: true
    endpoint: "jaeger.observability.svc.cluster.local:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"

auth:
  jwt:
    issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
    audience: "k1s0-api"
    public_key_path: ""
  oidc:
    discovery_url: "https://auth.k1s0.internal.example.com/realms/k1s0/.well-known/openid-configuration"
    client_id: "auth-server"
    client_secret: ""        # Vault パス: secret/data/k1s0/system/auth/oidc キー: client_secret
    redirect_uri: ""
    scopes: ["openid", "profile", "email"]
    jwks_uri: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"

# 認証サーバー固有設定
auth_server:
  # パーミッションキャッシュ
  permission_cache:
    ttl: "5m"                # ロール→パーミッション変換テーブルのキャッシュ TTL
    refresh_on_miss: true    # キャッシュミス時にバックグラウンドリフレッシュ
  # 監査ログ
  audit:
    kafka_enabled: true      # Kafka への非同期配信を有効化
    retention_days: 365      # DB 内の保持日数
  # Keycloak Admin API
  keycloak_admin:
    token_cache_ttl: "5m"    # Admin API トークンのキャッシュ TTL
```

### 設定の読み込み実装

#### Go

```go
// internal/infra/config/config.go
package config

import (
    "fmt"
    "os"
    "time"

    "github.com/go-playground/validator/v10"
    "gopkg.in/yaml.v3"
)

type Config struct {
    App           AppConfig           `yaml:"app"`
    Server        ServerConfig        `yaml:"server"`
    GRPC          GRPCConfig          `yaml:"grpc"`
    Database      DatabaseConfig      `yaml:"database"`
    Kafka         KafkaConfig         `yaml:"kafka"`
    Observability ObservabilityConfig `yaml:"observability"`
    Auth          AuthConfig          `yaml:"auth"`
    AuthServer    AuthServerConfig    `yaml:"auth_server"`
}

type ServerConfig struct {
    Host            string        `yaml:"host"`
    Port            int           `yaml:"port" validate:"required,min=1,max=65535"`
    ReadTimeout     time.Duration `yaml:"read_timeout"`
    WriteTimeout    time.Duration `yaml:"write_timeout"`
    ShutdownTimeout time.Duration `yaml:"shutdown_timeout"`
}

type AuthServerConfig struct {
    PermissionCache PermissionCacheConfig `yaml:"permission_cache"`
    Audit           AuditConfig           `yaml:"audit"`
    KeycloakAdmin   KeycloakAdminConfig   `yaml:"keycloak_admin"`
}

type PermissionCacheConfig struct {
    TTL            string `yaml:"ttl"`
    RefreshOnMiss  bool   `yaml:"refresh_on_miss"`
}

type AuditConfig struct {
    KafkaEnabled  bool `yaml:"kafka_enabled"`
    RetentionDays int  `yaml:"retention_days"`
}

type KeycloakAdminConfig struct {
    TokenCacheTTL string `yaml:"token_cache_ttl"`
}

func Load(path string) (*Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return nil, fmt.Errorf("failed to read config: %w", err)
    }
    var cfg Config
    if err := yaml.Unmarshal(data, &cfg); err != nil {
        return nil, fmt.Errorf("failed to parse config: %w", err)
    }
    return &cfg, nil
}

func (c *Config) Validate() error {
    validate := validator.New()
    return validate.Struct(c)
}
```

#### Rust

```rust
// src/infra/config/mod.rs
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: GrpcConfig,
    pub database: DatabaseConfig,
    pub kafka: KafkaConfig,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
    pub auth_server: AuthServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: String,
    pub write_timeout: String,
    pub shutdown_timeout: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthServerConfig {
    pub permission_cache: PermissionCacheConfig,
    pub audit: AuditServerConfig,
    pub keycloak_admin: KeycloakAdminConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PermissionCacheConfig {
    pub ttl: String,
    pub refresh_on_miss: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditServerConfig {
    pub kafka_enabled: bool,
    pub retention_days: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakAdminConfig {
    pub token_cache_ttl: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        if self.auth.jwt.issuer.is_empty() {
            anyhow::bail!("auth.jwt.issuer is required");
        }
        if self.auth.oidc.jwks_uri.is_empty() {
            anyhow::bail!("auth.oidc.jwks_uri is required");
        }
        Ok(())
    }
}
```

---

## データベースマイグレーション

監査ログテーブルは PostgreSQL に格納する。ユーザー情報は Keycloak が管理するため、認証サーバーの DB には監査ログのみを格納する。

```sql
-- migrations/001_create_audit_logs.sql

CREATE TABLE IF NOT EXISTS audit_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type  VARCHAR(100) NOT NULL,
    user_id     VARCHAR(255) NOT NULL,
    ip_address  VARCHAR(45)  NOT NULL,
    user_agent  TEXT         NOT NULL DEFAULT '',
    resource    VARCHAR(500) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    result      VARCHAR(20)  NOT NULL CHECK (result IN ('SUCCESS', 'FAILURE')),
    metadata    JSONB        NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- 検索用インデックス
CREATE INDEX idx_audit_logs_user_id ON audit_logs (user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs (event_type);
CREATE INDEX idx_audit_logs_recorded_at ON audit_logs (recorded_at DESC);
CREATE INDEX idx_audit_logs_result ON audit_logs (result);

-- 複合インデックス（ユーザー + 日時範囲の検索最適化）
CREATE INDEX idx_audit_logs_user_recorded ON audit_logs (user_id, recorded_at DESC);

-- パーティショニング（月単位）は運用フェーズで検討
COMMENT ON TABLE audit_logs IS '認証・認可の監査ログ。保持期間は 1 年間（可観測性設計.md 参照）';
```

---

## テスト方針

### レイヤー別テスト

| レイヤー | テスト種別 | Go | Rust |
| --- | --- | --- | --- |
| domain/service | 単体テスト | `testify/assert` | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `gomock` | `mockall` |
| adapter/handler | 統合テスト（HTTP/gRPC） | `httptest` + `testify` | `axum::test` + `tokio::test` |
| adapter/gateway | 統合テスト | `httptest` | `mockall` + `wiremock` |
| infra/persistence | 統合テスト（DB） | `testcontainers-go` | `testcontainers` |
| infra/auth | 単体テスト | `httptest` | `tokio::test` |

### Go テスト例

```go
// internal/usecase/validate_token_test.go
package usecase

import (
    "context"
    "testing"

    "github.com/stretchr/testify/assert"
    "go.uber.org/mock/gomock"
)

func TestValidateToken_Success(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()

    mockVerifier := auth.NewMockTokenVerifier(ctrl)
    mockVerifier.EXPECT().
        VerifyToken(gomock.Any(), "valid-token").
        Return(&auth.TokenClaims{
            Sub:    "user-uuid-1234",
            Issuer: "https://auth.k1s0.internal.example.com/realms/k1s0",
        }, nil)

    uc := NewValidateTokenUseCase(mockVerifier, config.JWTConfig{
        Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
        Audience: "k1s0-api",
    })

    claims, err := uc.Execute(context.Background(), "valid-token")
    assert.NoError(t, err)
    assert.Equal(t, "user-uuid-1234", claims.Sub)
}

func TestValidateToken_InvalidIssuer(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()

    mockVerifier := auth.NewMockTokenVerifier(ctrl)
    mockVerifier.EXPECT().
        VerifyToken(gomock.Any(), "token-wrong-issuer").
        Return(&auth.TokenClaims{
            Sub:    "user-uuid-1234",
            Issuer: "https://evil.example.com/realms/k1s0",
        }, nil)

    uc := NewValidateTokenUseCase(mockVerifier, config.JWTConfig{
        Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
        Audience: "k1s0-api",
    })

    _, err := uc.Execute(context.Background(), "token-wrong-issuer")
    assert.ErrorIs(t, err, ErrInvalidIssuer)
}
```

### Rust テスト例

```rust
// src/usecase/validate_token.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::auth::MockTokenVerifier;
    use crate::infra::config::JwtConfig;

    #[tokio::test]
    async fn test_validate_token_success() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .with(eq("valid-token"))
            .returning(|_| {
                Ok(TokenClaims {
                    sub: "user-uuid-1234".to_string(),
                    iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                    aud: "k1s0-api".to_string(),
                    ..Default::default()
                })
            });

        let uc = ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            JwtConfig {
                issuer: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                audience: "k1s0-api".to_string(),
                public_key_path: None,
            },
        );

        let claims = uc.execute("valid-token").await.unwrap();
        assert_eq!(claims.sub, "user-uuid-1234");
    }

    #[tokio::test]
    async fn test_validate_token_invalid_issuer() {
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| {
                Ok(TokenClaims {
                    sub: "user-uuid-1234".to_string(),
                    iss: "https://evil.example.com/realms/k1s0".to_string(),
                    aud: "k1s0-api".to_string(),
                    ..Default::default()
                })
            });

        let uc = ValidateTokenUseCase::new(
            Arc::new(mock_verifier),
            JwtConfig {
                issuer: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                audience: "k1s0-api".to_string(),
                public_key_path: None,
            },
        );

        let err = uc.execute("token-wrong-issuer").await.unwrap_err();
        assert!(matches!(err, AuthError::InvalidIssuer));
    }
}
```

### testcontainers による DB 統合テスト

#### Go

```go
// internal/infra/persistence/audit_log_repository_test.go
package persistence

import (
    "context"
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/testcontainers/testcontainers-go"
    "github.com/testcontainers/testcontainers-go/modules/postgres"
)

func TestAuditLogRepository_CreateAndSearch(t *testing.T) {
    ctx := context.Background()

    pgContainer, err := postgres.Run(ctx, "postgres:16-alpine",
        postgres.WithDatabase("k1s0_system_test"),
    )
    assert.NoError(t, err)
    defer pgContainer.Terminate(ctx)

    connStr, err := pgContainer.ConnectionString(ctx, "sslmode=disable")
    assert.NoError(t, err)

    db, err := NewDB(DatabaseConfig{/* connStr から設定 */})
    assert.NoError(t, err)
    defer db.Close()

    // マイグレーション実行
    runMigrations(db)

    repo := NewAuditLogRepository(db)

    // テスト実行
    log := &model.AuditLog{
        EventType: "LOGIN_SUCCESS",
        UserID:    "test-user",
        IPAddress: "127.0.0.1",
        Resource:  "/api/v1/auth/token",
        Action:    "POST",
        Result:    "SUCCESS",
    }
    err = repo.Create(ctx, log)
    assert.NoError(t, err)

    logs, count, err := repo.Search(ctx, AuditLogSearchParams{
        UserID:   "test-user",
        Page:     1,
        PageSize: 10,
    })
    assert.NoError(t, err)
    assert.Equal(t, 1, count)
    assert.Equal(t, "LOGIN_SUCCESS", logs[0].EventType)
}
```

#### Rust

```rust
// src/infra/persistence/audit_log_repository_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_audit_log_create_and_search() {
        let container = GenericImage::new("postgres", "16-alpine")
            .with_env_var("POSTGRES_DB", "k1s0_system_test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .start()
            .await
            .unwrap();

        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let pool = sqlx::PgPool::connect(
            &format!("postgresql://postgres:test@localhost:{}/k1s0_system_test", port),
        )
        .await
        .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let repo = AuditLogRepositoryImpl::new(pool);

        let log = AuditLog {
            id: uuid::Uuid::new_v4(),
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "test-user".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: String::new(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            metadata: std::collections::HashMap::new(),
            recorded_at: chrono::Utc::now(),
        };

        repo.create(&log).await.unwrap();

        let (logs, count) = repo
            .search(&AuditLogSearchParams {
                user_id: Some("test-user".to_string()),
                page: 1,
                page_size: 10,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(logs[0].event_type, "LOGIN_SUCCESS");
    }
}
```

---

## デプロイ

### Dockerfile

[Dockerイメージ戦略.md](Dockerイメージ戦略.md) のテンプレートに従う。

#### Go

```dockerfile
# ---- Build ----
FROM golang:1.23-bookworm AS build
WORKDIR /src
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /app ./cmd/

# ---- Runtime ----
FROM gcr.io/distroless/static-debian12
COPY --from=build /app /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

#### Rust

```dockerfile
# ---- Build ----
FROM rust:1.82-bookworm AS build
WORKDIR /src

# protoc のインストール（tonic-build に必要）
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12
COPY --from=build /src/target/release/auth-server /app
USER nonroot:nonroot
EXPOSE 8080 50051
ENTRYPOINT ["/app"]
```

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。認証サーバー固有の values は以下の通り。

```yaml
# values-auth.yaml
app:
  name: auth-server
  tier: system

image:
  repository: harbor.internal.example.com/k1s0/auth-server
  tag: "0.1.0"

service:
  ports:
    - name: http
      port: 80
      targetPort: 8080
    - name: grpc
      port: 50051
      targetPort: 50051

resources:
  requests:
    cpu: 100m
    memory: 128Mi
  limits:
    cpu: 500m
    memory: 256Mi

# Vault Agent Injector
podAnnotations:
  vault.hashicorp.com/agent-inject: "true"
  vault.hashicorp.com/role: "system"
  vault.hashicorp.com/agent-inject-secret-db-password: "secret/data/k1s0/system/auth/database"
  vault.hashicorp.com/agent-inject-secret-oidc: "secret/data/k1s0/system/auth/oidc"
  vault.hashicorp.com/agent-inject-secret-kafka-sasl: "secret/data/k1s0/system/kafka/sasl"

# ヘルスチェック
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /readyz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5

# ConfigMap マウント
configMap:
  name: auth-server-config
  mountPath: /etc/app/config.yaml
```

### Kong ルーティング

[認証認可設計.md](認証認可設計.md) の Kong ルーティング設計に従い、認証サーバーを Kong に登録する。

```yaml
services:
  - name: auth-v1
    url: http://auth-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: auth-v1-route
        paths:
          - /api/v1/auth
        strip_path: false
      - name: auth-v1-users-route
        paths:
          - /api/v1/users
        strip_path: false
      - name: auth-v1-audit-route
        paths:
          - /api/v1/audit
        strip_path: false
    plugins:
      - name: rate-limiting
        config:
          minute: 3000
          policy: redis
```

---

## 関連ドキュメント

- [認証認可設計.md](認証認可設計.md) -- Keycloak 設定・OAuth 2.0 フロー・RBAC 設計・Vault 戦略
- [API設計.md](API設計.md) -- REST / gRPC / GraphQL 設計・エラーレスポンス・バージョニング
- [config設計.md](config設計.md) -- config.yaml スキーマと環境別管理
- [可観測性設計.md](可観測性設計.md) -- OpenTelemetry・Prometheus・構造化ログ
- [メッセージング設計.md](メッセージング設計.md) -- Kafka トピック設計・監査イベント配信
- [Dockerイメージ戦略.md](Dockerイメージ戦略.md) -- マルチステージビルド・ベースイメージ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](コーディング規約.md) -- Linter・Formatter・命名規則
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャの詳細
- [APIゲートウェイ設計.md](APIゲートウェイ設計.md) -- Kong 構成管理
- [サービスメッシュ設計.md](サービスメッシュ設計.md) -- Istio 設計・mTLS
