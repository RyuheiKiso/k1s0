# @k1s0/portal

k1s0 配信ポータル（React + Vite）。`portal-bff` の REST / GraphQL を呼ぶエンドユーザ向け Web UI。

## 起動

```bash
# tier3/web ルートから。
pnpm install
pnpm --filter @k1s0/portal dev
# → http://localhost:5173
```

環境変数（`.env` または起動時 export）:

| 変数 | 既定値 | 説明 |
|---|---|---|
| `VITE_BFF_URL` | （無し） | BFF URL。必須。 |
| `VITE_TENANT_ID` | `tenant-dev` | tenant ID（X-Tenant-Id ヘッダ） |
| `VITE_ENVIRONMENT` | `dev` | 環境名 |

## ページ

- `/` Dashboard: テナントメトリクス（リリース時点 はプレースホルダ）
- `/state` State Explorer: 任意の k1s0 State キーを取得して表示

## ビルドと配信

```bash
pnpm --filter @k1s0/portal build
docker build -f apps/portal/Dockerfile -t ghcr.io/k1s0/t3-portal:dev .
```

ランタイムは nginx で静的配信（Node ランタイム不要）、`nginx.conf` で SPA fallback と長期キャッシュを設定する。
