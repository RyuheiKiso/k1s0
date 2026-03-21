# system-bff-proxy 実装設計

## 実装方針

- 実装言語は Go（Gin）。
- BFF は Cookie セッションを唯一のクライアント認証手段として扱う。
- OAuth2/OIDC トークンは Redis に保存し、ブラウザには HttpOnly Cookie のみ返す。
- 上流 API 呼び出し時はセッションからアクセストークンを復元し `Authorization: Bearer` を付与する。

## モジュール構成

```text
cmd/
  server/                # エントリポイント
internal/
  config/                # 設定ロード
  handler/
    auth_handler.go      # /auth/login, /auth/callback, /auth/session, /auth/exchange, /auth/logout
    proxy_handler.go     # /api/*path
    health_handler.go    # /healthz, /readyz, /metrics
    error_response.go    # 共通エラーレスポンス
  middleware/
    session.go           # セッション検証
    csrf_middleware.go   # CSRF 検証
    traceid.go           # request_id / trace_id 付与
    request_id.go        # request_id 取得ヘルパー
    trace_middleware.go  # トレースミドルウェア
    metrics_middleware.go # Prometheus メトリクス
  oauth/
    client.go            # Discovery, token exchange, refresh（OAuthClient interface を実装）
    pkce.go              # code_verifier/code_challenge 生成
  session/
    redis_store.go       # セッション保存/取得/削除
  upstream/
    reverse_proxy.go     # 上流 API への転送
```

## セッションモデル

- キー: `bff:session:{session_id}`
- 値:
  - `access_token`
  - `refresh_token`
  - `id_token`
  - `sub`
  - `expires_at`
  - `created_at`
  - `csrf_token`

## フロー要点

### ブラウザフロー（React SPA）

1. `GET /auth/login` で PKCE と state を生成し Cookie に保存。IdP にリダイレクト。
2. `GET /auth/callback` で state 検証後に token exchange 実行。Redis にセッション保存後、`session_id` Cookie を返却。
3. `GET /auth/session` で SPA がセッション確認。有効なら 200 + ユーザー情報 + CSRF トークン。
4. `ANY /api/*path` でセッション検証し、Bearer トークンを付与して上流へ転送。
5. 期限切れ時は refresh token で自動更新。失敗時は `BFF_PROXY_TOKEN_EXPIRED`。

### セキュリティ設計

#### redirect_to スキーム検証（allowlist 方式）

`redirect_to` クエリパラメータは **allowlist 方式** で検証する。`k1s0://` スキームのみを許可し、それ以外はすべて拒否する。
denylist 方式（特定スキームだけをブロック）は未知の危険スキームが通過するリスクがあるため採用しない。

#### セッション期限切れチェック（ミドルウェア）

`SessionMiddleware` は Redis からセッションを取得後、`SessionData.IsExpired()` でアクセストークンの有効期限を確認する。
Redis TTL のみに依存すると、アクセストークンが失効してもスライディング TTL 延長により Redis キーが残存する場合がある。
`ExpiresAt` フィールドが過去の場合は 401 `BFF_SESSION_EXPIRED` を返す。

### モバイルフロー（Flutter）

1. `GET /auth/login?redirect_to=k1s0://auth/callback` でログイン開始（`k1s0://` スキームのみ許可）。
2. `GET /auth/callback` でセッション作成後、ワンタイム交換コード（60秒 TTL）を生成し `k1s0://auth/callback?code=...` にリダイレクト。
3. `GET /auth/exchange?code=...` で交換コードを検証し、セッション Cookie を発行。モバイル HTTP クライアント（Dio）が Set-Cookie を保持。
4. 以降のフローはブラウザフローと同一。

## OAuthClient インターフェース

`AuthHandler` はテスト容易性のために具象型 `*oauth.Client` ではなく `OAuthClient` インターフェースに依存する。
`*oauth.Client` はこのインターフェースを暗黙的に満たすため、プロダクションコードの変更は不要。

```go
// handler/auth_handler.go
type OAuthClient interface {
    AuthCodeURL(state, codeChallenge string) (string, error)
    ExchangeCode(ctx context.Context, code, codeVerifier string) (*oauth.TokenResponse, error)
    ExtractSubject(ctx context.Context, idToken string) (string, error)
    LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error)
}
```

テスト時は関数フィールド方式の `mockOAuthClient` で振る舞いを差し替える（`auth_flow_test.go` 参照）。

### OIDC Discovery バックグラウンド再試行

起動時に OIDC Discovery エンドポイント（`/.well-known/openid-configuration`）から
プロバイダ情報を取得・キャッシュする。

#### 動作フロー

1. **起動時**: `Discover()` で OIDC エンドポイント情報を取得
2. **失敗時**: `retryOIDCDiscovery` ゴルーチンがバックグラウンドで再試行
   - 指数バックオフ: 初回 5 秒 → 最大 60 秒
   - コンテキストキャンセルで graceful shutdown に対応
3. **readiness**: `IsDiscovered()` が `true` になるまで `/readyz` は 503 を返す

#### スレッドセーフティ

`discovered` フラグは `atomic.Bool` で実装し、ゴルーチン間の race condition を防止。

## エラー実装ポリシー

- エラーコードは `BFF_*` の固定値を返す。
- 認証・CSRF 不備は 401/403、内部処理失敗は 500。
- `request_id` を全エラーレスポンスに付与する。

## Doc Sync (2026-03-21)

### BFF-Proxy 3件の改善 [技術品質監査 High 3-4, 3-5, Medium 7-2]

**P1-9: リバースプロキシのタイムアウト完全化 [High 3-4]**

`reverse_proxy.go` でタイムアウト設定が `ResponseHeaderTimeout` のみだった。
他のタイムアウトが未設定のままだと、TCP 接続確立・TLS ハンドシェイク・アイドル接続が
無制限にハングする可能性があった。

以下のタイムアウトを明示設定するよう変更：

| 設定 | 値 | 用途 |
| --- | --- | --- |
| `DialContext` (Timeout) | 30s | TCP 接続確立（DNS + 3way HS） |
| `DialContext` (KeepAlive) | 30s | TCP Keep-Alive |
| `TLSHandshakeTimeout` | 10s | TLS 証明書検証を含むハンドシェイク |
| `IdleConnTimeout` | 90s | アイドル接続の保持上限 |
| `ResponseHeaderTimeout` | configurable | アップストリーム応答待ち |

**P1-10: OAuth HTTP クライアントのタイムアウト設定可能化 [High 3-5]**

`oauth/client.go` の `NewClient` がタイムアウトを `10 * time.Second` にハードコードしていた。
`WithHTTPTimeout(d time.Duration)` オプション関数を追加し、後方互換性を維持しつつカスタマイズ可能にした。

```go
// デフォルト（10秒）
client := oauth.NewClient(...)

// カスタムタイムアウト
client := oauth.NewClient(..., oauth.WithHTTPTimeout(30 * time.Second))
```

**P1-19: ALLOW_REDIS_SKIP の本番環境ガード [Medium 7-2]**

`main.go` で `ALLOW_REDIS_SKIP=true` を設定すると、どの環境でも Redis 接続失敗を無視してしまっていた。
`cfg.App.Environment == "development"` の場合のみスキップを許可するよう変更した。

production / staging 環境では `ALLOW_REDIS_SKIP=true` でも Redis 接続失敗はエラーで終了する。
