# system-bff-proxy 設計

BFF（Backend for Frontend）パターンによる OAuth2/OIDC セッション管理プロキシ。ブラウザクライアントに Cookie ベースのセッション認証を提供し、上流 API へのリクエストに Bearer トークンを付与して転送する。Go 実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| OIDC 認可コードフロー（PKCE） | ブラウザ向けの安全なログインフローを実装。PKCE によって認可コード横取り攻撃を防止 |
| Cookie ベースセッション管理 | アクセストークン・リフレッシュトークンを Redis に保存し、ブラウザには HttpOnly Cookie のみ発行 |
| リバースプロキシ | セッション Cookie から Bearer トークンを復元して上流 API に転送 |
| 自動トークンリフレッシュ | アクセストークン期限切れ時にリフレッシュトークンで自動更新 |
| CSRF 保護 | セッション作成時に CSRF トークンを発行し、state 変更リクエストで検証 |
| スライディングセッション | リクエストごとにセッション TTL を延長するスライディングウィンドウをサポート |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | 技術 |
| --- | --- |
| 言語 | Go |
| HTTP フレームワーク | Gin |
| セッションストア | Redis（Sentinel 対応） |
| OIDC クライアント | 独自実装（PKCE + Discovery） |
| トレース | OpenTelemetry（OTLP gRPC） |
| メトリクス | Prometheus |

### 配置パス

配置: `regions/system/server/go/bff-proxy/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Go（Gin） |
| 役割 | ブラウザ向け BFF。トークンをサーバーサイドに保持し、クライアントには Cookie のみ渡す |
| セッション | Redis に保存。TTL 管理・スライディング延長をサポート |
| OIDC | PKCE S256 対応の認可コードフロー。Discovery エンドポイントから自動設定取得 |
| CSRF | セッション作成時に CSRF トークンを発行。`X-CSRF-Token` ヘッダーで検証 |
| プロキシ先 | `upstream.base_url` に設定した上流 API（デフォルト: graphql-gateway） |
| ポート | ホスト側 8094（内部 8080） |

---

## API 定義

### エンドポイント一覧

| Method | Path | セッション必須 | CSRF 必須 | 説明 |
| --- | --- | --- | --- | --- |
| GET | `/healthz` | No | No | ライブネスプローブ（常に 200 OK） |
| GET | `/readyz` | No | No | レディネスプローブ（Redis 疎通確認） |
| GET | `/metrics` | No | No | Prometheus メトリクス |
| GET | `/auth/login` | No | No | OIDC ログイン開始。IdP にリダイレクト |
| GET | `/auth/callback` | No | No | OIDC コールバック。セッション作成 |
| POST | `/auth/logout` | No | No | セッション削除・IdP ログアウト |
| ANY | `/api/*path` | Yes | Yes（設定時） | 上流 API へのリバースプロキシ |

#### GET /auth/login

PKCE コードチャレンジを生成し、OAuth state を Cookie に保存後、IdP の認可エンドポイントにリダイレクトする。

**レスポンス**: `302 Found` → IdP 認可 URL

#### GET /auth/callback

IdP からのコールバックを処理する。state 検証後、PKCE を用いてトークンを交換し、セッションを Redis に作成して Cookie を発行する。

**クエリパラメータ**

| パラメータ | 説明 |
| --- | --- |
| `code` | 認可コード |
| `state` | CSRF 保護用 state |

**成功レスポンス（200 OK）**

```json
{
  "status": "authenticated",
  "csrf_token": "hex-encoded-csrf-token"
}
```

**エラーレスポンス（400 / 500）**

```json
{
  "error": "BFF_AUTH_STATE_MISMATCH"
}
```

**エラーコード**

| コード | 説明 |
| --- | --- |
| `BFF_AUTH_STATE_MISSING` | state Cookie が存在しない |
| `BFF_AUTH_STATE_MISMATCH` | state パラメータが不一致 |
| `BFF_AUTH_CODE_MISSING` | 認可コードが存在しない |
| `BFF_AUTH_IDP_ERROR` | IdP がエラーを返した |
| `BFF_AUTH_TOKEN_EXCHANGE_FAILED` | トークン交換に失敗 |

#### POST /auth/logout

セッションを削除し、IdP のエンドセッションエンドポイントにリダイレクトする。id_token がセッションに存在する場合は `id_token_hint` を付与する。

**レスポンス**: `302 Found` → IdP ログアウト URL、または `200 OK`（ポストログアウト URI 未設定時）

#### ANY /api/*path

セッション Cookie を検証し、Redis からアクセストークンを取得して `Authorization: Bearer` ヘッダーに設定し、上流 API に転送する。

アクセストークン期限切れ時はリフレッシュトークンで自動更新する。

**セッション未検出時（401 Unauthorized）**

```json
{
  "error": "BFF_SESSION_MISSING",
  "message": "Session cookie not found"
}
```

**トークン期限切れ時（401 Unauthorized）**

```json
{
  "error": "BFF_PROXY_TOKEN_EXPIRED",
  "message": "Session expired, please re-authenticate"
}
```

---

## 認証フロー

```
ブラウザ           bff-proxy              Redis            IdP (Keycloak)    上流 API
   |                   |                    |                    |               |
   |-- GET /auth/login -->|                 |                    |               |
   |                   |-- PKCE 生成 ------>|                    |               |
   |<-- 302 IdP URL ---|                   |                    |               |
   |                   |                    |                    |               |
   |-- GET IdP /authorize ------------------------------------------>|          |
   |<-- 302 /auth/callback?code=... ---------------------------------|          |
   |                   |                    |                    |               |
   |-- GET /auth/callback -->|              |                    |               |
   |                   |-- トークン交換 ------------------------------>|         |
   |                   |<-- access_token / refresh_token ------------|          |
   |                   |-- セッション保存 -->|                    |               |
   |<-- 200 + Cookie --|                   |                    |               |
   |                   |                    |                    |               |
   |-- GET /api/... (Cookie) -->|           |                    |               |
   |                   |-- セッション取得 -->|                    |               |
   |                   |<-- AccessToken ----|                    |               |
   |                   |-- Authorization: Bearer -->|------------|-- → 上流 API  |
   |<-- API レスポンス --|                   |                    |               |
```

---

## アーキテクチャ

### モジュール構成

| パッケージ | 責務 |
| --- | --- |
| `cmd/` | エントリポイント・依存性注入・Gin ルーター構築 |
| `internal/handler/` | HTTP ハンドラー（auth, proxy, health） |
| `internal/oauth/` | OIDC Client（Discovery / PKCE / トークン交換・リフレッシュ） |
| `internal/session/` | Redis セッションストア |
| `internal/middleware/` | セッション検証・CSRF・Correlation ID・TraceID・Prometheus |
| `internal/config/` | YAML 設定ローダー |

### セッションデータ構造

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `access_token` | string | OAuth2 アクセストークン |
| `refresh_token` | string | OAuth2 リフレッシュトークン |
| `id_token` | string | OIDC ID トークン（ログアウト用） |
| `expires_at` | int64 | アクセストークン有効期限（Unix 秒） |
| `csrf_token` | string | CSRF 検証トークン |

---

## 設定フィールド

### auth

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `discovery_url` | string | OIDC Discovery URL（IdP のレルム URL） |
| `client_id` | string | OAuth2 クライアント ID |
| `client_secret` | string | OAuth2 クライアントシークレット（省略可） |
| `redirect_uri` | string | 認可コールバック URI |
| `post_logout_redirect_uri` | string | ログアウト後リダイレクト先 |
| `scopes` | []string | リクエストするスコープ（openid, profile, email 等） |

### session

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `redis.addr` | string | - | Redis アドレス（host:port） |
| `redis.master_name` | string | - | Sentinel 使用時のマスタ名 |
| `ttl` | string | `30m` | セッション TTL |
| `prefix` | string | `bff:session:` | Redis キープレフィックス |
| `sliding` | bool | `true` | スライディングウィンドウ有効化 |

### csrf

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `enabled` | bool | `true` | CSRF 検証有効化 |
| `header_name` | string | `X-CSRF-Token` | CSRF トークンヘッダー名 |

### upstream

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `base_url` | string | 上流 API のベース URL |
| `timeout` | string | リクエストタイムアウト（例: `30s`） |

---

## 設定ファイル例

```yaml
app:
  name: bff-proxy
  version: "0.1.0"
  tier: system
  environment: production

server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "10s"
  write_timeout: "30s"
  shutdown_timeout: "15s"

auth:
  discovery_url: "http://keycloak.k1s0-system.svc.cluster.local:8080/realms/k1s0"
  client_id: "k1s0-bff"
  client_secret: ""
  redirect_uri: "https://app.example.com/auth/callback"
  post_logout_redirect_uri: "https://app.example.com"
  scopes:
    - openid
    - profile
    - email

session:
  redis:
    addr: "redis-sentinel.k1s0-system.svc.cluster.local:26379"
    master_name: "mymaster"
    password: ""
    db: 0
  ttl: "30m"
  prefix: "bff:session:"
  sliding: true

csrf:
  enabled: true
  header_name: "X-CSRF-Token"

upstream:
  base_url: "http://graphql-gateway.k1s0-system.svc.cluster.local:8080"
  timeout: "30s"
```

---

## デプロイ

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| HTTP | 8080 | 全エンドポイント |

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- OIDC / JWT 設計方針
- [system-graphql-gateway/server.md](../graphql-gateway/server.md) -- 上流 GraphQL ゲートウェイ
- [system-auth/server.md](../auth/server.md) -- 認証サーバー（IdP バックエンド）
