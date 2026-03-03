# system-bff-proxy デプロイ設計

## デプロイ前提

- ランタイム: Kubernetes
- Service: ClusterIP（HTTP 8080）
- Ingress: `/auth/*`, `/api/*` を bff-proxy にルーティング
- 依存: Redis（Sentinel 対応）, OIDC Provider, 上流 API

## 必須環境変数

| 変数 | 説明 |
| --- | --- |
| `BFF_SERVER_PORT` | HTTP ポート（デフォルト: `8080`） |
| `BFF_OIDC_DISCOVERY_URL` | OIDC Discovery URL |
| `BFF_OIDC_CLIENT_ID` | OAuth2 Client ID |
| `BFF_OIDC_CLIENT_SECRET` | OAuth2 Client Secret |
| `BFF_OIDC_REDIRECT_URI` | `/auth/callback` の完全 URL |
| `BFF_OIDC_POST_LOGOUT_REDIRECT_URI` | ログアウト後の遷移先 |
| `BFF_SESSION_REDIS_ADDR` | Redis/Sentinel アドレス |
| `BFF_SESSION_REDIS_MASTER_NAME` | Sentinel master 名 |
| `BFF_SESSION_TTL` | セッション TTL（例: `30m`） |
| `BFF_SESSION_PREFIX` | セッションキー prefix（例: `bff:session:`） |
| `BFF_CSRF_ENABLED` | CSRF 検証有効フラグ |
| `BFF_CSRF_HEADER_NAME` | CSRF ヘッダー名（例: `X-CSRF-Token`） |
| `BFF_UPSTREAM_BASE_URL` | 上流 API のベース URL |
| `BFF_UPSTREAM_TIMEOUT` | 上流 API タイムアウト（例: `30s`） |

## ヘルスチェック

- Liveness: `GET /healthz`
- Readiness: `GET /readyz`
- Metrics: `GET /metrics`

## セキュリティ設定

- Cookie は `HttpOnly`, `Secure`, `SameSite=Lax`（本番）。
- TLS 終端は Ingress で実施。
- `client_secret` は Secret で注入し、平文 ConfigMap に置かない。
- `redis.password` は Secret 参照で注入。

## ロールアウト

1. bff-proxy を先行デプロイ。
2. `/auth/login` と `/auth/callback` の疎通確認。
3. `/api/*` プロキシ経由で上流 API の認可確認。
4. 監視（401/403/5xx, トークン更新失敗率）を確認して切り替え完了。
