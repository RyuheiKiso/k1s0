# auth-service マイグレーション

## 所有テーブル

auth-serviceは以下のテーブルを所有します：

| テーブル名 | 説明 |
|-----------|------|
| `fw_m_user` | ユーザーマスタ |
| `fw_m_role` | ロールマスタ |
| `fw_m_permission` | パーミッションマスタ |
| `fw_m_user_role` | ユーザー・ロール関連 |
| `fw_m_role_permission` | ロール・パーミッション関連 |

## マイグレーションファイル

| ファイル | 説明 |
|---------|------|
| `0001_create_auth_tables.up.sql` | 認証・認可テーブルの作成 |
| `0001_create_auth_tables.down.sql` | 認証・認可テーブルの削除 |

## 実行方法

```bash
# マイグレーション実行
sqlx migrate run --source migrations

# ロールバック
sqlx migrate revert --source migrations
```

## 初期データ

マイグレーションには以下の初期データが含まれます：

### システムロール
- `admin` - システム管理者
- `user` - 一般ユーザー
- `viewer` - 閲覧専用ユーザー

### 基本パーミッション
- `user:read` - ユーザー情報の読み取り
- `user:write` - ユーザー情報の作成・更新
- `user:delete` - ユーザーの削除
- `role:read` - ロール情報の読み取り
- `role:write` - ロールの作成・更新
- `role:delete` - ロールの削除
- `permission:read` - パーミッション情報の読み取り
- `permission:write` - パーミッションの作成・更新
- `admin:all` - 管理者権限（すべての操作）

## 注意事項

- `fw_update_timestamp()` 関数は複数のサービスで共有されます
- システムロール（`is_system = TRUE`）は削除できません
- パスワードハッシュには bcrypt を使用してください
