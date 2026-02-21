# system-server 設計

system tier の認証サーバー設計を定義する。Keycloak 26.0 と連携し、JWT トークン検証・ユーザー情報管理・監査ログ記録を提供する。
Rust での実装を定義する。

## 概要

system tier の認証サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| ユーザー情報の取得・管理 API | Keycloak Admin API と連携してユーザー情報を取得・管理する |
| JWT トークンの検証・イントロスペクション | JWKS エンドポイントから公開鍵を取得し、JWT の署名検証・有効期限検証を行う |
| RBAC による認可制御 | Keycloak の `realm_access.roles` と `resource_access` に基づいてパーミッションを判定する |
| 監査ログの記録 | 認証・認可イベントを PostgreSQL に記録し、Kafka に非同期配信する |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC フレームワーク | tonic v0.12 |
| JWT ライブラリ | jsonwebtoken v9 |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| バリデーション | validator v0.18 |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
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
| POST | `/api/v1/auth/permissions/check` | パーミッション確認 | `sys_operator` 以上 |
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
  "detail": {
    "client_id": "react-spa",
    "grant_type": "authorization_code"
  },
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736"
}
```

**レスポンス（201 Created）**

```json
{
  "id": "audit-uuid-5678",
  "created_at": "2026-02-17T10:30:00.123Z"
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
      "detail": {
        "client_id": "react-spa"
      },
      "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
      "created_at": "2026-02-17T10:30:00.123Z"
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

#### POST /api/v1/auth/permissions/check

指定されたユーザーが特定リソースに対する操作権限を持つかを確認する。gRPC の `CheckPermission` に対応する REST エンドポイント。

**リクエスト**

```json
{
  "user_id": "user-uuid-1234",
  "permission": "write",
  "resource": "audit_logs",
  "roles": ["sys_operator", "user"]
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `user_id` | string | Yes | 検証対象のユーザー ID |
| `permission` | string | Yes | 操作種別（`read`, `write`, `delete`, `admin`） |
| `resource` | string | Yes | 対象リソース（`users`, `auth_config`, `audit_logs` 等） |
| `roles` | string[] | Yes | JWT Claims から取得したロール一覧 |

**レスポンス（200 OK - 許可）**

```json
{
  "allowed": true,
  "reason": ""
}
```

**レスポンス（200 OK - 拒否）**

```json
{
  "allowed": false,
  "reason": "role 'user' does not have 'write' permission on resource 'audit_logs'"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_AUTH_INVALID_REQUEST",
    "message": "permission フィールドは必須です",
    "request_id": "req_abc123def456",
    "details": []
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
| `resource_id` | string | 操作対象リソースの ID |
| `detail` | object (JSON) | 操作の詳細情報（変更前後の値、client_id、grant_type 等） |
| `trace_id` | string | OpenTelemetry トレース ID |
| `created_at` | timestamp | 記録日時 |

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

## 詳細設計ドキュメント

実装・デプロイの詳細は以下の分割ドキュメントを参照。

- [system-server-実装設計.md](system-server-実装設計.md) -- Rust 実装詳細（ドメイン・リポジトリ・ユースケース・ハンドラー・config.yaml）
- [system-server-デプロイ設計.md](system-server-デプロイ設計.md) -- DB マイグレーション・テスト・Dockerfile・Helm values

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
