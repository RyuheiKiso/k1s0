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
    cors.go              # CORS ホワイトリスト制御（H-1対応）
    ratelimit.go         # IP ベースのトークンバケットレート制限（H-2対応）
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

#### SSRF 防御（ホワイトリスト方式）

アップストリームへの接続は `ssrfSafeDialContext` で保護する。DNS 解決後の IP アドレスを検証し、内部アドレスへの接続を拒否する。

**設計方針（ADR-0067）**: 設定ファイル（`upstream.base_url`）で定義されたホスト名は**許可リスト（allowedHosts）**として扱い、RFC-1918 SSRF チェックをバイパスする。これにより Docker/K8s 内部ネットワーク（`10.x.x.x`、`172.17.x.x` 等）へのアクセスが可能になる。

| 対象 | 挙動 |
| --- | --- |
| `allowedHosts` に含まれるホスト（設定ファイル由来） | RFC-1918 SSRF チェックをスキップ。クラウドメタデータ IP は除く。 |
| `allowedHosts` に含まれないホスト（動的ターゲット） | 通常の SSRF チェックを適用（RFC-1918 全域をブロック） |
| クラウドメタデータ（`169.254.0.0/16`） | `allowedHosts` に関係なく**常にブロック** |

`config.BFFConfig.AllowedUpstreamHosts()` メソッドで `upstream.base_url` からホスト名を自動抽出する。

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
2. **失敗時**: `retryOIDCDiscovery` ゴルーチンがバックグラウンドで**無限リトライ**（M-4 対応）
   - 短期フェーズ（最初 20 回）: 指数バックオフ（初回 5 秒 → 最大 60 秒）
   - 長期フェーズ（21 回目以降）: 5 分間隔で継続的にリトライ
   - コンテキストキャンセルで graceful shutdown に対応
3. **readiness**: `oidcReady` フラグが `true` になるまで `/readyz` は 503 を返す

#### スレッドセーフティ

`discovered` フラグは `atomic.Bool` で実装し、ゴルーチン間の race condition を防止。

## CSRF トークン有効期間（H-12 監査対応）

CSRF トークンに 30 分の TTL を設定し、長期間使い回されるリスクを軽減する。

### 仕組み

1. セッション作成時に `CSRFTokenCreatedAt`（Unix タイムスタンプ）を `SessionData` に記録する
2. `CSRFMiddleware` でトークン一致確認後に、`CSRFTokenCreatedAt` から 30 分以上経過していたら `403 BFF_CSRF_EXPIRED` を返す
3. トークンリフレッシュ時に CSRF トークンを再生成し、`CSRFTokenCreatedAt` も更新する

### 互換性

`CSRFTokenCreatedAt` が 0（旧セッション）の場合は TTL 検証をスキップする。新規セッションから順次適用される。

| エラーコード | HTTP | 条件 |
| --- | --- | --- |
| `BFF_CSRF_EXPIRED` | 403 | CSRF トークン生成から 30 分超過 |
| `BFF_CSRF_MISMATCH` | 403 | CSRF トークン不一致（従来どおり） |

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
// セッション固定化攻撃防止のため既存セッションを削除する（削除失敗は警告ログ出力し処理続行）（H-3 対応）
if existingSessionID, cookieErr := c.Cookie(CookieName); cookieErr == nil && existingSessionID != "" {
    if err := uc.sessionStore.Delete(ctx, existingSessionID); err != nil {
        slog.WarnContext(ctx, "既存セッションの削除に失敗しました（処理は続行します）",
            "session_id", existingSessionID,
            "error", err,
        )
    }
}
```

### S-04: Redis セッション暗号化

`session/encrypted_store.go` に `EncryptedStore` を実装した（`Store` インターフェース実装）。

- AES-256-GCM による authenticated encryption
- セッションごとにランダムな nonce を生成（12 バイト、GCM standard）
- 保存形式: `base64url(nonce || ciphertext || auth_tag)`
- `SESSION_ENCRYPTION_KEY` 環境変数に hex エンコードされた 32 バイトの鍵を設定する
- **POLY-002 / ADR-0063 対応**: AAD（Additional Authenticated Data）としてセッション ID を渡し、暗号文をセッション ID にバインドする。これによりセッションスワップ攻撃（暗号文を別のキーにコピーする攻撃）を防止する。
  - `Create`, `Get`, `Update`: `gcm.Seal(nonce, nonce, plaintext, []byte(sessionID))`
  - `CreateExchangeCode`, `GetExchangeCode`: `gcm.Seal(nonce, nonce, plaintext, []byte(code))`

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

---

## Doc Sync (2026-03-23)

### OIDC Discovery 永続的 503 の自動復帰（M-4）

`retryOIDCDiscovery` のリトライ上限（20 回）を撤廃し、Keycloak 復旧後に自動接続できる無限リトライに変更した。

#### 変更前の問題

20 回のリトライ失敗後にゴルーチンが終了していたため、Keycloak が復旧しても BFF Proxy ポッドの手動再起動が必要だった。

#### 変更後の動作

| フェーズ | 条件 | リトライ間隔 |
| --- | --- | --- |
| 短期フェーズ | 最初 20 回 | 指数バックオフ（5 秒 → 60 秒） |
| 長期フェーズ | 21 回目以降 | 5 分間隔で継続 |

- Keycloak が復旧すれば最大 5 分以内に自動接続される
- K8s の `readinessProbe` がトラフィックを遮断し続けるため、OIDC 未初期化状態でリクエストを受けない
- コンテキストキャンセル（SIGTERM/SIGINT）でのみゴルーチンが終了する

```go
// 短期フェーズ（最初 20 回）: 指数バックオフ（5秒〜60秒）
// 長期フェーズ（21 回目以降）: 5分間隔で継続的にリトライ
const shortPhaseRetries = 20
longPhaseInterval := 5 * time.Minute
```

### トークンリフレッシュ時のセッション不整合修正（M-5）

`proxy_usecase.go` の `PrepareProxy` でトークンリフレッシュ後のセッション更新順序を修正した。

#### 問題

メモリ上の `sess` を先に更新してから Redis へ保存していたため、Redis 更新失敗時にメモリと Redis の状態が乖離し、次リクエストでセッション不整合が発生していた。

| フェーズ | 旧動作（問題あり） | 新動作（修正後） |
| --- | --- | --- |
| 1 | メモリ `sess` を新トークンで更新 | 新トークン値を一時変数に格納（メモリ未変更） |
| 2 | Redis を更新（失敗してもメモリは更新済） | `tempSess`（shallow copy）を Redis に保存 |
| 3 | Redis 失敗はエラーログのみで続行 | Redis 失敗時はエラーを返しメモリを変更しない |
| 4 | — | Redis 成功後にのみメモリ `sess` を更新 |

#### 修正方針

Redis 先行・メモリ後行の順序を強制し、Redis 失敗時は `BFF_PROXY_SESSION_UPDATE_FAILED` エラーコードを返してリクエストを中断する。

```go
// M-5 対応: Redis を先に更新し、成功後にのみメモリ上のセッションを更新する。
tempSess := *sess
tempSess.AccessToken = updatedAccessToken
tempSess.RefreshToken = updatedRefreshToken
tempSess.IDToken = updatedIDToken
tempSess.ExpiresAt = updatedExpiresAt

if err := uc.sessionStore.Update(ctx, input.SessionID, &tempSess, uc.sessionTTL); err != nil {
    return nil, &ProxyUseCaseError{Code: "BFF_PROXY_SESSION_UPDATE_FAILED", Err: err}
}

// Redis 更新成功後にメモリ上のセッションを更新する
sess.AccessToken = updatedAccessToken
// ...
```

### OIDC Discovery 必須フィールドバリデーション（M-3）

`oauth/client.go` の `Discover()` 関数に、Discovery レスポンスの必須フィールドを検証するバリデーションを追加した。

#### 背景

Discovery レスポンスの必須フィールドが欠落していた場合、後続の認証フローで不正な動作（空エンドポイントへのリクエスト・JWKS 検証スキップ等）が起こり得た。

#### 変更内容

`json.Unmarshal` 後・`c.oidcConfig = &cfg` 前に以下の4フィールドを検証する:

| フィールド | 欠落時のリスク |
| --- | --- |
| `issuer` | JWKSVerifier の iss 検証が機能しない |
| `authorization_endpoint` | 認可リダイレクト先が空になる |
| `token_endpoint` | コード交換・リフレッシュが失敗する |
| `jwks_uri` | ID トークンの署名検証が不可能になる |

また `tokenRequest()` にも `access_token` 空チェックを追加した。`access_token` が欠落したレスポンスをそのまま返すと、上流 API 呼び出しがすべて 401 となるため、早期エラーで呼び出し元に伝える。

### iss/aud 自動検証の明示（H-6）

`oauth/client.go` の `ensureVerifier()` 内の `oidc.NewVerifier` 呼び出し箇所にコメントを追記し、`go-oidc` の自動検証を明示した。

`go-oidc` ライブラリの `IDTokenVerifier.Verify()` は内部で以下を自動検証する:

- `iss`（Issuer）: `NewVerifier` に渡した `cfg.Issuer` と一致すること
- `aud`（Audience）: `oidc.Config.ClientID` が `aud` クレームに含まれること

`oidc.Config{ClientID: c.clientID}` の設定が正しく行われていることを確認済み。追加の手動検証コードは不要。

## CORS 設計（H-1 対応）

`middleware/cors.go` に `CORSMiddleware` を実装した。

### 設計方針

- **ホワイトリスト方式**: `cors.allow_origins` に明示的に列挙したオリジンのみ許可する。`*` のような広域許可は実装しない。
- **Credentials 対応**: `Access-Control-Allow-Credentials: true` を付与し、Cookie 認証と組み合わせて動作する。
- **OPTIONS プリフライト処理**: `204 No Content` を即座に返してチェーンを終了する。
- **Vary ヘッダー**: `Vary: Origin` を付与し、CDN/プロキシが誤ったキャッシュをしないようにする。

### CORS Credentials の設計判断（H-13 監査対応）

`Access-Control-Allow-Credentials: true` をエンドポイント毎に制御する設計と、その理由を明示する。

#### なぜ Credentials が必要か

BFF-Proxy は **HttpOnly Cookie** によるセッション管理を採用している。
ブラウザが Cookie を Cross-Origin リクエストに自動付与するには、
サーバーが `Access-Control-Allow-Credentials: true` を返す必要がある。

Flutter や React SPA が `/auth/session`・`/api/*` を呼び出す際、
Cookie ベースのセッション認証が機能するために本ヘッダーは必須である。

#### エンドポイント別の Credentials 制御（H-13 監査対応）

監査指摘に基づき、`credentials_paths` 設定によってエンドポイント毎に
`Access-Control-Allow-Credentials: true` を付与するかどうかを制御する。

| パスプレフィックス | Credentials | 理由 |
| --- | --- | --- |
| `/auth/` | `true` | 認証フロー（login/callback/logout/refresh）はセッション Cookie が必要 |
| `/api/` | `true` | 認証済みユーザー向け API は Cookie による認証が必要 |
| `/healthz` | 付与しない | 公開ヘルスチェックエンドポイントは認証不要 |
| `/metrics` | 付与しない | Prometheus スクレイプは認証不要 |

`credentials_paths` が未設定の場合は後方互換として全オリジンに `true` を返す。

#### Credentials と `*` の同時指定が禁止される理由

CORS 仕様（Fetch Standard）では、`Allow-Credentials: true` と
`Access-Control-Allow-Origin: *` の組み合わせはブラウザによってブロックされる。
このため、オリジンを明示的にホワイトリスト管理することが技術的必須要件となっている。
`cors.go` 起動時に `*` が含まれる場合は `error` を返して起動を阻止する。

#### セキュリティ上の注意点

| 項目 | 内容 |
| --- | --- |
| 許可オリジンの管理 | `config.yaml` の `cors.allow_origins` で列挙管理。PR レビューで変更確認を徹底する |
| localhost の本番混入防止 | `http://localhost:*` は `ENV=dev` の場合のみ設定し、本番 config.yaml には含めない |
| Cookie の SameSite=Lax | CSRF 対策として `SameSite=Lax` を設定済み。Credentials と組み合わせて二重防御 |
| プリフライトキャッシュ | `max_age_secs: 600`（10分）でプリフライトリクエストの頻度を抑制 |
| Credentials のパス制限 | `credentials_paths` で明示したパスのみに Credentials を付与し最小権限を実現 |

### 設定例（config.yaml）

```yaml
cors:
  enabled: true
  allow_origins:
    - "https://app.k1s0.example.com"
    - "http://localhost:3000"   # 開発環境のみ
  # H-13: 認証が必要なエンドポイントのみ Credentials を許可する
  credentials_paths:
    - "/auth/"
    - "/api/"
  max_age_secs: 600
```

許可されていないオリジンからのリクエストには CORS ヘッダーを付与しない（ブラウザが自動的に拒否する）。

## レート制限設計（H-2 対応）

`middleware/ratelimit.go` に `RateLimitMiddleware` を実装した。

### 設計方針

- **IP ベースのトークンバケット**: `sync.Map` と `time` のみを使用した外部依存ゼロの実装。
- **burst 許容**: 一時的なトラフィックスパイクを `burst` 設定で許容する。
- **429 Too Many Requests**: 制限超過時に `Retry-After: 1` ヘッダーと 429 を返す。
- **定期クリーンアップ**: 10 分間アクセスのない IP エントリを自動削除してメモリリークを防止する。

### 設定例（config.yaml）

```yaml
rate_limit:
  enabled: true
  rps: 100     # 1 IP あたり 100 req/sec
  burst: 200   # 瞬間的に 200 req まで許容
```

## Cookie SameSite=Lax 設計記録（M-10 監査対応）

### BFF-Proxy が SameSite=Lax を採用した理由

`auth_handler.go` の全 Cookie 設定箇所（Login/Callback/Session）で
`http.SameSiteLaxMode` を使用している。

```go
// auth_handler.go: 全 Cookie 発行時に SameSite=Lax を設定（行 105, 188, 334 付近）
c.SetSameSite(http.SameSiteLaxMode)
c.SetCookie(CookieName, sessionID, ...)
```

### SameSite の各モードの比較

| モード | クロスサイトリクエストへの Cookie 送信 | BFF-Proxy での影響 |
|--------|-------------------------------------|------------------|
| Strict | 一切送信しない | IdP からのリダイレクト（GET /auth/callback）時も Cookie が送信されず認証フロー失敗 |
| **Lax（採用）** | 安全なリクエスト（GET/HEAD）のみ送信 | IdP からの GET リダイレクトでは Cookie が届く。POST 系のクロスサイトは遮断 |
| None | 全リクエストで送信 | CSRF リスクが高まる。Secure フラグ必須 |

### Strict を選択しなかった理由

OAuth2/OIDC の認証コールバックは IdP からの **クロスオリジンの GET リダイレクト** である。
`SameSite=Strict` を設定すると、このリダイレクト時に `k1s0_oauth_state` Cookie と
`k1s0_pkce_verifier` Cookie が送信されず、state 検証が失敗する。

PKCE フローにおける認証コールバックは GET メソッドのリダイレクトであり、
これは `SameSite=Lax` の許可対象（トップレベルナビゲーションの安全メソッド）に該当する。

### CSRF との関係

`SameSite=Lax` は POST/PUT/DELETE のクロスサイトリクエストから Cookie を保護するが、
GET リクエストは保護対象外である。このため、状態変更が発生する API エンドポイントには
別途 CSRF トークン検証（`CsrfMiddleware`）を実施している（H-12 対応）。

二重防御の構成:
- `SameSite=Lax`: POST 系クロスサイトリクエストの自動遮断
- `X-CSRF-Token` ヘッダー検証: GET 以外のすべての状態変更操作を明示的に保護

Kong が起動しない状態（C-2 修正前）では BFF Proxy が唯一の入口となるため、このレート制限が DDoS 対策として機能する。
