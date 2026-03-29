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
| OIDC クライアント | 独自実装（PKCE + Discovery + JWKS署名検証）|
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
| OIDC | PKCE S256 対応の認可コードフロー。Discovery エンドポイントから自動設定取得。IDトークンは `jwks_uri` から取得したJWKS公開鍵で署名検証（`coreos/go-oidc/v3`）|
| CSRF | セッション作成時に CSRF トークンを発行。`X-CSRF-Token` ヘッダーで検証 |
| プロキシ先 | `upstream.base_url` に設定した上流 API（デフォルト: auth-server） |
| ポート | ホスト側 `${BFF_PROXY_HOST_PORT:-8082}`（内部 8080）— HIGH-14 監査対応: docker-compose.yaml の実際のマッピングに合わせて修正 |

---

## API 定義

### エンドポイント一覧

| Method | Path | セッション必須 | CSRF 必須 | 説明 |
| --- | --- | --- | --- | --- |
| GET | `/healthz` | No | No | ライブネスプローブ（常に 200 OK） |
| GET | `/readyz` | No | No | レディネスプローブ（Redis 疎通確認 + OIDC Discovery 完了確認） |
| GET | `/metrics` | No | No | Prometheus メトリクス |
| GET | `/auth/login` | No | No | OIDC ログイン開始。IdP にリダイレクト |
| GET | `/auth/callback` | No | No | OIDC コールバック。セッション作成 |
| GET | `/auth/session` | No | No | セッション確認。ユーザー情報返却 |
| GET | `/auth/exchange` | No | No | ワンタイム交換コード検証。モバイル向けセッション発行 |
| POST | `/auth/logout` | No | No | セッション削除・IdP ログアウト |
| ANY | `/api/*path` | Yes | Yes（設定時） | 上流 API へのリバースプロキシ |

#### GET /auth/login

PKCE コードチャレンジを生成し、OAuth state を Cookie に保存後、IdP の認可エンドポイントにリダイレクトする。

**クエリパラメータ（オプション）**

| パラメータ | 説明 |
| --- | --- |
| `redirect_to` | モバイルクライアント向け。認証完了後のリダイレクト先（カスタムスキームのみ許可、例: `k1s0://auth/callback`）。http/https スキームはオープンリダイレクト防止のため拒否される |

**レスポンス**: `302 Found` → IdP 認可 URL

#### GET /auth/callback

IdP からのコールバックを処理する。state 検証後、PKCE を用いてトークンを交換し、セッションを Redis に作成して Cookie を発行する。

**クエリパラメータ**

| パラメータ | 説明 |
| --- | --- |
| `code` | 認可コード |
| `state` | CSRF 保護用 state |

**成功レスポンス（200 OK）**（ブラウザフロー）

```json
{
  "status": "authenticated",
  "csrf_token": "hex-encoded-csrf-token"
}
```

**成功レスポンス（302 Found）**（モバイルフロー: `redirect_to` が Login で指定された場合）

ワンタイム交換コードを発行し、指定されたカスタムスキーム URL にリダイレクトする。

`302 Found` → `k1s0://auth/callback?code=<exchange-code>`

交換コードは60秒間有効で、`GET /auth/exchange` で検証する。

**エラーレスポンス（400 / 500）**

```json
{
  "error": "BFF_AUTH_STATE_MISMATCH"
}
```

**エラーコード**

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `BFF_AUTH_STATE_MISSING` | 400 | state Cookie が存在しない |
| `BFF_AUTH_STATE_MISMATCH` | 400 | state パラメータが不一致 |
| `BFF_AUTH_CODE_MISSING` | 400 | 認可コードが存在しない |
| `BFF_AUTH_IDP_ERROR` | 400 | IdP がエラーを返した |
| `BFF_AUTH_TOKEN_EXCHANGE_FAILED` | 500 | トークン交換に失敗 |
| `BFF_AUTH_PKCE_ERROR` | 500 | PKCE 生成失敗 |
| `BFF_AUTH_STATE_ERROR` | 500 | State 生成失敗 |
| `BFF_AUTH_URL_ERROR` | 500 | Auth URL 構築失敗 |
| `BFF_AUTH_VERIFIER_MISSING` | 400 | PKCE verifier cookie 欠落 |
| `BFF_AUTH_ID_TOKEN_INVALID` | 401 | id_token が無効または期限切れ |
| `BFF_AUTH_CSRF_ERROR` | 500 | CSRF トークン生成失敗 |
| `BFF_AUTH_SESSION_CREATE_FAILED` | 500 | セッション作成失敗 |
| `BFF_PROXY_NO_SESSION` | 401 | プロキシリクエストにセッション欠落 |
| `BFF_PROXY_TOKEN_EXPIRED` | 401 | アクセストークン期限切れ |
| `BFF_CSRF_NO_SESSION` | 403 | CSRF チェック時セッション欠落 |
| `BFF_CSRF_INVALID_SESSION` | 403 | CSRF チェック時無効セッション |
| `BFF_CSRF_MISMATCH` | 403 | CSRF トークン不一致 |
| `BFF_SESSION_MISSING` | 401 | セッション Cookie 欠落 |
| `BFF_SESSION_INVALID` | 401 | セッション無効（store に存在しない） |
| `BFF_SESSION_EXPIRED` | 401 | アクセストークンの `ExpiresAt` が現在時刻を過ぎており、かつ refresh token が存在しない場合（refresh token がある場合は handler で silent refresh を試みる） |
| `BFF_AUTH_SESSION_NOT_FOUND` | 401 | セッション確認時にセッション欠落 |
| `BFF_AUTH_SESSION_EXPIRED` | 401 | セッション確認時に有効期限切れ |
| `BFF_AUTH_SESSION_ERROR` | 500 | セッション確認時の取得エラー |
| `BFF_AUTH_EXCHANGE_CREATE_FAILED` | 500 | 交換コード作成失敗 |
| `BFF_AUTH_EXCHANGE_CODE_MISSING` | 400 | 交換コードパラメータ欠落 |
| `BFF_AUTH_EXCHANGE_CODE_INVALID` | 401 | 交換コード無効または期限切れ |
| `BFF_AUTH_EXCHANGE_ERROR` | 500 | 交換コード検証エラー |

#### GET /auth/session

セッション Cookie を検証し、現在のユーザー情報を返す。SPA のセッション確認に使用する。

**成功レスポンス（200 OK）**

```json
{
  "id": "user-subject",
  "authenticated": true,
  "csrf_token": "hex-encoded-csrf-token",
  "roles": ["admin", "user"]
}
```

`roles` フィールドには Keycloak の realm roles が含まれる。フロントエンドはこのフィールドを使用して `/admin` 等の権限が必要なルートへのアクセス可否を判定する。ロールが存在しない場合は空配列 `[]` を返す。roles は `oauth.ExtractRolesFromAccessToken` でアクセストークンの `realm_access.roles` から抽出し、`SessionData.Roles` に保存される。

**エラーレスポンス（401 / 500）**

```json
{
  "error": "BFF_AUTH_SESSION_NOT_FOUND"
}
```

#### GET /auth/exchange

モバイルクライアント向け。ワンタイム交換コードを検証し、セッション Cookie を発行する。

モバイル OAuth フローでは、ブラウザとアプリの Cookie が共有されないため、`/auth/callback` で発行された交換コードをこのエンドポイントで検証し、Dio（モバイル HTTP クライアント）にセッション Cookie を設定する。

**クエリパラメータ**

| パラメータ | 説明 |
| --- | --- |
| `code` | ワンタイム交換コード（`/auth/callback` のモバイルリダイレクトで発行、有効期限60秒） |

**成功レスポンス（200 OK）**

```json
{
  "id": "user-subject",
  "authenticated": true,
  "csrf_token": "hex-encoded-csrf-token"
}
```

**エラーレスポンス（400 / 401 / 500）**

```json
{
  "error": "BFF_AUTH_EXCHANGE_CODE_INVALID"
}
```

#### POST /auth/logout

セッションを削除し、IdP のエンドセッションエンドポイントにリダイレクトする。id_token がセッションに存在する場合は `id_token_hint` を付与する。

ログアウト後の遷移先は次の優先順位で決定する。

1. IdP の `end_session_endpoint`（`id_token_hint` + `post_logout_redirect_uri` を付与できた場合）
2. `post_logout_redirect_uri`（IdP ログアウト URL を構築できない場合のフォールバック）
3. JSON レスポンス `{ "status": "logged_out" }`（リダイレクト先未設定時）

**レスポンス**: `302 Found` → IdP ログアウト URL（`post_logout_redirect_uri` 設定時）、または `200 OK`（未設定時）

**成功レスポンス（200 OK）**

```json
{
  "status": "logged_out"
}
```

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

### ブラウザフロー（React SPA）

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
   |-- GET /auth/session (Cookie) -->|      |                    |               |
   |                   |-- セッション取得 -->|                    |               |
   |<-- 200 + ユーザー情報 + CSRF ---|      |                    |               |
   |                   |                    |                    |               |
   |-- GET /api/... (Cookie + CSRF) -->|    |                    |               |
   |                   |-- セッション取得 -->|                    |               |
   |                   |<-- AccessToken ----|                    |               |
   |                   |-- Authorization: Bearer -->|------------|-- → 上流 API  |
   |<-- API レスポンス --|                   |                    |               |
```

### モバイルフロー（Flutter）

```
Flutter App        In-App Browser     bff-proxy        Redis         IdP (Keycloak)
   |                   |                 |                |                |
   |-- open login URL ─>|                |                |                |
   |                   |-- GET /auth/login?redirect_to=k1s0://... ──>|     |
   |                   |                 |-- PKCE 生成 -->|                |
   |                   |<── 302 IdP URL ─|                |                |
   |                   |                 |                |                |
   |                   |-- GET IdP /authorize ─────────────────────>|      |
   |                   |<── 302 /auth/callback?code=... ───────────|      |
   |                   |                 |                |                |
   |                   |-- GET /auth/callback ──>|        |                |
   |                   |                 |-- トークン交換 ─────────>|      |
   |                   |                 |-- セッション保存 ─>|     |      |
   |                   |                 |-- 交換コード保存 ─>|     |      |
   |                   |<── 302 k1s0://auth/callback?code=EXCHANGE_CODE   |
   |                   |                 |                |                |
   |<── callback URL ──|                 |                |                |
   |                   |                 |                |                |
   |── GET /auth/exchange?code=... ─────>|                |                |
   |                                     |── 交換コード検証 ─>|            |
   |                                     |── 交換コード削除 ─>|            |
   |<── 200 + Set-Cookie + ユーザー情報 ─|                |                |
   |                                     |                |                |
   |── GET /auth/session (Cookie) ──────>|                |                |
   |<── 200 + ユーザー情報 + CSRF ───────|                |                |
```

---

## アーキテクチャ

### モジュール構成

| パッケージ | 責務 |
| --- | --- |
| `cmd/server/` | エントリポイント・依存性注入・Gin ルーター構築 |
| `internal/handler/` | HTTP ハンドラー（auth, proxy, health）。HTTP 変換のみを担当し、ビジネスロジックは usecase に委譲する |
| `internal/usecase/` | ビジネスロジック（認証フロー・プロキシ用セッション検証・トークンリフレッシュ） |
| `internal/port/` | usecase 層が依存する外部サービスのポートインターフェース定義（OAuthClient, SessionStore） |
| `internal/oauth/` | OIDC Client（Discovery / PKCE / トークン交換・リフレッシュ / JWKS署名検証） |
| `internal/session/` | Redis セッションストア |
| `internal/upstream/` | 上流 API へのリバースプロキシ |
| `internal/middleware/` | セッション検証・CSRF・Correlation ID・TraceID・Prometheus |
| `internal/config/` | YAML 設定ローダー |

### セッションデータ構造

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `access_token` | string | OAuth2 アクセストークン |
| `refresh_token` | string | OAuth2 リフレッシュトークン |
| `id_token` | string | OIDC ID トークン（ログアウト用） |
| `sub` | string | OIDC subject |
| `expires_at` | int64 | アクセストークン有効期限（Unix 秒） |
| `created_at` | int64 | セッション作成日時（Unix 秒） |
| `csrf_token` | string | CSRF 検証トークン |

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8080` | HTTP ポート |
| `read_timeout` | string | `10s` | リクエスト受信タイムアウト |
| `write_timeout` | string | `30s` | レスポンス送信タイムアウト |
| `shutdown_timeout` | string | `15s` | Graceful shutdown タイムアウト |

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
| `redis.password` | string | `""` | Redis 認証パスワード |
| `redis.db` | int | `0` | Redis DB 番号 |
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

### observability

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `log.level` | string | ログレベル（`debug`/`info`/`warn`/`error`） |
| `log.format` | string | ログフォーマット（`json` / `text`） |
| `trace.endpoint` | string | OTLP エクスポータ先 |
| `trace.enabled` | bool | トレース出力有効化 |
| `trace.sample_rate` | float | サンプリングレート（0.0 - 1.0） |
| `metrics.enabled` | bool | Prometheus メトリクス出力の有効化 |
| `metrics.path` | string | メトリクス公開パス |

> `log.format` / `metrics.path` / `trace.enabled` / `trace.sample_rate` は実装済みで、起動時設定として反映される。
> 起動時に `go-playground/validator/v10` の `validate.Struct` で全フィールドのバリデーション（`required` / `url` / `min` / `max` タグ）を実行し、不正値があれば即時終了する。

---

## 設定ファイル例
> ※ dev環境では省略可能なセクションがあります。


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
  base_url: "http://auth-server.k1s0-system.svc.cluster.local:8080"
  timeout: "30s"

observability:
  log:
    level: "info"
  trace:
    endpoint: "http://otel-collector:4317"
  metrics:
    enabled: true
```

---

## デプロイ

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| HTTP | 8080 | 全エンドポイント |

### HIGH-8 監査対応: SESSION_ENCRYPTION_KEY のクラスター間統一

複数の BFF-Proxy インスタンスを水平スケールする場合、`SESSION_ENCRYPTION_KEY` が全インスタンスで同一でなければ、別インスタンスで暗号化されたセッションが復号できず認証失敗が発生する。

**運用手順:**
1. Vault の `secret/k1s0/system/bff-proxy` に canonical キーを保存する
2. Kubernetes では SecretProviderClass 経由でキーを環境変数 `SESSION_ENCRYPTION_KEY` に注入する
3. ローカル開発では `.env.dev` に 32 バイトのランダムキー（`openssl rand -base64 32`）を設定する
4. 将来的な実装: 起動時に Vault から canonical キーを取得して一致確認するバリデーション

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../../_common/deploy.md#共通関連ドキュメント) を参照。

- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- OIDC / JWT 設計方針
- [system-graphql-gateway/server.md](../graphql-gateway/server.md) -- 上流 GraphQL ゲートウェイ
- [system-auth/server.md](../auth/server.md) -- 認証サーバー（IdP バックエンド）



## Doc Sync (2026-03-03)

### Message/Field Corrections
- 401 系エラーコードとして BFF_AUTH_ID_TOKEN_INVALID を使用する。
- trace 設定キーは trace.endpoint。併せて trace.enabled, trace.sample_rate を扱う。
- 監視/ログ設定として log.format, metrics.path を使用する。
- Logout (POST /auth/logout) は 200 OK 時に JSON ボディを返す。

## Doc Sync (2026-03-21)

### F03: SessionMiddleware の silent refresh 対応

`SessionMiddleware` はアクセストークンが期限切れの場合、refresh token の有無によって挙動が異なる。

| 条件 | 挙動 |
| --- | --- |
| expired + refresh token なし | 即座に `401 BFF_SESSION_EXPIRED` を返す |
| expired + refresh token あり | `session_needs_refresh = true` フラグを gin context に設定し、handler に処理を委ねる |

handler 側（`proxy_handler.go`）の silent refresh ロジックが `session_needs_refresh` フラグを参照してリフレッシュを試みる。

### F09: 環境名設定（dev / development / local）

`app.environment` の値として `dev`・`development`・`local` はすべて**開発環境**として扱われる。
`IsDevEnvironment()` ヘルパー（`internal/config/config.go`）で統一的に判定するため、
設定値と比較文字列の不一致によるバグが発生しない。

- `dev`（config.yaml デフォルト）/ `development` / `local` → 開発環境
- `prod` / `production` → 本番環境（`ALLOW_REDIS_SKIP` 無視、secure Cookie 有効）

JWKS（`oidc.NewRemoteKeySet`）はアプリケーションレベルの ctx を使用するため、
シャットダウン時に JWKS バックグラウンドフェッチがキャンセルされる。
---

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。

---

## ルーティング設計

<!-- M-06指摘事項: bff-proxyのルーティング設計と上流API設定方針を明文化 -->

bff-proxy は `/api/*path` のキャッチオールプロキシとして機能し、
Kong API Gateway を経由して各マイクロサービスにルーティングする。

個別のアップストリームへの直接ルーティングは実装されておらず、
Kong の route 設定でサービスへのディスパッチを行う。

本番環境では bff-proxy が Kong を upstream として設定することが必要。
M-06指摘事項として、`config.docker.yaml` の upstream base_url を Kong のアドレスに設定すること。

### upstream.base_url の設定指針

| 環境 | upstream.base_url の値 | 備考 |
| --- | --- | --- |
| ローカル開発（docker compose） | `http://auth-server.k1s0-system.svc.cluster.local:8080` | 直接サービスを指定（Kong なし） |
| 本番 / ステージング | `http://kong.k1s0-system.svc.cluster.local:8000` | Kong を経由して各サービスにディスパッチ |

### ルーティング方式の注意点

- bff-proxy 自体はルーティングロジックを持たず、`upstream.base_url` で指定した単一エンドポイントにプロキシする
- Kong がパスベースのルーティングを担当し、`/api/auth/*`, `/api/tenant/*` 等のプレフィックスでサービスへ転送する
- bff-proxy は認証（セッション検証・トークン付与）のみを担当し、ルーティングの責務を持たない

---

## レイヤー構成

### 設計方針

A-1 対応として Usecase 層を新設し、クリーンアーキテクチャの「handler は usecase を呼ぶだけ」という原則を採用した。
handler が直接 OAuth クライアントとセッションストアを操作していた処理を usecase に抽出することで、
ビジネスロジックのテスト容易性と handler の責務分離を実現する。

| 層 | パッケージ | 責務 |
| --- | --- | --- |
| Adapter（HTTP） | `internal/handler/` | HTTP リクエスト/レスポンス変換・Cookie 操作・リダイレクト |
| Usecase | `internal/usecase/` | 認証フロー・セッション検証・トークンリフレッシュのビジネスロジック |
| Port（Interface） | `internal/port/` | usecase が依存する外部サービスのインターフェース定義 |
| Infrastructure | `internal/oauth/`, `internal/session/` | OIDC クライアント・Redis セッションストアの具体実装 |

### 現在のモジュール構成

```
bff-proxy/
├── cmd/server/          # エントリポイント・DI・ルーター構築
├── internal/
│   ├── handler/         # HTTP ハンドラー（auth, proxy, health）
│   ├── usecase/         # ビジネスロジック（auth_usecase, proxy_usecase）
│   ├── port/            # ポートインターフェース（OAuthClient, SessionStore）
│   ├── middleware/      # セッション検証・CSRF・Correlation ID
│   ├── oauth/           # OIDC クライアント（Discovery / PKCE / トークン交換）
│   ├── session/         # Redis セッションストア
│   ├── upstream/        # 上流 API へのリバースプロキシ
│   └── config/          # YAML 設定ローダー
└── config/              # 設定ファイル
```

### 設計上の注意

- bff-proxy はドメインモデル（エンティティ・集約）を持たないため、domain 層は存在しない
- usecase は認証フロー・セッション管理に特化した手続き的なビジネスロジックを持つ
- port パッケージは依存性逆転の原則に基づき、usecase が infrastructure に直接依存しないよう仲介する

## Flutter カスタムスキーム（k1s0://）のリスクと移行計画（L-12 監査対応）

Flutter（モバイル）クライアントは認証コールバックに `k1s0://` カスタムスキームを使用している。

### 現行のカスタムスキームの仕組み

```
GET /auth/login?redirect_to=k1s0://auth/callback
  → BFF が IdP にリダイレクト
  → IdP コールバック後: 302 k1s0://auth/callback?code=<exchange-code>
  → Flutter の In-App Browser がカスタムスキームを検知してアプリに渡す
```

`redirect_to` パラメータは **許可リスト方式（allowlist）** で検証しており、
`k1s0://` スキームのみを許可する（`auth_usecase.go` の `validateRedirectTo()` 参照）。

### カスタムスキームのセキュリティリスク

| リスク | 内容 | 現行の緩和策 |
|--------|------|------------|
| スキームハイジャック | 悪意ある同名アプリが同一カスタムスキームを登録し、コールバックを横取り | ワンタイム交換コード（60秒 TTL）で影響を限定 |
| iOS の Universal Links / Android の App Links 非使用 | OS の検証メカニズム（AASA/assetlinks）を経由しない | — |
| コードリプレイ | 交換コードが傍受された場合に再利用される | `auth/exchange` エンドポイントで使用済みコードを削除 |

### 将来の移行計画：App Links / Universal Links への移行（2026-Q4 目標）

カスタムスキームのスキームハイジャックリスクを根本解消するため、
2026-Q4 に以下の移行を計画する。

| 項目 | 内容 |
|------|------|
| iOS | Universal Links (`https://app.k1s0.example.com/auth/callback`) |
| Android | App Links (`https://app.k1s0.example.com/auth/callback`) |
| AASA 設定 | `well-known/apple-app-site-association` をサーバーに配置 |
| assetlinks.json | `well-known/assetlinks.json` をサーバーに配置 |

移行後は BFF-Proxy の `validateRedirectTo()` の allowlist を
`https://app.k1s0.example.com` に変更し、`k1s0://` スキームは廃止する。

参考: 外部技術監査報告書 L-12 "k1s0:// カスタムスキームのリスクと移行計画の明示を求める"
