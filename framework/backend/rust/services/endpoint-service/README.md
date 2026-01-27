# endpoint-service

エンドポイント情報を担う framework 共通サービス。

## 概要

サービス間通信のエンドポイント情報を一元管理する。

## 責務

- エンドポイント情報の取得（Get/List）
- サービス名からエンドポイントへの解決（ResolveEndpoint）

## 公開API

### gRPC

- `proto/endpoint/v1/endpoint.proto`

主要 RPC:
- `GetEndpoint`: エンドポイント取得
- `ListEndpoints`: エンドポイント一覧取得
- `ResolveEndpoint`: サービス名からアドレスを解決

### ヘルスチェック

- gRPC Health Check Protocol (`grpc.health.v1.Health`)

## 依存

- DB（PostgreSQL）: `fw_m_endpoint`

## 設定

- `config/{env}.yaml`

## DB

### 所有テーブル

| テーブル | 説明 |
|---------|------|
| `fw_m_endpoint` | エンドポイント |

### マイグレーション

`migrations/` に配置

## 認証・認可

- エンドポイント取得は認証必須

## 監視

- メトリクス: エンドポイント取得成功/失敗数
- ログ: エンドポイント取得（JSON形式）
- トレース: リクエスト単位

## 起動方法

```bash
# 基本起動（開発環境）
cargo run -- --env dev --port 50052

# オプション一覧
endpoint-service --help

# 主なオプション:
#   --env <ENV>             環境名 (dev, stg, prod) [default: dev]
#   --port, -p <PORT>       gRPCポート [default: 50052]
#   --config <PATH>         設定ファイルパス
#   --secrets-dir <PATH>    シークレットディレクトリ
#   --database-url <URL>    PostgreSQL接続URL（環境変数 DATABASE_URL も可）
#   --namespace <NS>        Kubernetesネームスペース [default: default]
#   --cluster-domain <DOM>  クラスタドメイン [default: cluster.local]
```

### 動作モード

- **InMemory**: `--database-url` なしで起動。開発・テスト用。
- **PostgreSQL**: `--database-url` 指定で起動（現在実装中）。

### アドレス解決

Kubernetes DNS規約に基づいてアドレスを生成：
- `auth-service` (grpc) → `auth-service.{namespace}.svc.{cluster_domain}:50051`
- `api-gateway` (http) → `api-gateway.{namespace}.svc.{cluster_domain}:8080`

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

基本実装完了。InMemoryリポジトリで動作。PostgreSQLリポジトリは実装済み（統合未完了）。
