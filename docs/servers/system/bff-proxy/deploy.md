# system-bff-proxy デプロイ設計

## デプロイ前提

- ランタイム: Kubernetes
- Service: ClusterIP（HTTP 8080）
- Ingress: `/auth/*`（login, callback, session, exchange, logout）, `/api/*` を bff-proxy にルーティング
- 依存: Redis（Sentinel 対応）, OIDC Provider, 上流 API

## 設定方式

bff-proxy は `config.yaml` を正とする。  
`BFF_*` 環境変数はサポートしない。

| 変数 | 説明 |
| --- | --- |
| `CONFIG_PATH` | 設定ファイルパス（省略時: `config/config.yaml`） |
| `ENV_CONFIG_PATH` | 上書き設定ファイルパス（環境別設定） |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP エクスポート先（トレース送信先） |

```yaml
server:
  port: 8080
auth:
  discovery_url: "https://idp.example.com"
  client_id: "bff-proxy"
  client_secret: ""
  redirect_uri: "https://app.example.com/auth/callback"
session:
  redis:
    addr: "redis:6379"
    master_name: "mymaster"
  ttl: "30m"
csrf:
  enabled: true
upstream:
  base_url: "http://auth-server:8080"
  timeout: "30s"
```

## ヘルスチェック

- Liveness: `GET /healthz`
- Readiness: `GET /readyz`
- Metrics: `GET /metrics`

## Vault 認証設定（H-1 / M-18 監査対応）

| 項目 | 値 |
|------|-----|
| Vault ロール | `system` |
| ServiceAccount 名 | `bff-proxy-sa` |
| namespace | `k1s0-system` |
| token_max_ttl | 14400（4h） |

`infra/terraform/modules/vault/auth.tf` の `system` ロールに `bound_service_account_names = ["bff-proxy-sa", ...]` が設定されていること。
`infra/helm/services/system/bff-proxy/values.yaml` の `serviceAccount.name = "bff-proxy-sa"` / `vault.role = "system"` と一致させること。

## セキュリティ設定

- Cookie は `HttpOnly`, `Secure`, `SameSite=Lax`（本番）。
- TLS 終端は Ingress で実施。
- `client_secret` は Secret で注入し、平文 ConfigMap に置かない。
- `redis.password` は Secret 参照で注入。

## ロールアウト

1. bff-proxy を先行デプロイ。
2. `/auth/login`, `/auth/callback`, `/auth/session`, `/auth/exchange` の疎通確認。
3. `/api/*` プロキシ経由で上流 API の認可確認。
4. 監視（401/403/5xx, トークン更新失敗率）を確認して切り替え完了。
