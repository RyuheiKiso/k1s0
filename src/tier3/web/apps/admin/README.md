# @k1s0/admin

k1s0 管理画面（React + Vite）。`admin-bff` の REST のみを呼ぶ管理者専用 UI。

## 起動

```bash
pnpm install
pnpm --filter @k1s0/admin dev
# → http://localhost:5174
```

## ページ

- `/` テナント一覧（admin-bff /api/admin/tenants 連携、リリース時点 で実装）
- `/audit` 監査ログ（tier1 Audit Service 連携）

## 認可

`admin-bff` は role=admin のみ受け入れる。リリース時点 では Bearer token `admin-token` を `localStorage` に注入してアクセスする想定。リリース時点 で Keycloak OIDC SSO に置換する。
