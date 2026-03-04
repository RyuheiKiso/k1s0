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
    auth_handler.go      # /auth/login, /auth/callback, /auth/logout
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
    client.go            # Discovery, token exchange, refresh
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

1. `GET /auth/login` で PKCE と state を生成し Cookie に保存。
2. `GET /auth/callback` で state 検証後に token exchange 実行。
3. Redis にセッション保存後、`session_id` Cookie を返却。
4. `ANY /api/*path` でセッション検証し、Bearer トークンを付与して上流へ転送。
5. 期限切れ時は refresh token で自動更新。失敗時は `BFF_PROXY_TOKEN_EXPIRED`。

## エラー実装ポリシー

- エラーコードは `BFF_*` の固定値を返す。
- 認証・CSRF 不備は 401/403、内部処理失敗は 500。
- `request_id` を全エラーレスポンスに付与する。
