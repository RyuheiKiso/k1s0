# E2E テスト

Playwright を使用した k1s0 プラットフォームの E2E テスト。

## セットアップ

```bash
cd tests/e2e

# 依存関係のインストール（package-lock.json を使用して再現性を確保）
npm ci

# Playwright ブラウザのインストール
npx playwright install --with-deps chromium
```

## 前提条件

- Docker Compose でサービス群が起動済みであること
  ```bash
  docker compose --profile infra --profile system up -d
  ```
- 全サービスが healthy 状態であること
  ```bash
  docker compose --profile infra --profile system ps
  ```

## AI サービスの起動（実験的）

AI 関連サービス（ai-gateway, ai-agent）は実験中のサービスです。
通常の開発では不要ですが、AI 機能のテストが必要な場合は以下のプロファイルを追加します:

```bash
docker compose --profile infra --profile system --profile experimental-ai up -d
```

## テスト実行

```bash
# 全テスト実行
npm test

# UI モードで実行（デバッグ用）
npm run test:ui

# ブラウザ表示ありで実行
npm run test:headed

# デバッグモード
npm run test:debug

# レポート表示
npm run report
```

## テスト構成

| ファイル | 内容 |
|----------|------|
| `specs/health-check.spec.ts` | 各サービスのヘルスチェック検証 |
| `specs/auth-flow.spec.ts` | BFF 認証フロー検証（未認証 401・ログインリダイレクト・CSRF 保護） |
| `specs/api-crud.spec.ts` | API CRUD エンドポイントルーティング確認（Order/Inventory/Payment） |

## 環境変数

全ての URL とポートは `specs/config.ts` で一元管理されている。spec ファイルにハードコードせず、以下の環境変数で上書き可能。

| 変数 | デフォルト | 説明 |
|------|-----------|------|
| `BASE_URL` | `http://localhost:8082` | BFF プロキシのベース URL |
| `KEYCLOAK_URL` | `http://localhost:8180` | Keycloak のベース URL |
| `AUTH_PORT` | `8083` | auth サービスのポート |
| `CONFIG_PORT` | `8084` | config サービスのポート |
| `SAGA_PORT` | `8085` | saga サービスのポート |
| `DLQ_MANAGER_PORT` | `8086` | dlq-manager サービスのポート |
| `FEATUREFLAG_PORT` | `8087` | featureflag サービスのポート |
| `RATELIMIT_PORT` | `8088` | ratelimit サービスのポート |
| `TENANT_PORT` | `8089` | tenant サービスのポート |
| `VAULT_PORT` | `8091` | vault サービスのポート |
| `BFF_PROXY_PORT` | `8082` | bff-proxy サービスのポート |
| `CI` | - | CI 環境フラグ（リトライ・ワーカー数の制御） |
