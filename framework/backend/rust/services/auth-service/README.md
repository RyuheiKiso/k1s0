# auth-service

認証・認可を担う framework 共通サービス。

## 概要

ユーザー認証と権限管理の中央サービス。

## 責務

- ユーザー認証（ログイン、トークン発行）
- トークンリフレッシュ
- 権限チェック（`CheckPermission`）
- ロール管理

## 公開API

### gRPC

- `proto/auth/v1/auth.proto`

主要 RPC:
- `Authenticate`: 認証とトークン発行
- `RefreshToken`: トークンリフレッシュ
- `CheckPermission`: 権限チェック
- `GetUser`: ユーザー情報取得
- `ListUserRoles`: ユーザーのロール一覧取得

### ヘルスチェック

- gRPC Health Check Protocol (`grpc.health.v1.Health`)

## 依存

- DB（PostgreSQL）: `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission`, `fw_t_refresh_token`

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
| `fw_t_refresh_token` | リフレッシュトークン |

### マイグレーション

`migrations/` に配置

## 認証・認可

本サービス自体が認証・認可の中央サービスである。

## 監視

- メトリクス: 認証成功/失敗数、権限チェック成功/失敗数
- ログ: 認証試行、権限チェック結果（JSON形式）
- トレース: リクエスト単位

## 起動方法

```bash
# 基本起動（開発環境）
cargo run -- --env dev --port 50051

# オプション一覧
auth-service --help

# 主なオプション:
#   --env <ENV>           環境名 (dev, stg, prod) [default: dev]
#   --port, -p <PORT>     gRPCポート [default: 50051]
#   --config <PATH>       設定ファイルパス
#   --secrets-dir <PATH>  シークレットディレクトリ
#   --database-url <URL>  PostgreSQL接続URL（環境変数 DATABASE_URL も可）
#   --issuer <ISSUER>     JWT発行者 [default: k1s0-auth]
#   --jwt-secret <SECRET> JWT秘密鍵（開発用、本番はシークレットファイルを使用）
```

### 動作モード

- **InMemory**: `--database-url` なしで起動。開発・テスト用。
- **PostgreSQL**: `--database-url` 指定で起動（現在実装中）。

## リリース

- 本番リリース前に必ず `buf breaking` を通す
- 破壊的変更は ADR 必須

## ステータス

基本実装完了。InMemoryリポジトリで動作。PostgreSQLリポジトリは実装済み（統合未完了）。
