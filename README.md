# test-db-cache

## 概要

test-db-cache サービスの概要を記述してください。

## 責務

このサービスが担当する責務を記述してください。

- TODO: 責務1
- TODO: 責務2

## 公開API

公開APIの詳細を記述してください。
## 依存

### 内部依存（framework）

- k1s0-config: 設定管理
- k1s0-error: エラー処理
- k1s0-observability: ログ/トレース/メトリクス

### 外部依存

- TODO: 外部依存を記述

## 設定

設定ファイル: `config/{env}.yaml`

| キー | 説明 | デフォルト |
|------|------|------------|
| TODO | TODO | TODO |

## DB

このサービスが所有するテーブル:

- TODO: テーブル名を記述

マイグレーション: `migrations/` を参照
## 認証・認可

TODO: 認証・認可の方針を記述

## 監視

### メトリクス

- TODO: メトリクス名を記述

### アラート

- TODO: アラート条件を記述

## 起動方法

```bash
# 開発環境
cargo run -- --env dev --config ./config

# 本番環境
./test-db-cache --env prod --config /etc/k1s0/config --secrets-dir /var/run/secrets/k1s0
```

## リリース

- CI: `.github/workflows/` を参照
- デプロイ: `deploy/` を参照（Kustomize）
