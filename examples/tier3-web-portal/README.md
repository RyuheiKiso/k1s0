# `examples/tier3-web-portal/` — React + Vite 最小 portal

tier3 Web layer（React + TypeScript + Vite + pnpm）の典型的な実装パタンを示す
最小 portal 例。

## 目的

- `src/tier3/web/apps/portal` の構造（Vite / React Router / @k1s0/sdk）を新規メンバーが
  真似できる
- tier3 から tier1 への呼び出しが「BFF 経由 / 直接 SDK」のどちらでも動くことを示す
- Storybook なし・テスト最小・i18n stub 含めた「最小だが本番形」の例

## 想定読者

- tier3 Web の新規コミッタ
- 既存社内ポータルを k1s0 SDK で書き直す開発者
- @k1s0/sdk TypeScript の利用パタンを学びたい人

## scope（リリース時点）

リリース時点では以下 3 点のみを満たす最小骨格を配置する:

1. `package.json` / `vite.config.ts` / `tsconfig.json`
2. `index.html` / `src/main.tsx` / `src/App.tsx`
3. ホームページ 1 枚（tier1 health check を `@k1s0/sdk` で叩いて表示）

**未実装（採用初期に拡張予定）:**

- React Router の複数ルート例
- `@k1s0/sdk` を使った State / Audit / Decision 呼び出し例
- 認証統合（Keycloak OIDC、ADR-SEC-001 整合）
- i18n（react-i18next）
- E2E（Playwright）
- Dockerfile（multi-stage / nginx 配信）
- catalog-info.yaml

## 関連 docs / ADR

- `docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/`（tier3 Web レイアウト準ずる）
- ADR-DEV-001（Paved Road）
- ADR-FM-001（OpenFeature / flagd）

## 起動方法（採用初期完成後の想定）

```bash
cd examples/tier3-web-portal
pnpm install
pnpm dev
# http://localhost:5173 を開く
```

## 参照する tier1 API

- HealthService（standard gRPC health protocol、疎通確認）
- LogService（フロント発生イベントのログ記録、採用初期）
- AuditService（操作ログ、採用初期）
