# config-service

動的設定を担う framework 共通サービス。

## 概要

`fw_m_setting` を扱い、feature サービスに動的設定を提供する。

## 責務

- 設定の取得（Get/List）
- 設定のキャッシュ
- 設定変更の通知（将来）

## 公開API

### gRPC

- `proto/config/v1/config.proto`（予定）

主要 RPC:
- `GetSetting`: 設定取得
- `ListSettings`: 設定一覧取得

## 依存

- DB（PostgreSQL）: `fw_m_setting`

## 設定

- `config/{env}.yaml`
- キャッシュ TTL、最大保持世代数

## DB

### 所有テーブル

| テーブル | 説明 |
|---------|------|
| `fw_m_setting` | 設定 |

### マイグレーション

`migrations/` に配置（予定）

## 認証・認可

- 設定取得は認証必須
- 設定変更は管理者権限必須

## 監視

- メトリクス: 設定取得成功/失敗数、キャッシュヒット率
- ログ: 設定取得、キャッシュ更新
- トレース: リクエスト単位

## 起動方法

```bash
cargo run -- --env dev --config ./config/dev.yaml --secrets-dir ./secrets/dev/
```

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

置き場のみ固定。実装はフェーズ24-25で行う。
