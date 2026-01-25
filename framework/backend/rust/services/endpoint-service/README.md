# endpoint-service

エンドポイント情報を担う framework 共通サービス。

## 概要

サービス間通信のエンドポイント情報を一元管理する。

## 責務

- エンドポイント情報の取得（Get/List）
- サービス名からエンドポイントへの解決

## 公開API

### gRPC

- `proto/endpoint/v1/endpoint.proto`（予定）

主要 RPC:
- `GetEndpoint`: エンドポイント取得
- `ListEndpoints`: エンドポイント一覧取得

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

`migrations/` に配置（予定）

## 認証・認可

- エンドポイント取得は認証必須

## 監視

- メトリクス: エンドポイント取得成功/失敗数
- ログ: エンドポイント取得
- トレース: リクエスト単位

## 起動方法

```bash
cargo run -- --env dev --config ./config/dev.yaml --secrets-dir ./secrets/dev/
```

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

置き場のみ固定。実装はフェーズ26で行う。
