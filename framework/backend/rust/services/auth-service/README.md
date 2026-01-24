# auth-service

認証・認可を担う framework 共通サービス。

## 概要

ユーザー認証と権限管理の中央サービス。

## 責務

- ユーザー認証（ログイン、トークン発行）
- 権限チェック（`CheckPermission`）
- ロール管理

## 公開API

### gRPC

- `proto/auth/v1/auth.proto`（予定）

主要 RPC:
- `Authenticate`: 認証とトークン発行
- `CheckPermission`: 権限チェック
- `GetUser`: ユーザー情報取得

## 依存

- DB（PostgreSQL）: `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission`

## 設定

- `config/{env}.yaml`
- 秘密情報: JWT 秘密鍵（`auth.jwt_private_key_file`）

## DB

### 所有テーブル

| テーブル | 説明 |
|---------|------|
| `fw_m_user` | ユーザー |
| `fw_m_role` | ロール |
| `fw_m_permission` | 権限 |
| `fw_m_user_role` | ユーザー・ロール紐付け |
| `fw_m_role_permission` | ロール・権限紐付け |

### マイグレーション

`migrations/` に配置（予定）

## 認証・認可

本サービス自体が認証・認可の中央サービスである。

## 監視

- メトリクス: 認証成功/失敗数、権限チェック成功/失敗数
- ログ: 認証試行、権限チェック結果
- トレース: リクエスト単位

## 起動方法

```bash
cargo run -- --env dev --config ./config/dev.yaml --secrets-dir ./secrets/dev/
```

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

置き場のみ固定。実装はフェーズ27で行う。
