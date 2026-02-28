# system-auth-server 設計

> **ガイド**: 設計背景・実装例は [server.guide.md](./server.guide.md) を参照。

system tier の認証・認可・監査基盤サーバー。REST/gRPC でトークン検証・ユーザー管理・権限チェック・監査ログ機能を提供する。Rust 実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| JWT トークン検証 | Keycloak 発行の JWT を JWKS で検証し、Claims を返却する |
| トークンイントロスペクション | RFC 7662 準拠のトークン情報取得 |
| ユーザー管理 API | Keycloak 連携によるユーザー情報・ロール取得 |
| RBAC 権限チェック | ロールベースのアクセス制御判定 |
| 監査ログ記録・検索 | 認証イベントの記録と検索 |
| ナビゲーション設定 | クライアント向けナビゲーション構成の提供 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| JWT 検証 | jsonwebtoken + JWKS |
| API ドキュメント | utoipa + utoipa-swagger-ui |

### 配置パス

配置: `regions/system/server/rust/auth/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_AUTH_` とする。

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

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `token` | string | Yes | 検証対象の JWT トークン |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `valid` | bool | トークンの有効性 |
| `claims.sub` | string | ユーザー ID |
| `claims.iss` | string | 発行者 URL |
| `claims.aud` | string | audience |
| `claims.exp` | int64 | 有効期限（Unix timestamp） |
| `claims.iat` | int64 | 発行日時（Unix timestamp） |
| `claims.jti` | string | トークン ID |
| `claims.preferred_username` | string | ユーザー名 |
| `claims.email` | string | メールアドレス |
| `claims.scope` | string | スコープ |
| `claims.realm_access.roles` | string[] | ロールリスト |

**エラーレスポンス（401）**: `SYS_AUTH_TOKEN_INVALID`

#### POST /api/v1/auth/token/introspect

RFC 7662 準拠のトークンイントロスペクション。トークンが無効でも 200 を返し、`active: false` で応答する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `token` | string | Yes | 検証対象のトークン |
| `token_type_hint` | string | No | トークン種別ヒント（`access_token`） |

**レスポンスフィールド（200 OK - アクティブ）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `active` | bool | トークンの有効性 |
| `sub` | string | ユーザー ID |
| `client_id` | string | クライアント ID |
| `username` | string | ユーザー名 |
| `token_type` | string | トークン種別（`Bearer`） |
| `exp` | int64 | 有効期限 |
| `iat` | int64 | 発行日時 |
| `scope` | string | スコープ |
| `realm_access.roles` | string[] | ロールリスト |

**レスポンス（200 OK - 非アクティブ）**: `{ "active": false }`

#### GET /api/v1/users

ユーザー一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1ページあたりの件数 |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `users[].id` | string | ユーザー ID |
| `users[].username` | string | ユーザー名 |
| `users[].email` | string | メールアドレス |
| `users[].display_name` | string | 表示名 |
| `users[].status` | string | ステータス（`active` / `suspended` / `deleted`） |
| `users[].email_verified` | bool | メール検証済みフラグ |
| `users[].created_at` | string | 作成日時（RFC 3339） |
| `users[].attributes` | object | 追加属性 |
| `pagination.total_count` | int | 総件数 |
| `pagination.page` | int | 現在ページ |
| `pagination.page_size` | int | ページサイズ |
| `pagination.has_next` | bool | 次ページ有無 |

#### GET /api/v1/users/:id

指定された ID のユーザー情報を取得する。レスポンスフィールドは GET /api/v1/users の `users[]` と同一。

**エラーレスポンス（404）**: `SYS_AUTH_USER_NOT_FOUND`

#### GET /api/v1/users/:id/roles

指定されたユーザーの Realm ロールおよびクライアントロールを取得する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `user_id` | string | ユーザー ID |
| `realm_roles[].id` | string | ロール ID |
| `realm_roles[].name` | string | ロール名 |
| `realm_roles[].description` | string | ロール説明 |
| `client_roles` | map\<string, Role[]\> | クライアント別ロールマップ |

#### POST /api/v1/auth/permissions/check

指定されたロール群が、特定リソースに対する権限を持つか判定する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `roles` | string[] | Yes | 判定対象のロールリスト |
| `permission` | string | Yes | 必要な権限（`read` / `write` / `admin`） |
| `resource` | string | Yes | 対象リソース |

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `allowed` | bool | 許可フラグ |
| `reason` | string | 拒否時の理由 |

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

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `logs[].id` | string | 監査ログ ID（UUID） |
| `logs[].event_type` | string | イベント種別 |
| `logs[].user_id` | string | ユーザー ID |
| `logs[].ip_address` | string | リクエスト元 IP |
| `logs[].user_agent` | string | User-Agent |
| `logs[].resource` | string | 対象リソースパス |
| `logs[].resource_id` | string? | 対象リソース ID |
| `logs[].action` | string | HTTP メソッドまたは操作種別 |
| `logs[].result` | string | 結果（`SUCCESS` / `FAILURE`） |
| `logs[].detail` | object | 付加情報 |
| `logs[].trace_id` | string | 分散トレーシング ID |
| `logs[].created_at` | string | 作成日時（RFC 3339） |
| `pagination` | object | ページネーション（users と同構造） |

#### POST /api/v1/audit/logs

監査ログエントリを記録する。

**リクエストフィールド**

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

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | 監査ログ ID（UUID） |
| `created_at` | string | 作成日時（RFC 3339） |

#### GET /healthz

**レスポンス**: `{ "status": "ok" }`（200 OK）

#### GET /readyz

PostgreSQL と Keycloak への接続を確認する。

**レスポンスフィールド（200 OK / 503）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `status` | string | `ready` / `not ready` |
| `checks.database` | string | DB 接続状態 |
| `checks.keycloak` | string | Keycloak 接続状態 |

### gRPC サービス定義

proto ファイルは [API設計.md](../../architecture/api/API設計.md) D-009 の命名規則に従い、以下に配置する。

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

## RBAC

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

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の4レイヤー構成に従う。

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

- [system-server-deploy.md](../_common/deploy.md)

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST (HTTP) | 8080 | REST API + Swagger UI |
| gRPC | 50051 | gRPC サービス |

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-config-server.md](../config/server.md) -- 設定管理サーバー設計（同 tier の参考実装）
