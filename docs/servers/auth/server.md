# system-auth-server 設計

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
| POST | `/api/v1/api-keys` | API キー作成 | `api_keys` | `write` |
| GET | `/api/v1/api-keys` | API キー一覧取得 | `api_keys` | `read` |
| GET | `/api/v1/api-keys/:id` | API キー取得 | `api_keys` | `read` |
| DELETE | `/api/v1/api-keys/:id/revoke` | API キー無効化 | `api_keys` | `write` |

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
| `users[].first_name` | string | 名 |
| `users[].last_name` | string | 姓 |
| `users[].enabled` | bool | 有効フラグ |
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
| `permission` | string | Yes | 必要な権限（`read` / `write` / `delete` / `admin`） |
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

#### GET /api/v1/navigation

クライアント向けナビゲーション設定を返却する。

**レスポンスフィールド（200 OK）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `version` | int | ナビゲーション設定バージョン |
| `guards[]` | object[] | ナビゲーションガード定義 |
| `guards[].id` | string | ガード ID |
| `guards[].type` | string | ガード種別 |
| `guards[].redirect_to` | string | リダイレクト先 |
| `guards[].roles` | string[] | 必要ロール |
| `routes[]` | object[] | ルート定義 |
| `routes[].id` | string | ルート ID |
| `routes[].path` | string | パス |
| `routes[].component_id` | string? | コンポーネント ID |
| `routes[].guards` | string[] | 適用ガード |
| `routes[].children` | object[] | 子ルート |

**エラーレスポンス（500）**: `SYS_NAV_CONFIG_READ_ERROR` / `SYS_NAV_CONFIG_PARSE_ERROR`

#### POST /api/v1/api-keys

API キーを作成する。作成時のみ `raw_key`（平文キー）が返却される。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `tenant_id` | string | Yes | テナント ID |
| `name` | string | Yes | API キー名 |
| `scopes` | string[] | Yes | API キーのスコープ |
| `expires_at` | string | No | 有効期限（RFC 3339） |

**レスポンスフィールド（201 Created）**

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | API キー ID（UUID） |
| `name` | string | API キー名 |
| `prefix` | string | API キーのプレフィックス（表示用） |
| `raw_key` | string | API キー平文（作成時のみ返却） |
| `scopes` | string[] | スコープ |
| `expires_at` | string? | 有効期限 |
| `created_at` | string | 作成日時（RFC 3339） |

#### GET /api/v1/api-keys

テナントに紐づく API キー一覧を取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `tenant_id` | string | Yes | テナント ID |

**レスポンスフィールド（200 OK）**

`ApiKeySummary[]` の配列:

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | API キー ID（UUID） |
| `name` | string | API キー名 |
| `prefix` | string | API キーのプレフィックス |
| `scopes` | string[] | スコープ |
| `expires_at` | string? | 有効期限 |
| `revoked` | bool | 無効化済みフラグ |
| `created_at` | string | 作成日時（RFC 3339） |

#### GET /api/v1/api-keys/:id

指定された ID の API キー情報を取得する。レスポンスフィールドは `ApiKeySummary` と同一。

**エラーレスポンス（404）**: `SYS_AUTH_API_KEY_NOT_FOUND`

#### DELETE /api/v1/api-keys/:id/revoke

指定された API キーを無効化する。

**レスポンス**: 204 No Content

**エラーレスポンス（404）**: `SYS_AUTH_API_KEY_NOT_FOUND`

> **注**: API Key 管理は REST API のみ提供する。gRPC（Proto）での API Key RPC は定義しない。

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

> **注**: REST API の `/api/v1/auth/permissions/check` はロールベースの純粋な権限チェックのため `user_id` フィールドを持たない。gRPC の `CheckPermission` はサービス間通信でユーザー特定が必要なため `user_id` を含む。

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
  string first_name = 4;
  string last_name = 5;
  bool enabled = 6;
  bool email_verified = 7;
  k1s0.system.common.v1.Timestamp created_at = 8;
  map<string, StringList> attributes = 9;
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

| ロール | read | write | delete | admin |
| --- | --- | --- | --- | --- |
| `sys_admin` | Yes | Yes | Yes | Yes |
| `sys_operator` | Yes | Yes | No | No |
| `sys_auditor` | Yes | No | No | No |

`sys_admin` は全権限を持つ。`sys_operator` は read/write、`sys_auditor` は read のみ。

### RBAC エンドポイント設定

| エンドポイントグループ | リソース | 必要権限 |
| --- | --- | --- |
| `/api/v1/users/**` | `users` | `read` |
| `/api/v1/auth/permissions/check` | `auth_config` | `read` |
| `GET /api/v1/audit/logs` | `audit_logs` | `read` |
| `POST /api/v1/audit/logs` | `audit_logs` | `write` |
| `POST /api/v1/api-keys` | `api_keys` | `write` |
| `GET /api/v1/api-keys/**` | `api_keys` | `read` |
| `DELETE /api/v1/api-keys/:id/revoke` | `api_keys` | `write` |

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
| `cache_ttl_secs` | int | `600` | JWKS キャッシュ TTL（秒）。[JWT設計.md](../../architecture/auth/JWT設計.md) の JWKS キャッシュ TTL 10 分と整合 |

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

## API リクエスト・レスポンス例

### POST /api/v1/auth/token/validate

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

### POST /api/v1/auth/token/introspect

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

### GET /api/v1/users

**レスポンス（200 OK）**

```json
{
  "users": [
    {
      "id": "user-uuid-1234",
      "username": "taro.yamada",
      "email": "taro.yamada@example.com",
      "first_name": "Taro",
      "last_name": "Yamada",
      "enabled": true,
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

### GET /api/v1/users/:id

**レスポンス（200 OK）**

```json
{
  "id": "user-uuid-1234",
  "username": "taro.yamada",
  "email": "taro.yamada@example.com",
  "first_name": "Taro",
  "last_name": "Yamada",
  "enabled": true,
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

### GET /api/v1/users/:id/roles

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

### POST /api/v1/auth/permissions/check

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

### GET /api/v1/audit/logs

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

### POST /api/v1/audit/logs

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

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2026-02-20T10:30:00Z"
}
```

### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

### GET /readyz

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

### パーミッションキャッシュ

moka を使用したインメモリキャッシュで RBAC 判定結果をキャッシュする。

- TTL: `permission_cache.ttl_secs`（デフォルト 300 秒）
- キャッシュミス時の自動リフレッシュ: `permission_cache.refresh_on_miss`

---

## 依存関係図

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

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-config-server.md](../config/server.md) -- 設定管理サーバー設計（同 tier の参考実装）
