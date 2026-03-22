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
  - `roles` (Keycloak realm roles — JWKS 検証済み ID トークンから取得)
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

| 条件 | 挙動 |
| --- | --- |
| expired + refresh token なし | 即座に `401 BFF_SESSION_EXPIRED` を返す |
| expired + refresh token あり | gin context に `session_needs_refresh = true` フラグを設定し、handler に通す |

handler 側（`proxy_handler.go`）が `sess.IsExpired() && sess.RefreshToken != ""` を確認して silent refresh を実行する。

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
    // ExtractClaims は JWKS 署名検証済みの ID トークンから subject と realm roles を返す。
    // アクセストークン（署名未検証）からロールを取得する旧方式を廃止し、
    // ロール改ざん攻撃を防止する（S-02 対応）。
    ExtractClaims(ctx context.Context, idToken string) (subject string, roles []string, err error)
    LogoutURL(idTokenHint, postLogoutRedirectURI string) (string, error)
    ClearDiscoveryCache()
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

## セキュリティ設計（追記）

### S-03: セッション固定化防止

`Callback` ハンドラーの冒頭で既存セッションを削除する。
認証完了前の Cookie にセッション ID が存在する場合、セッションストアから削除してから新規セッションを作成する。
これにより攻撃者が認証前に取得したセッション ID を認証後に再利用できなくなる。

```go
if existingSessionID, cookieErr := c.Cookie(CookieName); cookieErr == nil && existingSessionID != "" {
    _ = h.sessionStore.Delete(c.Request.Context(), existingSessionID)
}
```

### S-04: Redis セッション暗号化

`session/encrypted_store.go` に `EncryptedStore` を実装した（`Store` インターフェース実装）。

- AES-256-GCM による authenticated encryption
- セッションごとにランダムな nonce を生成（12 バイト、GCM standard）
- 保存形式: `base64url(nonce || ciphertext || auth_tag)`
- `SESSION_ENCRYPTION_KEY` 環境変数に hex エンコードされた 32 バイトの鍵を設定する

`main.go` の起動時に `SESSION_ENCRYPTION_KEY` が設定されていれば `EncryptedStore` を使用し、未設定の場合は `RedisStore` にフォールバックして警告を出力する。

### S-05: url.Parse エラーハンドリング

モバイルフローの Callback ハンドラーで `url.Parse` のエラーを無視していた箇所（`redirectURL, _ := url.Parse(...)`）を修正した。
パース失敗時は `400 BFF_AUTH_REDIRECT_URL_INVALID` を返す。

## Go 実装（追記）

### G-01: HTTP サーバータイムアウト修正

`main.go` の `http.Server` タイムアウトをプロキシ用途に適した値に変更した。

| パラメーター | 変更前デフォルト | 変更後デフォルト | 理由 |
| --- | --- | --- | --- |
| `ReadTimeout` | 10s | 60s | 大きなリクエストボディ・スロークライアントへの対応 |
| `WriteTimeout` | 30s | 120s | 上流サービスの処理時間（upstreamTimeout 30s + バッファ）を考慮 |

設定ファイル（`config.yaml`）の `server.read_timeout` / `server.write_timeout` で上書き可能。

### G-02: リクエストボディサイズ制限

`main.go` の Gin ルーター初期化後に `http.MaxBytesReader` ミドルウェアを追加した。
全エンドポイントに対してリクエストボディを 64MB に制限し、大容量リクエストによる DoS・OOM を防止する。

```go
router.Use(func(c *gin.Context) {
    c.Request.Body = http.MaxBytesReader(c.Writer, c.Request.Body, 64*1024*1024)
    c.Next()
})
```

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
`config.IsDevEnvironment(env)` の場合のみスキップを許可するよう変更した。
`dev`・`development`・`local` が開発環境として扱われ、production / staging 環境では `ALLOW_REDIS_SKIP=true` でも Redis 接続失敗はエラーで終了する。

**F09: 環境名統一（IsDevEnvironment）[2026-03-21]**

`main.go` 内で `env == "development"` と `env != "dev"` が混在しており、`config.yaml` のデフォルト `dev` と不一致だった。
`internal/config/config.go` に `IsDevEnvironment()` ヘルパーを追加し、全比較箇所を統一した。
JWKS の `context.Background()` もアプリケーションレベルの `ctx` に変更し、シャットダウン時にキャンセルされるよう修正した。

---

## Doc Sync (2026-03-21)

### singleflight によるトークンリフレッシュ重複排除 [技術品質監査 Medium G-03]

`proxy_handler.go` のトークンリフレッシュに `golang.org/x/sync/singleflight` を導入した。
同一セッション ID に対して並行リクエストが殺到した場合、最初の 1 件のみ `RefreshToken` を呼び出し、
他のリクエストは同じ結果を共有する。これにより Keycloak のレート制限エラーを防止する。

### Redis PubSub ドロップコールバック [技術品質監査 Medium G-04]

`building-blocks/redis_pubsub.go` のメッセージドロップ時にコールバックを呼び出す機能を追加した（G-04 対応）。

```go
// Prometheus カウンターをコールバックとして登録する例
pubsub := buildingblocks.NewRedisPubSub(name, client,
    buildingblocks.WithDroppedMessageCallback(func(topic string) {
        droppedCounter.WithLabelValues(topic).Inc()
    }),
)
```

### OIDC リトライ上限 [技術品質監査 Medium G-05]

`retryOIDCDiscovery` に最大 20 回のリトライ上限を追加した。
無限リトライによる長時間待機を防止する。20 回失敗後はゴルーチンを終了し、エラーログを出力する。

### TrustedProxies 設定 [技術品質監査 Medium G-06]

`main.go` に `router.SetTrustedProxies(nil)` を追加した。
Gin のデフォルト（全プロキシ信頼）を無効化し、X-Forwarded-For ヘッダーを直接の接続元 IP で上書きする。
ロードバランサー配下では適切な CIDR（例: `10.0.0.0/8`）に変更すること。

---

## Doc Sync (2026-03-21)

### トークンリフレッシュ失敗時のセッション削除（H-003）

トークンリフレッシュが失敗した場合、Redis 上の無効なセッションを即座に削除する。
削除しない場合、セッション TTL が残っている間、攻撃者がセッション ID を使い回せるリスクがある。

```go
// proxy_handler.go: リフレッシュ失敗時のセッション削除
if err != nil {
    if delErr := h.sessionStore.Delete(ctx, sessionID); delErr != nil {
        h.logger.Error("期限切れセッションの削除に失敗しました", ...)
    }
    abortErrorWithMessage(c, http.StatusUnauthorized, ...)
    return
}
```

### 型アサーションの安全化（H-009）

`c.Get()` が返す `interface{}` 値への型アサーションは comma-ok パターンを使用する。
直接型アサーション（`x.(bool)` 形式）は実行時パニックのリスクがある。

```go
// 正しいパターン
if refresh, ok := needsRefresh.(bool); ok && refresh { ... }
// 禁止パターン
if needsRefresh.(bool) { ... }  // パニックのリスク
```

### OTel トレーサーシャットダウン（H-004）

`tp.Shutdown()` にはタイムアウト付きコンテキストを渡す。
`context.Background()` を使用すると、OTel Collector が無応答の場合にシャットダウンが無期限にブロックされる。

```go
shutdownTraceCtx, cancelTrace := context.WithTimeout(context.Background(), 5*time.Second)
defer cancelTrace()
tp.Shutdown(shutdownTraceCtx)
```

---

## Doc Sync (2026-03-22)

### OIDC TLS 検証の本番環境 Fatal 化（M-11）

`main.go` の OIDC DiscoveryURL TLS チェックを環境に応じた分岐に変更した。

| 環境 | 動作 |
| --- | --- |
| 本番環境（`production`, `prod`） | `logger.Error` を出力後に `os.Exit(1)` で即時終了 |
| 開発・ステージング環境 | `logger.Warn` を出力して起動を継続 |

本番環境では IdP との通信に TLS（HTTPS）が必須である。`https://` で始まらない `discovery_url` が設定された場合、中間者攻撃のリスクが生じるため、サーバー起動を拒否する。

```go
// OIDC DiscoveryURL が HTTPS でない場合は環境に応じて処理を分岐する。
if !strings.HasPrefix(cfg.Auth.DiscoveryURL, "https://") {
    if isProductionEnvironment(cfg.App.Environment) {
        logger.Error("OIDC discovery_url が TLS (https) を使用していません。本番環境では https が必須です",
            slog.String("discovery_url", cfg.Auth.DiscoveryURL),
            slog.String("environment", cfg.App.Environment),
        )
        os.Exit(1)
    }
    logger.Warn("OIDC discovery_url が TLS (https) を使用していません。本番環境では https を使用してください",
        slog.String("discovery_url", cfg.Auth.DiscoveryURL),
        slog.String("environment", cfg.App.Environment),
    )
}
```

環境判定には既存の `isProductionEnvironment()` ヘルパーを使用する（`production`, `prod` を本番として扱う）。
