# E2E テスト

Playwright を使用した k1s0 プラットフォームの E2E テスト。

## セットアップ

```bash
cd tests/e2e

# 依存関係のインストール
npm install

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

## 環境変数

| 変数 | デフォルト | 説明 |
|------|-----------|------|
| `BASE_URL` | `http://localhost:8082` | BFF プロキシのベース URL |
| `CI` | - | CI 環境フラグ（リトライ・ワーカー数の制御） |
