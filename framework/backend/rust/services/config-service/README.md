# config-service

動的設定を担う framework 共通サービス。

## 概要

`fw_m_setting` を扱い、feature サービスに動的設定を提供する。

## 責務

- 設定の取得（Get/List）
- 設定のキャッシュ（InMemory/Redis）
- 設定変更の通知（WatchSettings）

## 公開API

### gRPC

- `proto/config/v1/config.proto`

主要 RPC:
- `GetSetting`: 設定取得
- `ListSettings`: 設定一覧取得
- `WatchSettings`: 設定変更の監視（Server Streaming）

### ヘルスチェック

- gRPC Health Check Protocol (`grpc.health.v1.Health`)

## 依存

- DB（PostgreSQL）: `fw_m_setting`
- キャッシュ（オプション）: Redis

## 設定

- `config/{env}.yaml`
- キャッシュ TTL、最大保持世代数

## DB

### 所有テーブル

| テーブル | 説明 |
|---------|------|
| `fw_m_setting` | 設定 |

### マイグレーション

`migrations/` に配置

## 認証・認可

- 設定取得は認証必須
- 設定変更は管理者権限必須

## 監視

- メトリクス: 設定取得成功/失敗数、キャッシュヒット率
- ログ: 設定取得、キャッシュ更新（JSON形式）
- トレース: リクエスト単位

## 起動方法

```bash
# 基本起動（開発環境）
cargo run -- --env dev --port 50051

# オプション一覧
config-service --help

# 主なオプション:
#   --env <ENV>           環境名 (dev, stg, prod) [default: dev]
#   --port, -p <PORT>     gRPCポート [default: 50051]
#   --config <PATH>       設定ファイルパス
#   --secrets-dir <PATH>  シークレットディレクトリ
#   --database-url <URL>  PostgreSQL接続URL（環境変数 DATABASE_URL も可）
#   --redis-url <URL>     Redis接続URL（環境変数 REDIS_URL も可）
```

### 動作モード

- **InMemory**: `--database-url` なしで起動。開発・テスト用。
- **PostgreSQL**: `--database-url` 指定で起動（現在実装中）。
- **Redis Cache**: `--redis-url` 指定で有効化（現在実装中）。

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

基本実装完了。InMemoryリポジトリ/キャッシュで動作。PostgreSQL/Redisは実装済み（統合未完了）。
